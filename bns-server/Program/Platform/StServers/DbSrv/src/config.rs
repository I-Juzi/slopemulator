//! `Db.ini` (DbSrv sub_1400D3550, sub_1400D4370 area).
//!
//! `[Sql.<Logical>]`: Database (default: computer name), Driver (default
//! `SQL Server Native Client 10.0`), Address (`ip`, `ip:port`, `host/instance`),
//! Mirror, Username, Password or EncryptedPassword, TransTimeoutSeconds
//! (default 20, 0 = forever). A value with `;` is logged and cleared.
//! `[ThreadTasks] ThreadTasks` = worker count. `[DbSrvOptions]` ServerKeyBits
//! (2048), ServerPrName (`plaync.pr`) for EncryptedPassword.
//!
//! Secrets: the shipped config has `Password=keyhere`. `main` replaces it with
//! secret `db.<Logical>.password` (`sts_core::secrets`). EncryptedPassword
//! (Base64, RSA with the server `.pr` key) is not supported by the port.

use sts_core::ini::{Ini, Section};

pub const DEFAULT_DRIVER: &str = "SQL Server Native Client 10.0";

#[derive(Clone, PartialEq, Eq)]
pub struct SqlConfig {
    pub logical: String,
    pub database: String,
    pub driver: String,
    pub host: String,
    pub port: Option<u16>,
    pub instance: Option<String>,
    pub mirror: String,
    pub username: String,
    pub password: String,
    pub trans_timeout_secs: u32,
}

fn value(sec: &Section, key: &str, default: &str) -> String {
    let v = sec.str_or(key, default);
    if v.contains(';') {
        eprintln!("DbTrans: semicolons not allowed in {key} string");
        return String::new();
    }
    v.chars().take(63).collect()
}

fn computer_name() -> String {
    std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| {
            std::fs::read_to_string("/etc/hostname")
                .unwrap_or_default()
                .trim()
                .to_owned()
        })
}

/// Debug output hides the password.
impl std::fmt::Debug for SqlConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SqlConfig")
            .field("logical", &self.logical)
            .field("database", &self.database)
            .field("host", &self.host)
            .field("port", &self.port)
            .field("instance", &self.instance)
            .field("username", &self.username)
            .field(
                "password",
                &if self.password.is_empty() { "" } else { "***" },
            )
            .field("trans_timeout_secs", &self.trans_timeout_secs)
            .finish_non_exhaustive()
    }
}

impl SqlConfig {
    pub fn from_ini(ini: &Ini, logical: &str) -> Result<Self, String> {
        let name = format!("Sql.{logical}");
        let sec = ini
            .section(&name)
            .ok_or_else(|| format!("DbTrans: missing configuration section '{name}'"))?;
        let address = value(sec, "Address", "");
        // `host:port` → `host,port` in the ODBC string; `host/instance` is a named instance.
        let (hostpart, instance) = match address.split_once(['/', '\\']) {
            Some((h, i)) => (h.to_owned(), Some(i.to_owned())),
            None => (address.clone(), None),
        };
        let (host, port) = match hostpart.rsplit_once(':') {
            Some((h, p)) => (h.to_owned(), p.parse().ok()),
            None => (hostpart, None),
        };
        let password = if !value(sec, "EncryptedPassword", "").is_empty() {
            eprintln!("DbTrans: EncryptedPassword is not supported by the port; use Password");
            String::new()
        } else {
            value(sec, "Password", "")
        };
        Ok(Self {
            logical: logical.to_owned(),
            database: value(sec, "Database", &computer_name()),
            driver: value(sec, "Driver", DEFAULT_DRIVER),
            host: if host.is_empty() {
                "localhost".into()
            } else {
                host
            },
            port,
            instance,
            mirror: value(sec, "Mirror", ""),
            username: value(sec, "Username", ""),
            password,
            trans_timeout_secs: sec.int_or("TransTimeoutSeconds", 20).max(0) as u32,
        })
    }

    pub fn tiberius(&self) -> tiberius::Config {
        let mut c = tiberius::Config::new();
        c.host(&self.host);
        if let Some(p) = self.port {
            c.port(p);
        }
        if let Some(i) = &self.instance {
            c.instance_name(i);
        }
        c.database(&self.database);
        c.application_name("DbSrv");
        if !self.username.is_empty() && !self.password.is_empty() {
            c.authentication(tiberius::AuthMethod::sql_server(
                &self.username,
                &self.password,
            ));
        }
        c.trust_cert();
        c
    }
}

pub fn thread_tasks(ini: &Ini) -> usize {
    match ini.int_or("ThreadTasks", "ThreadTasks", 0) {
        n if n > 0 => n as usize,
        _ => std::thread::available_parallelism().map_or(4, |n| n.get()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_section() {
        let ini = Ini::parse("[Sql.AcctDb]\nDatabase=PlatformAcctDb\nAddress=127.0.0.1:1433\nUsername=sa\nPassword=keyhere\n[Sql.X]\nDatabase=a;b\nAddress=SQLHOST/INST\n");
        let c = SqlConfig::from_ini(&ini, "AcctDb").unwrap();
        assert_eq!(
            (c.host.as_str(), c.port, c.trans_timeout_secs),
            ("127.0.0.1", Some(1433), 20)
        );
        assert_eq!(c.password, sts_core::SECRET_PLACEHOLDER);
        let x = SqlConfig::from_ini(&ini, "X").unwrap();
        assert_eq!(
            (x.database.as_str(), x.instance.as_deref()),
            ("", Some("INST"))
        );
        assert!(SqlConfig::from_ini(&ini, "None").is_err());
    }
}
