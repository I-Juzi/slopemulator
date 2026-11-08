//! Remembers the last game and schema folders between starts.

use serde_json::{Value, json};
use std::path::PathBuf;

pub fn config_dir() -> Option<PathBuf> {
    let base = if cfg!(windows) {
        std::env::var_os("APPDATA").map(PathBuf::from)
    } else {
        std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
    };
    base.map(|b| b.join("bnsedit"))
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Settings {
    pub game: Option<PathBuf>,
    pub schema32: Option<PathBuf>,
    pub schema64: Option<PathBuf>,
}

impl Settings {
    pub fn load() -> Self {
        let Some(file) = config_dir().map(|d| d.join("settings.json")) else {
            return Self::default();
        };
        let Ok(v) = std::fs::read(file).map(|b| serde_json::from_slice::<Value>(&b)) else {
            return Self::default();
        };
        let Ok(v) = v else { return Self::default() };
        let get = |k: &str| v.get(k).and_then(Value::as_str).map(PathBuf::from);
        Self {
            game: get("game"),
            schema32: get("schema32"),
            schema64: get("schema64"),
        }
    }

    pub fn save(&self) {
        let Some(dir) = config_dir() else { return };
        let s = |p: &Option<PathBuf>| p.as_ref().map(|p| p.to_string_lossy().into_owned());
        let v = json!({ "game": s(&self.game), "schema32": s(&self.schema32), "schema64": s(&self.schema64) });
        let _ = std::fs::create_dir_all(&dir);
        let _ = std::fs::write(dir.join("settings.json"), v.to_string());
    }
}
