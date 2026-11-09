//! Parser for `clientdbg.ini` (simple `key=value` lines).

pub const DEFAULT_HOST: &str = "100.77.181.120";
pub const DEFAULT_PORT: u16 = 47100;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub allow_write: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            host: DEFAULT_HOST.to_string(),
            port: DEFAULT_PORT,
            allow_write: false,
        }
    }
}

/// Parse ini text. Missing or bad values keep the defaults.
/// Lines that start with `;`, `#` or `[` are ignored.
pub fn parse(text: &str) -> Config {
    let mut cfg = Config::default();
    for line in text.lines() {
        let line = line.trim().trim_start_matches('\u{feff}');
        if line.is_empty() || line.starts_with([';', '#', '[']) {
            continue;
        }
        let Some((k, v)) = line.split_once('=') else {
            continue;
        };
        let (k, v) = (k.trim().to_ascii_lowercase(), v.trim());
        match k.as_str() {
            "host" if !v.is_empty() => cfg.host = v.to_string(),
            "port" => {
                if let Ok(p) = v.parse::<u16>() {
                    cfg.port = p;
                }
            }
            "allow_write" => cfg.allow_write = v == "1" || v.eq_ignore_ascii_case("true"),
            _ => {}
        }
    }
    cfg
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ini_parse() {
        let c = parse("host=1.2.3.4\nport=47100\nallow_write=1");
        assert_eq!(c.host, "1.2.3.4");
        assert_eq!(c.port, 47100);
        assert!(c.allow_write);
        let d = parse("");
        assert_eq!(d, Config::default());
        assert!(!d.allow_write);
        let e = parse("; note\r\nport = 5\r\nhost=\r\n");
        assert_eq!(e.port, 5);
        assert_eq!(e.host, DEFAULT_HOST);
    }
}
