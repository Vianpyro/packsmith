//! Run every `expected-diagnostics.json` conformance case through the compiler
//! and check the produced diagnostics against it.
//!
//! The language-agnostic conformance runner in `xtask` does not execute these
//! cases yet (`docs/BACKLOG.md`); this test is the Rust compiler holding itself
//! to the same contract in the meantime. Asserted: `outcome` (no pack emitted),
//! and per diagnostic `code`, `severity`, and `address` as an unordered set.
//! Never asserted: `message`.

use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

use packsmith_compiler::{Graph, Severity, compile};

fn cases_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../conformance/cases")
}

/// `(code, severity, node, slot, index)` -- the fields a failure case asserts.
/// `code` is `None` when the expectation is `null` (a pending code); it then
/// matches any produced code.
type Key = (Option<String>, String, Option<String>, String, i64);

fn key_from_expected(d: &serde_json::Value) -> Key {
    let addr = &d["address"];
    (
        d["code"].as_str().map(str::to_string),
        d["severity"].as_str().unwrap_or_default().to_string(),
        addr["node"].as_str().map(str::to_string),
        addr["slot"].as_str().unwrap_or_default().to_string(),
        addr["index"].as_i64().unwrap_or_default(),
    )
}

#[test]
fn every_failure_case_produces_its_expected_diagnostics() {
    let mut checked = 0;

    for entry in fs::read_dir(cases_dir()).expect("read conformance/cases") {
        let dir = entry.expect("dir entry").path();
        let expected_path = dir.join("expected-diagnostics.json");
        if !expected_path.is_file() {
            continue;
        }
        let name = dir.file_name().unwrap().to_string_lossy().into_owned();

        let expected: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&expected_path).unwrap())
                .unwrap_or_else(|e| panic!("{name}: expected-diagnostics.json is not JSON: {e}"));
        assert_eq!(
            expected["outcome"], "compile-failure",
            "{name}: only compile-failure cases carry expected-diagnostics.json"
        );

        // A case whose expected diagnostics are all `code: null` is pending on a
        // stage the compiler does not have yet (command grammar, ADR-0012).
        // `.claude/rules/spec.md` treats `null` as satisfied until a real code
        // lands; this test skips the whole case the same way.
        let diagnostics = expected["diagnostics"]
            .as_array()
            .expect("diagnostics array");
        if diagnostics.iter().any(|d| d["code"].is_null()) {
            continue;
        }

        let graph: Graph =
            serde_json::from_str(&fs::read_to_string(dir.join("input.json")).unwrap())
                .unwrap_or_else(|e| panic!("{name}: input.json is not a graph: {e}"));
        let target: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(dir.join("target.json")).unwrap()).unwrap();
        let target_id = target["id"].as_str().expect("target id");

        let out = compile(&graph, target_id);

        assert!(
            out.diagnostics
                .iter()
                .any(|d| d.severity == Severity::Error),
            "{name}: expected a compile failure, got none"
        );
        assert!(
            out.ir.packs.iter().all(|p| p.resources.is_empty()),
            "{name}: a failed compile must emit no resources"
        );

        let want: BTreeSet<Key> = diagnostics.iter().map(key_from_expected).collect();

        let got_full: Vec<Key> = out
            .diagnostics
            .iter()
            .map(|d| {
                (
                    d.code.clone(),
                    match d.severity {
                        Severity::Error => "error".to_string(),
                        Severity::Warning => "warning".to_string(),
                    },
                    d.address.node.clone(),
                    d.address.slot.clone(),
                    i64::from(d.address.index),
                )
            })
            .collect();

        assert_eq!(
            got_full.len(),
            want.len(),
            "{name}: expected {} diagnostic(s), got {}: {got_full:?}",
            want.len(),
            got_full.len()
        );

        for produced in &got_full {
            let matched = want.iter().any(|w| {
                w.1 == produced.1
                    && w.2 == produced.2
                    && w.3 == produced.3
                    && w.4 == produced.4
                    && (w.0.is_none() || w.0 == produced.0)
            });
            assert!(
                matched,
                "{name}: unexpected diagnostic {produced:?}; wanted {want:?}"
            );
        }

        checked += 1;
    }

    assert!(
        checked >= 7,
        "expected at least 7 failure cases, checked {checked}"
    );
}
