#![forbid(unsafe_code)]

//! IR to file tree to deterministic zip.
//!
//! The emitter is the only component that knows directory names, file
//! extensions, and format numbers, and it reads all of them from target data
//! (ADR-0006). Output is byte-identical for the same inputs: entries are sorted,
//! zip timestamps are fixed (ADR-0007).

use std::collections::BTreeMap;

use serde::Serialize;

use packsmith_ir::{Ir, Text};
use packsmith_mcversion::TargetData;

/// Why an IR document could not be emitted for a target.
#[derive(Debug, thiserror::Error)]
pub enum EmitError {
    #[error("target {target} does not support pack kind '{kind}'")]
    UnsupportedPackKind { target: String, kind: String },
}

/// The pack contents, path to bytes, exactly as they appear inside the built
/// zip. Sorted by path, so iteration order is the emit order (ADR-0007).
pub type FileTree = BTreeMap<String, Vec<u8>>;

/// Lower an IR document to its file tree.
pub fn file_tree(ir: &Ir, target: &TargetData) -> Result<FileTree, EmitError> {
    let mut tree = FileTree::new();
    for pack in &ir.packs {
        let format =
            target
                .pack_format(&pack.kind)
                .ok_or_else(|| EmitError::UnsupportedPackKind {
                    target: target.target().to_string(),
                    kind: pack.kind.clone(),
                })?;
        let meta = PackMcmeta::new(pack.description.clone(), format);
        tree.insert("pack.mcmeta".to_string(), meta.to_bytes());
        // Resources are not modelled yet (packsmith-ir). When they are, each
        // lands at `<root>/<namespace>/<category dir>/<path>.<ext>` from target
        // data, and the tree stays sorted for free (ROADMAP Phase 1).
    }
    Ok(tree)
}

/// Pack the file tree into a zip, deterministically: STORE (no compression, so
/// no compressor version or level reaches the bytes; ADR-0018), entries in
/// sorted order, every timestamp pinned to 1980-01-01 (ADR-0007).
pub fn zip(tree: &FileTree) -> Vec<u8> {
    // DOS time/date: 1980-01-01 00:00:00, the zero point of the DOS epoch.
    const DOS_TIME: u16 = 0;
    const DOS_DATE: u16 = 0b0000_0000_0010_0001;
    const VERSION: u16 = 20;

    let mut out = Vec::new();
    let mut central = Vec::new();

    for (name, data) in tree {
        let name = name.as_bytes();
        let crc = crc32(data);
        let size = to_u32(data.len());
        let offset = to_u32(out.len());

        out.extend_from_slice(&0x0403_4b50u32.to_le_bytes()); // local file header
        out.extend_from_slice(&VERSION.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes()); // flags
        out.extend_from_slice(&0u16.to_le_bytes()); // method: STORE
        out.extend_from_slice(&DOS_TIME.to_le_bytes());
        out.extend_from_slice(&DOS_DATE.to_le_bytes());
        out.extend_from_slice(&crc.to_le_bytes());
        out.extend_from_slice(&size.to_le_bytes()); // compressed
        out.extend_from_slice(&size.to_le_bytes()); // uncompressed
        out.extend_from_slice(&to_u16(name.len()).to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes()); // extra len
        out.extend_from_slice(name);
        out.extend_from_slice(data);

        central.extend_from_slice(&0x0201_4b50u32.to_le_bytes()); // central dir header
        central.extend_from_slice(&VERSION.to_le_bytes()); // version made by (FAT)
        central.extend_from_slice(&VERSION.to_le_bytes()); // version needed
        central.extend_from_slice(&0u16.to_le_bytes()); // flags
        central.extend_from_slice(&0u16.to_le_bytes()); // method: STORE
        central.extend_from_slice(&DOS_TIME.to_le_bytes());
        central.extend_from_slice(&DOS_DATE.to_le_bytes());
        central.extend_from_slice(&crc.to_le_bytes());
        central.extend_from_slice(&size.to_le_bytes());
        central.extend_from_slice(&size.to_le_bytes());
        central.extend_from_slice(&to_u16(name.len()).to_le_bytes());
        central.extend_from_slice(&0u16.to_le_bytes()); // extra len
        central.extend_from_slice(&0u16.to_le_bytes()); // comment len
        central.extend_from_slice(&0u16.to_le_bytes()); // disk number
        central.extend_from_slice(&0u16.to_le_bytes()); // internal attrs
        central.extend_from_slice(&0u32.to_le_bytes()); // external attrs
        central.extend_from_slice(&offset.to_le_bytes());
        central.extend_from_slice(name);
    }

    let cd_offset = to_u32(out.len());
    let cd_size = to_u32(central.len());
    let count = to_u16(tree.len());
    out.extend_from_slice(&central);

    out.extend_from_slice(&0x0605_4b50u32.to_le_bytes()); // end of central directory
    out.extend_from_slice(&0u16.to_le_bytes()); // this disk
    out.extend_from_slice(&0u16.to_le_bytes()); // disk with central dir
    out.extend_from_slice(&count.to_le_bytes());
    out.extend_from_slice(&count.to_le_bytes());
    out.extend_from_slice(&cd_size.to_le_bytes());
    out.extend_from_slice(&cd_offset.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes()); // comment len
    out
}

/// `pack.mcmeta` in the modern shape, required since 25w31a / 1.21.9
/// (`.claude/rules/minecraft.md`): `min_format` is the `[major, minor]` of the
/// requested target, `max_format` is the bare major. Minor format bumps are
/// additive, so a pack built against 107.1 is equally valid at 107.5; pinning
/// the upper bound to `[107, 1]` would instead warn the user, unfixably, on
/// every point release.
///
/// Only the modern shape is emitted. The legacy single-integer `pack_format`
/// shape, and choosing between the two from target data, is Phase 2 work
/// (ROADMAP): no target this compiler supports needs it.
#[derive(Serialize)]
struct PackMcmeta {
    pack: PackSection,
}

#[derive(Serialize)]
struct PackSection {
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<Text>,
    min_format: [u32; 2],
    max_format: u32,
}

impl PackMcmeta {
    fn new(description: Option<Text>, format: packsmith_mcversion::PackFormat) -> Self {
        Self {
            pack: PackSection {
                description,
                min_format: [format.major, format.minor],
                max_format: format.major,
            },
        }
    }

    /// Compact JSON with a trailing newline. Field order is the struct's, so it
    /// is deterministic without canonicalization.
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = serde_json::to_vec(self).unwrap_or_default();
        bytes.push(b'\n');
        bytes
    }
}

fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &byte in bytes {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xEDB8_8320 & mask);
        }
    }
    !crc
}

fn to_u32(n: usize) -> u32 {
    u32::try_from(n).unwrap_or(u32::MAX)
}

fn to_u16(n: usize) -> u16 {
    u16::try_from(n).unwrap_or(u16::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn target() -> TargetData {
        TargetData::load(&packsmith_mcversion::bundled_data_dir(), "26.2")
            .expect("26.2 data ships with packsmith-mcversion")
    }

    fn empty_pack_ir() -> Ir {
        Ir {
            version: 0,
            target: packsmith_ir::Target {
                id: "26.2".to_string(),
            },
            packs: vec![packsmith_ir::Pack {
                kind: "data".to_string(),
                description: Some(Text::from("An empty Packsmith project.")),
                resources: Vec::new(),
            }],
        }
    }

    #[test]
    fn empty_pack_is_one_pack_mcmeta_shaped_from_target_data() {
        let tree = file_tree(&empty_pack_ir(), &target()).expect("emits");
        assert_eq!(tree.keys().collect::<Vec<_>>(), vec!["pack.mcmeta"]);
        // 26.2 data pack format is 107.1 (.claude/rules/minecraft.md), read from
        // target data, not written here.
        assert_eq!(
            String::from_utf8(tree["pack.mcmeta"].clone()).unwrap(),
            "{\"pack\":{\"description\":\"An empty Packsmith project.\",\
             \"min_format\":[107,1],\"max_format\":107}}\n"
        );
    }

    #[test]
    fn an_unsupported_pack_kind_is_a_diagnostic_not_a_panic() {
        let mut ir = empty_pack_ir();
        ir.packs[0].kind = "worldgen".to_string();
        assert!(matches!(
            file_tree(&ir, &target()),
            Err(EmitError::UnsupportedPackKind { .. })
        ));
    }

    #[test]
    fn zip_is_byte_identical_across_runs() {
        let tree = file_tree(&empty_pack_ir(), &target()).expect("emits");
        assert_eq!(zip(&tree), zip(&tree));
    }

    #[test]
    fn zip_has_a_local_header_and_an_end_of_central_directory() {
        let bytes = zip(&file_tree(&empty_pack_ir(), &target()).expect("emits"));
        assert_eq!(&bytes[..4], b"PK\x03\x04");
        assert_eq!(&bytes[bytes.len() - 22..bytes.len() - 18], b"PK\x05\x06");
    }
}
