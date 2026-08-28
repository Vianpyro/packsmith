#![forbid(unsafe_code)]

//! Minecraft: Java Edition target data, loaded at runtime.
//!
//! The compiler and emitter never hardcode a format number, a directory name,
//! or a registry identifier (ADR-0006). All of it lives in a per-target data
//! file under `data/<version>.json`, produced by `cargo xtask sync-target` from
//! a pinned `misode/mcmeta` commit (ADR-0014). That data is derived from
//! Minecraft and is not covered by Packsmith's licence grants (ADR-0015); it is
//! read from a file here, never `include_str!`d into a binary, so it stays an
//! aggregate alongside our code rather than a component compiled into it.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;

/// Everything the compiler and emitter need to know about one target release.
///
/// Deserialized straight from `data/<version>.json`. Fields that only document
/// provenance are kept out of this type; see the file itself for those.
#[derive(Debug, Clone, Deserialize)]
pub struct TargetData {
    target: String,
    pack_formats: BTreeMap<String, PackFormat>,
    pack_kinds: BTreeMap<String, PackKind>,
    categories: BTreeMap<String, Category>,
    registries: Registries,
    commands: serde_json::Value,
}

/// A pack format version. The major number is what the game compares; the minor
/// number distinguishes additive revisions within a major (`min_format` /
/// `max_format` since 1.21.9).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct PackFormat {
    pub major: u32,
    pub minor: u32,
}

/// One kind of pack (`data`, `resource`, ...). The set is open (ADR-0010): the
/// emitter looks a kind up here rather than matching an enum.
#[derive(Debug, Clone, Deserialize)]
pub struct PackKind {
    /// Root directory inside the zip, e.g. `data` or `assets`.
    pub root: String,
    /// Key into [`TargetData::pack_format`] for this kind's format number.
    pub format: String,
}

/// Where a resource category's files land and what extension they carry. The set
/// of categories is open (ADR-0010): an unknown category is a "not supported for
/// this target" diagnostic, not a parse error.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Category {
    /// Path under the pack root, e.g. `function` or `tags/function`.
    pub directory: String,
    /// File extension with no leading dot, e.g. `mcfunction` or `json`.
    pub extension: String,
}

/// Registry identifier lists used for validation and completion. Namespaced
/// (`minecraft:stone`). Only the lists ADR-0014 names are carried.
#[derive(Debug, Clone, Deserialize)]
struct Registries {
    block: Vec<String>,
    item: Vec<String>,
    entity_type: Vec<String>,
}

/// Why a target's data could not be loaded. `NotFound` is separate so callers
/// can turn "unknown target" into a diagnostic while still surfacing a corrupt
/// file as a hard error.
#[derive(Debug, thiserror::Error)]
pub enum LoadError {
    #[error("no target data for '{version}' (looked for {})", path.display())]
    NotFound { version: String, path: PathBuf },
    #[error("reading target data for '{version}' from {}: {source}", path.display())]
    Io {
        version: String,
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("parsing target data for '{version}' from {}: {source}", path.display())]
    Parse {
        version: String,
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
}

impl TargetData {
    /// Load the data file for `version` from `data_dir`.
    ///
    /// A missing file is [`LoadError::NotFound`]: the caller asked for a target
    /// Packsmith has no data for, and that must fail loudly rather than fall
    /// back to a guess (ADR-0006).
    pub fn load(data_dir: &Path, version: &str) -> Result<Self, LoadError> {
        let path = data_dir.join(format!("{version}.json"));
        let bytes = match std::fs::read(&path) {
            Ok(bytes) => bytes,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Err(LoadError::NotFound {
                    version: version.to_string(),
                    path,
                });
            }
            Err(source) => {
                return Err(LoadError::Io {
                    version: version.to_string(),
                    path,
                    source,
                });
            }
        };
        serde_json::from_slice(&bytes).map_err(|source| LoadError::Parse {
            version: version.to_string(),
            path,
            source,
        })
    }

    /// The release this data describes, e.g. `26.2`.
    pub fn target(&self) -> &str {
        &self.target
    }

    /// The pack format for a pack kind (`data`, `resource`), resolved through
    /// the kind's `format` key. `None` if the kind is unknown for this target.
    pub fn pack_format(&self, kind: &str) -> Option<PackFormat> {
        let key = &self.pack_kinds.get(kind)?.format;
        self.pack_formats.get(key).copied()
    }

    /// The pack kind entry (`root`, `format`). `None` if unknown for this target.
    pub fn pack_kind(&self, kind: &str) -> Option<&PackKind> {
        self.pack_kinds.get(kind)
    }

    /// Directory and extension for a resource category. `None` if the target
    /// does not define it (ADR-0010: a diagnostic, not a parse error).
    pub fn category(&self, name: &str) -> Option<&Category> {
        self.categories.get(name)
    }

    /// Every category this target defines, keyed by category string.
    pub fn categories(&self) -> &BTreeMap<String, Category> {
        &self.categories
    }

    /// Block identifiers, namespaced.
    pub fn blocks(&self) -> &[String] {
        &self.registries.block
    }

    /// Item identifiers, namespaced.
    pub fn items(&self) -> &[String] {
        &self.registries.item
    }

    /// Entity type identifiers, namespaced.
    pub fn entity_types(&self) -> &[String] {
        &self.registries.entity_type
    }

    /// The Brigadier command tree, as published by mcmeta with the non-grammar
    /// `permissions` fields removed.
    pub fn commands(&self) -> &serde_json::Value {
        &self.commands
    }
}

/// The `data/` directory of this crate as built, for the CLI and tests.
///
/// A hosted or WASM build passes its own directory to [`TargetData::load`]
/// instead; this is a path, not embedded data.
pub fn bundled_data_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("data")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn target_26_2() -> TargetData {
        TargetData::load(&bundled_data_dir(), "26.2").expect("26.2 data ships with the crate")
    }

    #[test]
    fn missing_target_is_not_found_not_a_panic() {
        let err = TargetData::load(&bundled_data_dir(), "0.0")
            .expect_err("there is no data file for 0.0");
        assert!(matches!(err, LoadError::NotFound { .. }));
    }

    #[test]
    fn data_pack_format_is_resolved_through_the_kind() {
        let t = target_26_2();
        let fmt = t
            .pack_format("data")
            .expect("data kind is defined for 26.2");
        // Released 26.2 (misode/mcmeta 26.2-summary). Not the 107.0 pre-release
        // quoted in ADR-0014.
        assert_eq!(
            fmt,
            PackFormat {
                major: 107,
                minor: 1
            }
        );
        assert!(t.pack_format("nonsense").is_none());
    }

    #[test]
    fn function_category_lands_where_target_data_says() {
        let t = target_26_2();
        let f = t
            .category("function")
            .expect("function category is defined");
        assert_eq!(f.directory, "function");
        assert_eq!(f.extension, "mcfunction");
        assert_eq!(
            t.category("tags/function").map(|c| c.extension.as_str()),
            Some("json")
        );
        assert!(t.category("worldgen/biome").is_none(), "not a v1 category");
    }

    #[test]
    fn registry_lists_are_namespaced_and_populated() {
        let t = target_26_2();
        assert!(t.blocks().contains(&"minecraft:stone".to_string()));
        assert!(t.items().contains(&"minecraft:stick".to_string()));
        assert!(t.entity_types().contains(&"minecraft:creeper".to_string()));
    }

    #[test]
    fn command_tree_carries_grammar_but_not_permissions() {
        let t = target_26_2();
        let root = t.commands();
        assert_eq!(root["type"], "root");
        assert!(root["children"]["say"].is_object());
        assert!(
            find_key(root, "permissions").is_none(),
            "permissions are stripped: they are not grammar"
        );
    }

    fn find_key<'a>(v: &'a serde_json::Value, key: &str) -> Option<&'a serde_json::Value> {
        match v {
            serde_json::Value::Object(map) => {
                if let Some(found) = map.get(key) {
                    return Some(found);
                }
                map.values().find_map(|child| find_key(child, key))
            }
            serde_json::Value::Array(items) => items.iter().find_map(|child| find_key(child, key)),
            _ => None,
        }
    }
}
