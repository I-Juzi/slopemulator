//! STS XML documents.
//!
//! The writer matches the platform builder (RapidXml `IXBuilder`, no indent):
//! one element per line, `<X/>` for an element without content, text escaped
//! as `&amp;` `&lt;`, attribute values also as `&quot;`. Integers use `%d`.

use std::fmt::Write as _;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Element {
    pub name: String,
    pub attrs: Vec<(String, String)>,
    pub text: String,
    pub children: Vec<Element>,
}

impl Element {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into(), ..Self::default() }
    }

    /// `<Name>text</Name>`; an empty text gives `<Name/>`.
    pub fn text(name: impl Into<String>, text: impl Into<String>) -> Self {
        Self { name: name.into(), text: text.into(), ..Self::default() }
    }

    pub fn int(name: impl Into<String>, value: i64) -> Self {
        Self::text(name, value.to_string())
    }

    /// `<Name type="array">` container.
    pub fn array(name: impl Into<String>) -> Self {
        Self::new(name).attr("type", "array")
    }

    pub fn attr(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.attrs.push((name.into(), value.into()));
        self
    }

    pub fn child(mut self, child: Element) -> Self {
        self.children.push(child);
        self
    }

    pub fn push(&mut self, child: Element) -> &mut Self {
        self.children.push(child);
        self
    }

    /// Adds `<name>value</name>` only when `value != 0`.
    pub fn push_nonzero(&mut self, name: &str, value: u32) -> &mut Self {
        if value != 0 {
            self.children.push(Element::int(name, value as i32 as i64));
        }
        self
    }

    pub fn push_text_nonempty(&mut self, name: &str, value: &str) -> &mut Self {
        if !value.is_empty() {
            self.children.push(Element::text(name, value));
        }
        self
    }

    pub fn get(&self, name: &str) -> Option<&Element> {
        self.children.iter().find(|c| c.name == name)
    }

    pub fn all<'a>(&'a self, name: &'a str) -> impl Iterator<Item = &'a Element> + 'a {
        self.children.iter().filter(move |c| c.name == name)
    }

    pub fn get_text(&self, name: &str) -> Option<&str> {
        self.get(name).map(|e| e.text.as_str())
    }

    /// Reads a child as an integer. Missing or bad → `default`.
    /// Accepts signed text, as the platform writes with `%d`.
    pub fn get_u32(&self, name: &str, default: u32) -> u32 {
        self.get_text(name).and_then(parse_u32).unwrap_or(default)
    }

    pub fn attr_value(&self, name: &str) -> Option<&str> {
        self.attrs.iter().find(|(n, _)| n == name).map(|(_, v)| v.as_str())
    }

    pub fn attr_u32(&self, name: &str, default: u32) -> u32 {
        self.attr_value(name).and_then(parse_u32).unwrap_or(default)
    }

    pub fn has(&self, name: &str) -> bool {
        self.get(name).is_some()
    }

    pub fn to_xml(&self) -> String {
        let mut out = String::new();
        self.write(&mut out);
        out
    }

    pub fn write(&self, out: &mut String) {
        out.push('<');
        out.push_str(&self.name);
        for (name, value) in &self.attrs {
            let _ = write!(out, " {name}=\"");
            escape(out, value, true);
            out.push('"');
        }
        if self.text.is_empty() && self.children.is_empty() {
            out.push_str("/>\n");
            return;
        }
        if !self.text.is_empty() {
            out.push('>');
            escape(out, &self.text, false);
            if !self.children.is_empty() {
                out.push('\n');
            }
        } else {
            out.push_str(">\n");
        }
        for child in &self.children {
            child.write(out);
        }
        let _ = writeln!(out, "</{}>", self.name);
    }

    /// Parses one document. Whitespace-only text is dropped.
    pub fn parse(input: &[u8]) -> Result<Element, ParseError> {
        use quick_xml::events::Event;
        let mut reader = quick_xml::Reader::from_reader(input);
        let mut stack: Vec<Element> = Vec::new();
        let mut root = None;
        loop {
            match reader.read_event().map_err(|_| ParseError)? {
                Event::Start(e) => stack.push(start(&e)?),
                Event::Empty(e) => {
                    let el = start(&e)?;
                    close(&mut stack, &mut root, el)?;
                }
                Event::End(_) => {
                    let el = stack.pop().ok_or(ParseError)?;
                    close(&mut stack, &mut root, el)?;
                }
                Event::Text(t) => {
                    let s = t.decode().map_err(|_| ParseError)?;
                    let s = quick_xml::escape::unescape(&s).map_err(|_| ParseError)?;
                    if let Some(top) = stack.last_mut() {
                        if !s.trim().is_empty() {
                            top.text.push_str(&s);
                        }
                    }
                }
                Event::CData(t) => {
                    if let Some(top) = stack.last_mut() {
                        top.text.push_str(std::str::from_utf8(&t).map_err(|_| ParseError)?);
                    }
                }
                Event::GeneralRef(r) => {
                    let name = std::str::from_utf8(&r).map_err(|_| ParseError)?;
                    let s = quick_xml::escape::unescape(&format!("&{name};"))
                        .map_err(|_| ParseError)?
                        .into_owned();
                    if let Some(top) = stack.last_mut() {
                        top.text.push_str(&s);
                    }
                }
                Event::Eof => break,
                _ => {}
            }
        }
        if !stack.is_empty() {
            return Err(ParseError);
        }
        root.ok_or(ParseError)
    }
}

fn start(e: &quick_xml::events::BytesStart<'_>) -> Result<Element, ParseError> {
    let name = std::str::from_utf8(e.name().as_ref()).map_err(|_| ParseError)?.to_owned();
    let mut el = Element::new(name);
    for a in e.attributes() {
        let a = a.map_err(|_| ParseError)?;
        let key = std::str::from_utf8(a.key.as_ref()).map_err(|_| ParseError)?.to_owned();
        let value = a.unescape_value().map_err(|_| ParseError)?.into_owned();
        el.attrs.push((key, value));
    }
    Ok(el)
}

fn close(stack: &mut [Element], root: &mut Option<Element>, el: Element) -> Result<(), ParseError> {
    match stack.last_mut() {
        Some(parent) => parent.children.push(el),
        None if root.is_none() => *root = Some(el),
        None => return Err(ParseError),
    }
    Ok(())
}

fn escape(out: &mut String, s: &str, attr: bool) {
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '"' if attr => out.push_str("&quot;"),
            _ => out.push(c),
        }
    }
}

pub fn parse_u32(s: &str) -> Option<u32> {
    let s = s.trim();
    if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        return u32::from_str_radix(hex, 16).ok();
    }
    if let Ok(v) = s.parse::<u32>() {
        return Some(v);
    }
    s.parse::<i32>().ok().map(|v| v as u32)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParseError;

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("bad XML")
    }
}
impl std::error::Error for ParseError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writer_layout_matches_builder() {
        let mut reply = Element::new("Reply");
        reply.push(Element::int("UserId", 5)).push(Element::text("Empty", ""));
        reply.push(Element::new("Error").attr("code", "4").attr("text", "a\"<&"));
        assert_eq!(
            reply.to_xml(),
            "<Reply>\n<UserId>5</UserId>\n<Empty/>\n<Error code=\"4\" text=\"a&quot;&lt;&amp;\"/>\n</Reply>\n"
        );
    }

    #[test]
    fn roundtrip() {
        let el = Element::array("Providers")
            .child(Element::new("Provider").attr("code", "7"))
            .child(Element::text("Name", "a&b<c"));
        assert_eq!(Element::parse(el.to_xml().as_bytes()).unwrap(), el);
    }

    #[test]
    fn signed_ints() {
        assert_eq!(parse_u32("-1"), Some(u32::MAX));
        assert_eq!(parse_u32("0x10"), Some(16));
    }
}
