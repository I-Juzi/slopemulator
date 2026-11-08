//! `<Request>` body of `Db.<Logical>/Exec` (DbSrv sub_1400D3B90).
//!
//! ```xml
//! <Request><SchemaName>v2016102001</SchemaName><ProcName>GetAccount</ProcName>
//!   <Params><S>text</S><I name="@Id">5</I><S type="null"/><T><R><C>a</C></R></T></Params>
//! </Request>
//! ```
//! Only the element name `T` is special. Every other param is text: `varchar`
//! when all bytes are ASCII, else `nvarchar`.

use sts_core::{error::ERR_BAD_PARAM, xml::Element, StsError};

/// Module number of the execution errors (`*(a1+1376) = 2`, sub_1400D2B80).
pub const MODULE_EXEC: u32 = 2;
/// Module number of the connection-handler errors (DbConn, sub_1400D2860).
pub const MODULE_CONN: u32 = 1;
pub const MAX_PARAMS: usize = 32;
/// varchar text longer than this is bound with column size 0 (max).
pub const VARCHAR_MAX_INLINE: usize = 8000;
/// nvarchar text longer than this (UTF-16 units) is bound with column size 0.
pub const NVARCHAR_MAX_INLINE: usize = 4000;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value {
    Null,
    /// SQL_VARCHAR (12): all bytes < 0x80.
    VarChar(String),
    /// SQL_WVARCHAR (−9): text with a byte ≥ 0x80.
    NVarChar(String),
    /// SQL_SS_TABLE (−153): rows of cells; `None` = NULL cell.
    Table(Vec<Vec<Option<String>>>),
}

impl Value {
    pub fn text(s: &str) -> Self {
        if s.is_ascii() {
            Value::VarChar(s.to_owned())
        } else {
            Value::NVarChar(s.to_owned())
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Param {
    pub name: Option<String>,
    pub value: Value,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecRequest {
    pub schema: String,
    /// Full procedure name with the `p_` prefix.
    pub proc_name: String,
    pub params: Vec<Param>,
}

fn bad(line: u32) -> StsError {
    StsError::new(ERR_BAD_PARAM, MODULE_EXEC, line)
}

/// `[A-Za-z0-9]*`, shorter than 128 characters.
fn valid_ident(s: &str) -> bool {
    s.len() < 128 && s.bytes().all(|b| b.is_ascii_alphanumeric())
}

fn is_null(e: &Element) -> bool {
    e.attr_value("type") == Some("null")
}

impl ExecRequest {
    pub fn parse(body: &Element) -> Result<Self, StsError> {
        let schema = body.get("SchemaName").map_or("dbo", |e| e.text.as_str());
        if !valid_ident(schema) {
            return Err(bad(621));
        }
        let proc = body.get("ProcName").map_or("unspecified", |e| e.text.as_str());
        if !valid_ident(proc) {
            return Err(bad(632));
        }
        let mut req = ExecRequest { schema: schema.to_owned(), proc_name: format!("p_{proc}"), params: vec![] };
        let Some(params) = body.get("Params") else { return Ok(req) };
        let mut named = 0usize;
        for p in &params.children {
            if req.params.len() >= MAX_PARAMS {
                return Err(bad(658));
            }
            let name = p.attr_value("name").map(str::to_owned);
            named += name.is_some() as usize;
            let value = if p.name == "T" {
                let mut rows = Vec::new();
                let mut cols = None;
                for row in &p.children {
                    if *cols.get_or_insert(row.children.len()) != row.children.len() {
                        return Err(bad(738));
                    }
                    rows.push(row.children.iter().map(|c| (!is_null(c)).then(|| c.text.clone())).collect());
                }
                Value::Table(rows)
            } else if is_null(p) {
                Value::Null
            } else {
                Value::text(&p.text)
            };
            req.params.push(Param { name, value });
        }
        // Either all params are named or none (sub_1400D3B90 line 828).
        if named != 0 && named != req.params.len() {
            return Err(bad(828));
        }
        Ok(req)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(xml: &str) -> Result<ExecRequest, StsError> {
        ExecRequest::parse(&Element::parse(xml.as_bytes()).unwrap())
    }

    #[test]
    fn parses_params() {
        let r = parse("<Request><SchemaName>v1</SchemaName><ProcName>Get</ProcName><Params><I>5</I><S type=\"null\"/><S>\u{d55c}</S><T><R><C>a</C><C type=\"null\"/></R></T></Params></Request>").unwrap();
        assert_eq!(r.schema, "v1");
        assert_eq!(r.proc_name, "p_Get");
        assert_eq!(r.params[0].value, Value::VarChar("5".into()));
        assert_eq!(r.params[1].value, Value::Null);
        assert_eq!(r.params[2].value, Value::NVarChar("\u{d55c}".into()));
        assert_eq!(r.params[3].value, Value::Table(vec![vec![Some("a".into()), None]]));
    }

    #[test]
    fn defaults_and_errors() {
        let r = parse("<Request/>").unwrap();
        assert_eq!((r.schema.as_str(), r.proc_name.as_str()), ("dbo", "p_unspecified"));
        assert_eq!(parse("<Request><SchemaName>a.b</SchemaName></Request>").unwrap_err().line, 621);
        assert_eq!(parse("<Request><ProcName>x;drop</ProcName></Request>").unwrap_err().line, 632);
        let many = format!("<Request><Params>{}</Params></Request>", "<S>1</S>".repeat(33));
        assert_eq!(parse(&many).unwrap_err().line, 658);
        assert_eq!(parse("<Request><Params><T><R><C/></R><R/></T></Params></Request>").unwrap_err().line, 738);
        let e = parse("<Request><Params><S name=\"@a\">1</S><S>2</S></Params></Request>").unwrap_err();
        assert_eq!((e.code, e.module, e.line), (10, 2, 828));
    }
}
