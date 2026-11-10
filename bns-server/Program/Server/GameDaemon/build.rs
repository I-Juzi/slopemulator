//! Build script. It writes one `pub const <Name>: u16` for each `GameCmd` variant.
//!
//! The source is the generated enum file of `packet-ids`. The constants let `match` arms and
//! ID lists use official names. The value of each constant comes from `GameCmd::<Name>.id()`.

use std::{env, fs, path::PathBuf};

fn main() {
    let source = "../PacketIds/src/generated/commands.rs";
    println!("cargo:rerun-if-changed={source}");
    let text = fs::read_to_string(source).expect("read packet-ids commands.rs");
    let mut out = String::new();
    let mut inside = false;
    for line in text.lines() {
        if line.starts_with("pub enum GameCmd ") {
            inside = true;
            continue;
        }
        if !inside {
            continue;
        }
        if line.starts_with('}') {
            break;
        }
        let trimmed = line.trim();
        if trimmed.starts_with("//") || trimmed.starts_with('#') {
            continue;
        }
        if let Some((name, _)) = trimmed.split_once(" = ") {
            out.push_str(&format!(
                "#[allow(non_upper_case_globals, dead_code)]\npub const {name}: u16 = GameCmd::{name}.id() as u16;\n"
            ));
        }
    }
    let path = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR")).join("game_cmd_ids.rs");
    fs::write(path, out).expect("write game_cmd_ids.rs");
}
