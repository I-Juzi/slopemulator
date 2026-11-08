//! Run-time secrets for the `keyhere` placeholder.
//!
//! Configs in git hold `keyhere` for every key, password and certificate.
//! At start-up a service asks [`SecretStore`] for the real value. The store
//! looks in this order:
//!
//! 1. Environment variable `STS_SECRET_<NAME>`. `<NAME>` is the secret name in
//!    upper case; every character that is not a letter or digit becomes `_`.
//!    Example: `db.AcctDb.password` → `STS_SECRET_DB_ACCTDB_PASSWORD`.
//! 2. File `<dir>/<name>`. `<dir>` is `$STS_SECRETS_DIR`, else `secrets/`.
//!    Git ignores `secrets/` (see `secrets/README.md`).
//!
//! A value that is not `keyhere` is returned unchanged. Secret values are
//! never logged; errors name the secret, never its value.

use crate::SECRET_PLACEHOLDER;
use std::{
    collections::HashMap,
    fmt, fs, io,
    path::{Path, PathBuf},
};

/// Secret names used by the port. One key for one job.
pub mod names {
    /// SQL password for `[Sql.<Logical>]` in `Db.ini`.
    pub fn db_password(logical: &str) -> String {
        format!("db.{logical}.password")
    }
    /// `Route.ini [TLS] EncryptedKey` (TLS between servers; off on one host).
    pub const ROUTE_TLS_KEY: &str = "route.tls.key";
    /// Private key that signs game tokens. Its public half is in the patched
    /// client (`StsMsgCli64.dll`). ❓ Which original `Certificate/*` file this
    /// replaces is not traced yet.
    pub const TOKEN_SIGNING_KEY: &str = "auth.token_private.pem";
    /// CliGate TLS certificate and key (PEM). Not the token key.
    pub const CLIGATE_TLS: &str = "cligate.tls.pem";
    /// App secret: 5th column of `Game.ini [Configurations]`.
    pub fn app_secret(app: &str) -> String {
        format!("game.{app}.secret")
    }
}

#[derive(Debug)]
pub enum SecretError {
    /// The config says `keyhere` and no source has a value.
    Missing { name: String, env: String, path: PathBuf },
    /// The file exists but cannot be read.
    Io { name: String, path: PathBuf, source: io::Error },
    /// The value is not valid UTF-8 (text secrets only).
    NotText { name: String },
}

impl fmt::Display for SecretError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing { name, env, path } => write!(
                f,
                "secret '{name}' is {SECRET_PLACEHOLDER}: set {env} or create {}",
                path.display()
            ),
            Self::Io { name, path, source } => {
                write!(f, "secret '{name}': cannot read {}: {source}", path.display())
            }
            Self::NotText { name } => write!(f, "secret '{name}' is not UTF-8 text"),
        }
    }
}

impl std::error::Error for SecretError {}

/// Where secrets come from.
#[derive(Clone, Default)]
pub struct SecretStore {
    dir: PathBuf,
    env: HashMap<String, String>,
}

impl fmt::Debug for SecretStore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SecretStore")
            .field("dir", &self.dir)
            .field("env_names", &self.env.keys().collect::<Vec<_>>())
            .finish()
    }
}

/// Environment variable name for a secret name.
pub fn env_name(name: &str) -> String {
    let mut s = String::from("STS_SECRET_");
    s.extend(name.chars().map(|c| if c.is_ascii_alphanumeric() { c.to_ascii_uppercase() } else { '_' }));
    s
}

/// True if a config value is the placeholder.
pub fn is_placeholder(value: &str) -> bool {
    value.trim() == SECRET_PLACEHOLDER
}

impl SecretStore {
    /// Store from the process environment.
    pub fn from_env() -> Self {
        let dir = std::env::var_os("STS_SECRETS_DIR").map_or_else(|| PathBuf::from("secrets"), PathBuf::from);
        let env = std::env::vars().filter(|(k, _)| k.starts_with("STS_SECRET_")).collect();
        Self { dir, env }
    }

    /// Store with an explicit directory and variables (tests, tools).
    pub fn new(dir: impl Into<PathBuf>, env: impl IntoIterator<Item = (String, String)>) -> Self {
        Self { dir: dir.into(), env: env.into_iter().collect() }
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    fn path(&self, name: &str) -> PathBuf {
        self.dir.join(name)
    }

    /// Raw bytes of a secret (PEM files, key blobs).
    pub fn get_bytes(&self, name: &str) -> Result<Vec<u8>, SecretError> {
        let env = env_name(name);
        if let Some(v) = self.env.get(&env) {
            return Ok(v.clone().into_bytes());
        }
        let path = self.path(name);
        match fs::read(&path) {
            Ok(b) => {
                warn_if_open(name, &path);
                Ok(b)
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => Err(SecretError::Missing { name: name.into(), env, path }),
            Err(source) => Err(SecretError::Io { name: name.into(), path, source }),
        }
    }

    /// Text secret. One trailing line break from a file is removed.
    pub fn get_text(&self, name: &str) -> Result<String, SecretError> {
        let b = self.get_bytes(name)?;
        let mut s = String::from_utf8(b).map_err(|_| SecretError::NotText { name: name.into() })?;
        if s.ends_with('\n') {
            s.pop();
            if s.ends_with('\r') {
                s.pop();
            }
        }
        Ok(s)
    }

    /// Replace `keyhere` with the real secret. Any other value is kept.
    pub fn resolve(&self, name: &str, value: &str) -> Result<String, SecretError> {
        if is_placeholder(value) {
            self.get_text(name)
        } else {
            Ok(value.to_owned())
        }
    }
}

#[cfg(unix)]
fn warn_if_open(name: &str, path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    if let Ok(m) = fs::metadata(path) {
        if m.permissions().mode() & 0o077 != 0 {
            eprintln!("warning: secret '{name}' file {} is readable by others; use chmod 600", path.display());
        }
    }
}

#[cfg(not(unix))]
fn warn_if_open(_: &str, _: &Path) {}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmpdir(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("sts-secrets-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn env_names() {
        assert_eq!(env_name("db.AcctDb.password"), "STS_SECRET_DB_ACCTDB_PASSWORD");
        assert_eq!(env_name(names::TOKEN_SIGNING_KEY), "STS_SECRET_AUTH_TOKEN_PRIVATE_PEM");
    }

    #[test]
    fn non_placeholder_is_kept() {
        let s = SecretStore::new("/nonexistent", []);
        assert_eq!(s.resolve("db.X.password", "plain").unwrap(), "plain");
    }

    #[test]
    fn env_wins_over_file() {
        let d = tmpdir("env");
        fs::write(d.join("db.AcctDb.password"), "from-file\n").unwrap();
        let s = SecretStore::new(&d, [("STS_SECRET_DB_ACCTDB_PASSWORD".into(), "from-env".into())]);
        assert_eq!(s.resolve("db.AcctDb.password", "keyhere").unwrap(), "from-env");
        let s = SecretStore::new(&d, []);
        assert_eq!(s.resolve("db.AcctDb.password", " keyhere ").unwrap(), "from-file");
        fs::remove_dir_all(d).unwrap();
    }

    #[test]
    fn crlf_and_bytes() {
        let d = tmpdir("crlf");
        fs::write(d.join("a"), "v\r\n").unwrap();
        fs::write(d.join("b.pem"), "-----X-----\n").unwrap();
        let s = SecretStore::new(&d, []);
        assert_eq!(s.get_text("a").unwrap(), "v");
        assert_eq!(s.get_bytes("b.pem").unwrap(), b"-----X-----\n");
        fs::remove_dir_all(d).unwrap();
    }

    #[test]
    fn missing_names_the_secret_not_a_value() {
        let s = SecretStore::new("/nonexistent-dir", []);
        let e = s.resolve("route.tls.key", "keyhere").unwrap_err();
        let msg = e.to_string();
        assert!(msg.contains("route.tls.key") && msg.contains("STS_SECRET_ROUTE_TLS_KEY"), "{msg}");
        assert!(matches!(e, SecretError::Missing { .. }));
    }

    #[test]
    fn debug_hides_values() {
        let s = SecretStore::new("x", [("STS_SECRET_A".into(), "hidden-value".into())]);
        assert!(!format!("{s:?}").contains("hidden-value"));
    }
}
