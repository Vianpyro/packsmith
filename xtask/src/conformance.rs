//! Conformance runner: build each verified case with the real `packsmith`
//! binary, check the built tree against its `expected/`, and build it twice to
//! confirm the bytes are identical (ADR-0007).
//!
//! The structural check (every case is well-formed) lives in `main.rs`; this is
//! the half that needs a working compiler and emitter.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Deserialize;

/// Build and verify every case whose `expected/` tree has been hand-checked
/// against a real game instance. Returns how many were run.
///
/// A case is skipped while its `expected/` still holds a `PLACEHOLDER.md`: that
/// marker means the tree has not been verified in game yet, so there is nothing
/// trustworthy to compare against.
pub(crate) fn run_verified_cases(cases_dir: &Path) -> Result<usize, Vec<String>> {
    if !crate::run_cargo(&["build", "-p", "packsmith-cli", "--locked"]) {
        return Err(vec!["cargo build -p packsmith-cli failed".to_string()]);
    }
    let bin = cli_binary();
    if !bin.is_file() {
        return Err(vec![format!("cli binary not found at {}", bin.display())]);
    }

    let mut names: Vec<String> = match std::fs::read_dir(cases_dir) {
        Ok(entries) => entries
            .filter_map(Result::ok)
            .filter(|e| e.path().is_dir())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect(),
        Err(e) => return Err(vec![format!("cannot read {}: {e}", cases_dir.display())]),
    };
    names.sort();

    let mut problems = Vec::new();
    let mut ran = 0;
    for name in &names {
        let case = cases_dir.join(name);
        let expected = case.join("expected");
        if !expected.is_dir() || expected.join("PLACEHOLDER.md").is_file() {
            continue;
        }
        ran += 1;
        if let Err(e) = verify_case(&bin, &case, name) {
            problems.push(format!("{name}: {e}"));
        }
    }

    if problems.is_empty() {
        Ok(ran)
    } else {
        Err(problems)
    }
}

fn verify_case(bin: &Path, case: &Path, name: &str) -> Result<(), String> {
    let target = read_target_id(&case.join("target.json"))?;
    let input = case.join("input.json");

    let work = std::env::temp_dir().join(format!("packsmith-conformance-{name}"));
    let _ = std::fs::remove_dir_all(&work);
    std::fs::create_dir_all(&work).map_err(|e| format!("creating {}: {e}", work.display()))?;

    let zip1 = work.join("build-1.zip");
    let zip2 = work.join("build-2.zip");
    run_build(bin, &input, &target, &zip1)?;
    run_build(bin, &input, &target, &zip2)?;

    let bytes1 = std::fs::read(&zip1).map_err(|e| format!("reading {}: {e}", zip1.display()))?;
    let bytes2 = std::fs::read(&zip2).map_err(|e| format!("reading {}: {e}", zip2.display()))?;
    if bytes1 != bytes2 {
        return Err("two builds produced different zip bytes (ADR-0007)".to_string());
    }

    let produced = read_store_zip(&bytes1)?;
    let expected = read_dir_tree(&case.join("expected"))?;
    compare(&expected, &produced)?;

    let _ = std::fs::remove_dir_all(&work);
    Ok(())
}

fn run_build(bin: &Path, input: &Path, target: &str, output: &Path) -> Result<(), String> {
    let status = Command::new(bin)
        .arg("build")
        .arg(input)
        .arg("--target")
        .arg(target)
        .arg("--output")
        .arg(output)
        .status()
        .map_err(|e| format!("running {}: {e}", bin.display()))?;
    if !status.success() {
        return Err(format!("`packsmith build` exited with {status}"));
    }
    Ok(())
}

fn cli_binary() -> PathBuf {
    let target_dir = std::env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| crate::repo_root().join("target"));
    let name = if cfg!(windows) {
        "packsmith.exe"
    } else {
        "packsmith"
    };
    target_dir.join("debug").join(name)
}

#[derive(Deserialize)]
struct TargetFile {
    id: String,
}

fn read_target_id(path: &Path) -> Result<String, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("reading {}: {e}", path.display()))?;
    let parsed: TargetFile = serde_json::from_slice(&bytes)
        .map_err(|e| format!("{} is not a valid target: {e}", path.display()))?;
    Ok(parsed.id)
}

/// Read a STORE-only zip back into a path-to-bytes map, walking local file
/// headers in order. The emitter never compresses and never writes a data
/// descriptor (`packsmith-emit`), so this is all the zip parsing the runner
/// needs.
fn read_store_zip(bytes: &[u8]) -> Result<BTreeMap<String, Vec<u8>>, String> {
    let mut tree = BTreeMap::new();
    let mut p = 0usize;
    while bytes.get(p..p + 4) == Some(b"PK\x03\x04".as_slice()) {
        let method = u16_at(bytes, p + 8).ok_or("truncated local file header")?;
        if method != 0 {
            return Err("zip entry is compressed; the emitter must use STORE".to_string());
        }
        let comp = u32_at(bytes, p + 18).ok_or("truncated local file header")? as usize;
        let nlen = usize::from(u16_at(bytes, p + 26).ok_or("truncated local file header")?);
        let elen = usize::from(u16_at(bytes, p + 28).ok_or("truncated local file header")?);
        let name_start = p + 30;
        let name_bytes = bytes
            .get(name_start..name_start + nlen)
            .ok_or("truncated entry name")?;
        let name = std::str::from_utf8(name_bytes)
            .map_err(|_| "entry name is not UTF-8")?
            .to_string();
        let data_start = name_start + nlen + elen;
        let data = bytes
            .get(data_start..data_start + comp)
            .ok_or("truncated entry data")?;
        tree.insert(name, data.to_vec());
        p = data_start + comp;
    }
    Ok(tree)
}

fn u16_at(bytes: &[u8], offset: usize) -> Option<u16> {
    bytes
        .get(offset..offset + 2)
        .map(|s| u16::from_le_bytes([s[0], s[1]]))
}

fn u32_at(bytes: &[u8], offset: usize) -> Option<u32> {
    bytes
        .get(offset..offset + 4)
        .map(|s| u32::from_le_bytes([s[0], s[1], s[2], s[3]]))
}

fn read_dir_tree(root: &Path) -> Result<BTreeMap<String, Vec<u8>>, String> {
    let mut tree = BTreeMap::new();
    collect(root, root, &mut tree)?;
    Ok(tree)
}

fn collect(root: &Path, dir: &Path, tree: &mut BTreeMap<String, Vec<u8>>) -> Result<(), String> {
    let entries = std::fs::read_dir(dir).map_err(|e| format!("reading {}: {e}", dir.display()))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("reading {}: {e}", dir.display()))?;
        let path = entry.path();
        if path.is_dir() {
            collect(root, &path, tree)?;
        } else {
            let rel = path
                .strip_prefix(root)
                .map_err(|e| e.to_string())?
                .to_string_lossy()
                .replace('\\', "/");
            let bytes =
                std::fs::read(&path).map_err(|e| format!("reading {}: {e}", path.display()))?;
            tree.insert(rel, bytes);
        }
    }
    Ok(())
}

fn compare(
    expected: &BTreeMap<String, Vec<u8>>,
    produced: &BTreeMap<String, Vec<u8>>,
) -> Result<(), String> {
    let want: Vec<&String> = expected.keys().collect();
    let got: Vec<&String> = produced.keys().collect();
    if want != got {
        return Err(format!(
            "file set differs: expected {want:?}, built {got:?}"
        ));
    }
    for (name, want) in expected {
        let Some(have) = produced.get(name) else {
            continue;
        };
        if want != have {
            return Err(format!(
                "{name} differs:\n    expected: {}\n    built:    {}",
                String::from_utf8_lossy(want).trim_end(),
                String::from_utf8_lossy(have).trim_end()
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A minimal STORE zip: one entry, `a.txt` -> `hi`, no compression, no data
    /// descriptor. Enough to lock the parser without pulling in the emitter.
    fn store_zip_one_entry() -> Vec<u8> {
        let name = b"a.txt";
        let data = b"hi";
        let mut z = Vec::new();
        z.extend_from_slice(b"PK\x03\x04");
        z.extend_from_slice(&20u16.to_le_bytes());
        z.extend_from_slice(&0u16.to_le_bytes());
        z.extend_from_slice(&0u16.to_le_bytes()); // STORE
        z.extend_from_slice(&0u16.to_le_bytes());
        z.extend_from_slice(&0u16.to_le_bytes());
        z.extend_from_slice(&0u32.to_le_bytes()); // crc, unchecked by the reader
        z.extend_from_slice(&(data.len() as u32).to_le_bytes());
        z.extend_from_slice(&(data.len() as u32).to_le_bytes());
        z.extend_from_slice(&(name.len() as u16).to_le_bytes());
        z.extend_from_slice(&0u16.to_le_bytes());
        z.extend_from_slice(name);
        z.extend_from_slice(data);
        z.extend_from_slice(b"PK\x05\x06"); // truncated central dir is fine: reader stops here
        z
    }

    #[test]
    fn reads_a_store_entry_back() {
        let tree = read_store_zip(&store_zip_one_entry()).expect("parses");
        assert_eq!(tree.get("a.txt").map(Vec::as_slice), Some(b"hi".as_slice()));
    }

    #[test]
    fn compare_flags_a_byte_difference() {
        let mut a = BTreeMap::new();
        a.insert("pack.mcmeta".to_string(), b"one".to_vec());
        let mut b = BTreeMap::new();
        b.insert("pack.mcmeta".to_string(), b"two".to_vec());
        assert!(compare(&a, &b).is_err());
        assert!(compare(&a, &a).is_ok());
    }

    #[test]
    fn compare_flags_a_missing_file() {
        let mut a = BTreeMap::new();
        a.insert("pack.mcmeta".to_string(), b"x".to_vec());
        let b = BTreeMap::new();
        assert!(compare(&a, &b).is_err());
    }
}
