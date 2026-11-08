use lmxb::{Document, Element};
use std::{fs, process::Command};

#[test]
fn dump_generated_xml_and_reject_bad_input() {
    let dir = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("lmxb-cli-tests")
        .join(std::process::id().to_string());
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("generated.lmxb");
    let doc = Document {
        reserved: [0; 64],
        source_flag: 1,
        source: "generated.xml".into(),
        root: Element {
            name_flag: 1,
            name: "generated".into(),
            attributes: vec![("id".into(), "123".into())],
            source_line: 1,
            children: vec![],
        },
    };
    fs::write(&path, lmxb::encode(&doc)).unwrap();
    let bin = env!("CARGO_BIN_EXE_lmxb-dump");
    let output = Command::new(bin).arg(&path).output().unwrap();
    assert!(output.status.success());
    assert_eq!(output.stdout, b"<generated id=\"123\"/>");
    assert!(output.stderr.is_empty());
    fs::write(&path, b"garbled generated data").unwrap();
    let output = Command::new(bin).arg(&path).output().unwrap();
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("bad magic"));
    let output = Command::new(bin).output().unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("usage:"));
    let output = Command::new(bin).args(["one", "two"]).output().unwrap();
    assert!(!output.status.success());
    // This directory contains only this test's generated fixture.
    fs::remove_file(path).unwrap();
    fs::remove_dir(dir).unwrap();
}
