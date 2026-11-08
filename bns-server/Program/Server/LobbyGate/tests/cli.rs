use std::process::Command;
#[test]
fn help_has_no_key_requirement() {
    let output = Command::new(env!("CARGO_BIN_EXE_lobbygate"))
        .arg("--help")
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8(output.stdout)
        .unwrap()
        .contains("--handshake-key FILE"));
}
#[test]
fn no_key_refuses_startup() {
    let output = Command::new(env!("CARGO_BIN_EXE_lobbygate"))
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8(output.stderr)
        .unwrap()
        .contains("--handshake-key is required"));
}
#[test]
fn unknown_option_refuses_startup() {
    let output = Command::new(env!("CARGO_BIN_EXE_lobbygate"))
        .arg("--unknown")
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8(output.stderr)
        .unwrap()
        .contains("unknown or duplicate option"));
}
