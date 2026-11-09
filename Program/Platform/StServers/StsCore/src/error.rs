//! Platform error: `{code, server, module, line, text}`.
//!
//! Reply form: status 400, reason = error name, body
//! `<Error code= server= module= line= text=/>`. Zero fields and an empty
//! text are left out. A zero `server` is filled with the local server id
//! when the reply is built. Names come from `data/errcodes.tsv` (1538 distinct codes).

use crate::xml::Element;
use std::{collections::HashMap, fmt, sync::OnceLock};

pub const ERR_BAD_DATA: u32 = 4;
pub const ERR_BAD_PARAM: u32 = 10;
pub const ERR_TIMEOUT: u32 = 42;
pub const ERR_PERMISSION: u32 = 49;
pub const ERR_SQL_TIMEOUT: u32 = 900;
pub const ERR_BAD_CONFIG: u32 = 3030;

/// Module numbers seen in the binaries.
pub mod module {
    pub const CLI_GATE: u32 = 1;
    pub const ROUTE: u32 = 2;
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StsError {
    pub code: u32,
    pub server: u32,
    pub module: u32,
    pub line: u32,
    pub text: String,
    /// Child elements of `<Error>` (auxiliary data, for example the DbSrv
    /// `<Response>` of a failed procedure; sub_1400E7150).
    pub extra: Vec<Element>,
}

impl StsError {
    pub fn new(code: u32, module: u32, line: u32) -> Self {
        Self {
            code,
            module,
            line,
            ..Self::default()
        }
    }

    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self
    }

    pub fn name(&self) -> &'static str {
        error_name(self.code)
    }

    pub fn to_element(&self, local_server: u32) -> Element {
        let mut e = Element::new("Error");
        let server = if self.server == 0 {
            local_server
        } else {
            self.server
        };
        for (name, v) in [
            ("code", self.code),
            ("server", server),
            ("module", self.module),
            ("line", self.line),
        ] {
            if v != 0 {
                e = e.attr(name, (v as i32).to_string());
            }
        }
        if !self.text.is_empty() {
            e = e.attr("text", self.text.clone());
        }
        e.children.extend(self.extra.iter().cloned());
        e
    }

    pub fn with_extra(mut self, extra: Element) -> Self {
        self.extra.push(extra);
        self
    }

    pub fn from_element(e: &Element) -> Self {
        Self {
            code: e.attr_u32("code", 0),
            server: e.attr_u32("server", 0),
            module: e.attr_u32("module", 0),
            line: e.attr_u32("line", 0),
            text: e.attr_value("text").unwrap_or_default().to_owned(),
            extra: e.children.clone(),
        }
    }
}

impl fmt::Display for StsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}({}) server {} module {} line {}",
            self.name(),
            self.code,
            self.server,
            self.module,
            self.line
        )
    }
}
impl std::error::Error for StsError {}

fn table() -> &'static HashMap<u32, &'static str> {
    static T: OnceLock<HashMap<u32, &'static str>> = OnceLock::new();
    T.get_or_init(|| {
        include_str!("../data/errcodes.tsv")
            .lines()
            .filter_map(|l| {
                let (c, n) = l.split_once('\t')?;
                Some((c.trim().parse::<i64>().ok()? as u32, n.trim()))
            })
            .fold(HashMap::new(), |mut m, (c, n)| {
                m.entry(c).or_insert(n); // first name wins
                m
            })
    })
}

/// Name of an error code; unknown codes give `"ErrUnknown"` ❓ (fallback text
/// not recovered from the binary).
pub fn error_name(code: u32) -> &'static str {
    table().get(&code).copied().unwrap_or("ErrUnknown")
}

pub fn error_code(name: &str) -> Option<u32> {
    table().iter().find(|(_, n)| **n == name).map(|(c, _)| *c)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names() {
        assert_eq!(error_name(4), "ErrBadData");
        assert_eq!(error_code("ErrBadParam"), Some(10));
        assert_eq!(table().len(), 1538); // 1550 rows, 12 duplicated in the binary table
    }

    #[test]
    fn body_leaves_out_zero() {
        let e = StsError::new(ERR_BAD_DATA, 0, 0);
        assert_eq!(e.to_element(0).to_xml(), "<Error code=\"4\"/>\n");
        let e = StsError::new(ERR_BAD_PARAM, module::ROUTE, 1115);
        assert_eq!(
            e.to_element(3).to_xml(),
            "<Error code=\"10\" server=\"3\" module=\"2\" line=\"1115\"/>\n"
        );
        assert_eq!(StsError::from_element(&e.to_element(3)).server, 3);
    }
}
