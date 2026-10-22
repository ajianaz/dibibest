//! `dibibest convert --to erdjson` works through the registry, no special-casing.

use std::io::Write;
use std::process::{Command, Stdio};

#[test]
fn convert_to_erdjson() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_dibibest"))
        .args(["convert", "--from", "dbml", "--to", "erdjson"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"Table users {\n  id int [pk]\n}\n")
        .unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success());
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["nodes"][0]["id"], "users");
    assert!(v["nodes"][0]["position"].is_null());
}
