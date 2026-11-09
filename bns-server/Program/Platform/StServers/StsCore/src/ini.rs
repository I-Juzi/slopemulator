//! Config INI files of the StServers (`Db.ini`, `Route.ini`, …).
//!
//! Format: `[Section]`, `Key=Value`, `;` comment lines. Files are ASCII or
//! UTF-16LE with a BOM. Section and key names compare without case ❓ (the
//! original reader was not traced in full). Values are trimmed.

use std::{io, path::Path};

#[derive(Clone, Debug, Default)]
pub struct Section {
    pub name: String,
    pub entries: Vec<(String, String)>,
}

impl Section {
    pub fn get(&self, key: &str) -> Option<&str> {
        self.entries
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(key))
            .map(|(_, v)| v.as_str())
    }

    /// Value, or `default` when the key is missing.
    pub fn str_or<'a>(&'a self, key: &str, default: &'a str) -> &'a str {
        self.get(key).unwrap_or(default)
    }

    /// Integer value (decimal or `0x` hex), or `default` when missing or bad.
    pub fn int_or(&self, key: &str, default: i64) -> i64 {
        self.get(key).and_then(parse_int).unwrap_or(default)
    }
}

#[derive(Clone, Debug, Default)]
pub struct Ini {
    pub sections: Vec<Section>,
}

fn parse_int(s: &str) -> Option<i64> {
    let s = s.trim();
    match s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        Some(h) => i64::from_str_radix(h, 16).ok(),
        None => s.parse().ok(),
    }
}

impl Ini {
    pub fn load(path: impl AsRef<Path>) -> io::Result<Self> {
        Ok(Self::parse_bytes(&std::fs::read(path)?))
    }

    pub fn parse_bytes(bytes: &[u8]) -> Self {
        let text = match bytes {
            [0xFF, 0xFE, rest @ ..] => {
                let units: Vec<u16> = rest
                    .as_chunks::<2>()
                    .0
                    .iter()
                    .map(|c| u16::from_le_bytes(*c))
                    .collect();
                String::from_utf16_lossy(&units)
            }
            [0xEF, 0xBB, 0xBF, rest @ ..] => String::from_utf8_lossy(rest).into_owned(),
            _ => String::from_utf8_lossy(bytes).into_owned(),
        };
        Self::parse(&text)
    }

    pub fn parse(text: &str) -> Self {
        let mut ini = Ini::default();
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with(';') {
                continue;
            }
            if let Some(name) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
                ini.sections.push(Section {
                    name: name.trim().to_owned(),
                    entries: vec![],
                });
            } else if let (Some((k, v)), Some(sec)) =
                (line.split_once('='), ini.sections.last_mut())
            {
                sec.entries.push((k.trim().to_owned(), v.trim().to_owned()));
            }
        }
        ini
    }

    pub fn section(&self, name: &str) -> Option<&Section> {
        self.sections
            .iter()
            .find(|s| s.name.eq_ignore_ascii_case(name))
    }

    pub fn get(&self, section: &str, key: &str) -> Option<&str> {
        self.section(section).and_then(|s| s.get(key))
    }

    pub fn int_or(&self, section: &str, key: &str, default: i64) -> i64 {
        self.section(section)
            .map_or(default, |s| s.int_or(key, default))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_sections_and_utf16() {
        let text = ";c\r\n[Sql.AcctDb]\r\n    Database=PlatformAcctDb\r\n  Password=keyhere\r\n[ThreadTasks]\r\nThreadTasks=0x32\r\n";
        let ini = Ini::parse(text);
        assert_eq!(ini.get("sql.acctdb", "database"), Some("PlatformAcctDb"));
        assert_eq!(ini.int_or("ThreadTasks", "ThreadTasks", 0), 50);
        let mut wide = vec![0xFF, 0xFE];
        wide.extend(text.encode_utf16().flat_map(|u| u.to_le_bytes()));
        assert_eq!(
            Ini::parse_bytes(&wide).get("Sql.AcctDb", "Password"),
            Some("keyhere")
        );
    }
}
