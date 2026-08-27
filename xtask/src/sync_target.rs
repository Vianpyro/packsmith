//! `cargo xtask sync-target --version <v>`: fetch target data from a pinned
//! `misode/mcmeta` commit, extract the thin functional subset ADR-0014 names,
//! and write `crates/packsmith-mcversion/data/<v>.json` with a provenance header
//! and the ADR-0015 SPDX marker.
//!
//! What is extracted, and nothing more:
//!   - data and resource pack format (major + minor), from the summary branch
//!     `version.json`;
//!   - the seven v1 resource categories and their directory + extension
//!     (ADR-0010, OPEN-QUESTIONS A3);
//!   - block, item, and entity_type identifier lists, from `registries/data.json`;
//!   - the Brigadier command tree, from `commands/data.json`, with the
//!     non-grammar `permissions` fields removed.
//!
//! mcmeta is fetched with a shallow, blobless, sparse `git` clone. It is never a
//! submodule (ADR-0014) and network access is not a `cargo build` dependency:
//! this runs only when a person runs the task.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use serde::Serialize;

const MCMETA_REMOTE: &str = "https://github.com/misode/mcmeta.git";
const MCMETA_SOURCE_URL: &str = "https://github.com/misode/mcmeta";

/// A pinned mcmeta `summary` state for one target. The commit is the integrity
/// anchor (ADR-0014); the tag is what a person reads. Add a row here to support
/// a new target, in its own commit alongside the generated data file.
struct Pin {
    commit: &'static str,
    tag: &'static str,
}

fn pin_for(version: &str) -> Option<Pin> {
    match version {
        // misode/mcmeta `26.2-summary`, the summary-branch state published for
        // released 26.2 (2026-06-16). Confirmed against the released
        // `version.json`: data pack format 107.1, not the 107.0 pre-release
        // quoted in ADR-0014.
        "26.2" => Some(Pin {
            commit: "711a353b47d84e6cb592a1b72f682e5f44759284",
            tag: "26.2-summary",
        }),
        _ => None,
    }
}

/// The seven v1 resource categories (ADR-0010, OPEN-QUESTIONS A3): the category
/// string, its directory under the pack root, and its file extension.
///
/// `recipe`, `loot_table`, and `advancement` are cross-checked against mcmeta's
/// registry list below and the run fails if mcmeta no longer agrees. `function`,
/// `tags/function`, `predicate`, and `item_modifier` are not datapack registries
/// in any Minecraft-generated report (the game special-cases them in
/// `ServerFunctionLibrary` and `LootDataType`); their directory and extension
/// are recorded here and documented in the ADR-0014 amendment.
const V1_CATEGORIES: &[(&str, &str, &str)] = &[
    ("advancement", "advancement", "json"),
    ("function", "function", "mcfunction"),
    ("item_modifier", "item_modifier", "json"),
    ("loot_table", "loot_table", "json"),
    ("predicate", "predicate", "json"),
    ("recipe", "recipe", "json"),
    ("tags/function", "tags/function", "json"),
];

/// Categories whose directory name is taken from mcmeta and must still be a
/// registry there, so a rename upstream is caught rather than silently carried.
const REGISTRY_BACKED: &[&str] = &["advancement", "loot_table", "recipe"];

pub fn run(mut args: impl Iterator<Item = String>) -> Result<()> {
    let mut version: Option<String> = None;
    let mut check = false;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--version" => {
                version = Some(args.next().context("--version needs a value, e.g. 26.2")?);
            }
            "--check" => check = true,
            other => bail!("sync-target: unexpected argument '{other}'"),
        }
    }
    let version = version.context("sync-target: --version <v> is required")?;
    let pin = pin_for(&version).with_context(|| {
        format!("no mcmeta pin for '{version}'; add one to xtask/src/sync_target.rs")
    })?;

    let work = TempClone::create(&version)?;
    fetch(&work.path, &pin)?;

    let head = git(&work.path, &["rev-parse", "HEAD"])?;
    if head.trim() != pin.commit {
        bail!(
            "mcmeta {} resolved to {} but the pin is {}; the tag moved",
            pin.tag,
            head.trim(),
            pin.commit
        );
    }

    let version_json: VersionJson = read_json(&work.path.join("version.json"))?;
    let registries: BTreeMap<String, serde_json::Value> =
        read_json(&work.path.join("registries/data.json"))?;
    let commands: serde_json::Value = read_json(&work.path.join("commands/data.json"))?;

    let inputs = ["version.json", "registries/data.json", "commands/data.json"]
        .into_iter()
        .map(|p| {
            Ok((
                p.to_string(),
                git(&work.path, &["rev-parse", &format!("HEAD:{p}")])?
                    .trim()
                    .to_string(),
            ))
        })
        .collect::<Result<BTreeMap<_, _>>>()?;

    let target = build_target(
        &version,
        &version_json,
        &registries,
        &commands,
        &pin,
        inputs,
    )?;
    let mut json = serde_json::to_string_pretty(&target).context("serializing target data")?;
    json.push('\n');

    let out_path = repo_root().join(format!("crates/packsmith-mcversion/data/{version}.json"));
    if check {
        verify(&out_path, &json)?;
        eprintln!("sync-target: {version} up to date");
    } else {
        std::fs::create_dir_all(out_path.parent().expect("data path has a parent"))
            .with_context(|| format!("creating {}", out_path.display()))?;
        std::fs::write(&out_path, &json)
            .with_context(|| format!("writing {}", out_path.display()))?;
        eprintln!("sync-target: wrote {}", out_path.display());
        eprintln!(
            "  data pack format {}.{}, resource pack format {}.{}",
            version_json.data_pack_version,
            version_json.data_pack_version_minor,
            version_json.resource_pack_version,
            version_json.resource_pack_version_minor,
        );
    }
    Ok(())
}

fn fetch(dir: &Path, pin: &Pin) -> Result<()> {
    let status = Command::new("git")
        .args([
            "-c",
            "advice.detachedHead=false",
            "clone",
            "--depth",
            "1",
            "--branch",
            pin.tag,
            "--filter=blob:none",
            "--sparse",
            MCMETA_REMOTE,
        ])
        .arg(dir)
        .status()
        .context("running git clone (is git on PATH?)")?;
    if !status.success() {
        bail!("git clone of mcmeta {} failed", pin.tag);
    }
    git(
        dir,
        &[
            "sparse-checkout",
            "set",
            "--no-cone",
            "/version.json",
            "/registries/data.json",
            "/commands/data.json",
        ],
    )?;
    Ok(())
}

fn build_target(
    version: &str,
    v: &VersionJson,
    registries: &BTreeMap<String, serde_json::Value>,
    commands: &serde_json::Value,
    pin: &Pin,
    inputs: BTreeMap<String, String>,
) -> Result<TargetFile> {
    let mut categories = BTreeMap::new();
    for (name, directory, extension) in V1_CATEGORIES {
        if REGISTRY_BACKED.contains(name) && !registries.get(*name).is_some_and(|r| r.is_array()) {
            bail!(
                "mcmeta {} no longer lists registry '{name}'; the category table needs review",
                pin.tag
            );
        }
        categories.insert(
            (*name).to_string(),
            Category {
                directory: (*directory).to_string(),
                extension: (*extension).to_string(),
            },
        );
    }

    let pack_formats = BTreeMap::from([
        (
            "data".to_string(),
            PackFormat {
                major: v.data_pack_version,
                minor: v.data_pack_version_minor,
            },
        ),
        (
            "resource".to_string(),
            PackFormat {
                major: v.resource_pack_version,
                minor: v.resource_pack_version_minor,
            },
        ),
    ]);
    // `data` and `assets` are Minecraft's fixed pack-root directory names; they
    // are structural, not in any generated report. Documented in the ADR-0014
    // amendment. v1 emits only the `data` kind.
    let pack_kinds = BTreeMap::from([
        (
            "data".to_string(),
            PackKind {
                root: "data".to_string(),
                format: "data".to_string(),
            },
        ),
        (
            "resource".to_string(),
            PackKind {
                root: "assets".to_string(),
                format: "resource".to_string(),
            },
        ),
    ]);

    Ok(TargetFile {
        spdx: "LicenseRef-Minecraft-Derived",
        comment: "Derived from Minecraft: Java Edition via misode/mcmeta. Not covered by \
                  Packsmith's licence grants; see NOTICE, ADR-0014, ADR-0015. Generated by \
                  `cargo xtask sync-target`; do not edit by hand.",
        provenance: Provenance {
            source_url: MCMETA_SOURCE_URL,
            mcmeta_commit: pin.commit,
            mcmeta_tag: pin.tag,
            mcmeta_version_id: v.id.clone(),
            extracted: today(),
            inputs,
        },
        target: version.to_string(),
        pack_formats,
        pack_kinds,
        categories,
        registries: Registries {
            block: id_list(registries, "block")?,
            item: id_list(registries, "item")?,
            entity_type: id_list(registries, "entity_type")?,
        },
        commands: strip_permissions(commands.clone()),
    })
}

fn id_list(registries: &BTreeMap<String, serde_json::Value>, name: &str) -> Result<Vec<String>> {
    let raw = registries
        .get(name)
        .and_then(|v| v.as_array())
        .with_context(|| format!("mcmeta registries has no array for '{name}'"))?;
    let mut ids: Vec<String> = raw
        .iter()
        .filter_map(|v| v.as_str())
        .map(|s| {
            if s.contains(':') {
                s.to_string()
            } else {
                format!("minecraft:{s}")
            }
        })
        .collect();
    ids.sort();
    ids.dedup();
    Ok(ids)
}

/// Remove every `permissions` key from the command tree. It carries the command
/// permission level, which does not decide whether a command string parses, so
/// it is not part of the grammar the validator (ADR-0012) needs.
fn strip_permissions(value: serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Object(map) => serde_json::Value::Object(
            map.into_iter()
                .filter(|(k, _)| k != "permissions")
                .map(|(k, v)| (k, strip_permissions(v)))
                .collect(),
        ),
        serde_json::Value::Array(items) => {
            serde_json::Value::Array(items.into_iter().map(strip_permissions).collect())
        }
        other => other,
    }
}

fn verify(path: &Path, expected: &str) -> Result<()> {
    let actual = std::fs::read_to_string(path).with_context(|| {
        format!(
            "{} does not exist; run `cargo xtask sync-target` without --check",
            path.display()
        )
    })?;
    if strip_extracted(&actual) != strip_extracted(expected) {
        bail!(
            "{} is stale; re-run `cargo xtask sync-target --version` to regenerate",
            path.display()
        );
    }
    Ok(())
}

/// Blank the one non-deterministic provenance field so `--check` compares the
/// extracted substance, not the day it last ran.
fn strip_extracted(json: &str) -> String {
    json.lines()
        .map(|line| {
            if line.trim_start().starts_with("\"extracted\":") {
                "    \"extracted\": \"\","
            } else {
                line
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn today() -> String {
    let days = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() / 86_400)
        .unwrap_or(0) as i64;
    let (y, m, d) = civil_from_days(days);
    format!("{y:04}-{m:02}-{d:02}")
}

/// Howard Hinnant's days-to-civil-date algorithm. `days` is days since
/// 1970-01-01.
fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (if m <= 2 { y + 1 } else { y }, m, d)
}

fn git(dir: &Path, args: &[&str]) -> Result<String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .with_context(|| format!("running git {}", args.join(" ")))?;
    if !out.status.success() {
        bail!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    String::from_utf8(out.stdout).context("git output was not utf-8")
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    let bytes = std::fs::read(path).with_context(|| format!("reading {}", path.display()))?;
    serde_json::from_slice(&bytes).with_context(|| format!("parsing {}", path.display()))
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask sits one level below the workspace root")
        .to_path_buf()
}

/// A throwaway clone directory, removed on drop.
struct TempClone {
    path: PathBuf,
}

impl TempClone {
    fn create(version: &str) -> Result<Self> {
        let path = std::env::temp_dir().join(format!(
            "packsmith-sync-target-{version}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&path);
        Ok(Self { path })
    }
}

impl Drop for TempClone {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

#[derive(Serialize)]
struct TargetFile {
    #[serde(rename = "SPDX-License-Identifier")]
    spdx: &'static str,
    #[serde(rename = "_comment")]
    comment: &'static str,
    provenance: Provenance,
    target: String,
    pack_formats: BTreeMap<String, PackFormat>,
    pack_kinds: BTreeMap<String, PackKind>,
    categories: BTreeMap<String, Category>,
    registries: Registries,
    commands: serde_json::Value,
}

#[derive(Serialize)]
struct Provenance {
    source_url: &'static str,
    mcmeta_commit: &'static str,
    mcmeta_tag: &'static str,
    mcmeta_version_id: String,
    extracted: String,
    inputs: BTreeMap<String, String>,
}

#[derive(Serialize)]
struct PackFormat {
    major: u32,
    minor: u32,
}

#[derive(Serialize)]
struct PackKind {
    root: String,
    format: String,
}

#[derive(Serialize)]
struct Category {
    directory: String,
    extension: String,
}

#[derive(Serialize)]
struct Registries {
    block: Vec<String>,
    item: Vec<String>,
    entity_type: Vec<String>,
}

#[derive(serde::Deserialize)]
struct VersionJson {
    id: String,
    data_pack_version: u32,
    #[serde(default)]
    data_pack_version_minor: u32,
    resource_pack_version: u32,
    #[serde(default)]
    resource_pack_version_minor: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn civil_from_days_matches_known_dates() {
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        assert_eq!(civil_from_days(18_993), (2022, 1, 1));
        assert_eq!(civil_from_days(20_663), (2026, 7, 29));
    }

    #[test]
    fn strip_permissions_is_recursive() {
        let input = serde_json::json!({
            "type": "root",
            "permissions": {"level": "gamemasters"},
            "children": {
                "say": {"type": "literal", "permissions": {"level": "gamemasters"}}
            }
        });
        let out = strip_permissions(input);
        assert!(out.get("permissions").is_none());
        assert!(out["children"]["say"].get("permissions").is_none());
        assert_eq!(out["children"]["say"]["type"], "literal");
    }

    #[test]
    fn id_list_namespaces_and_sorts() {
        let registries = BTreeMap::from([(
            "block".to_string(),
            serde_json::json!(["stone", "minecraft:acacia_door", "dirt"]),
        )]);
        assert_eq!(
            id_list(&registries, "block").unwrap(),
            vec!["minecraft:acacia_door", "minecraft:dirt", "minecraft:stone"]
        );
    }
}
