//! Byte-derived LMXBOSLB v3 codec. See `spec/gamed/01-lmxboslb.md`.
//!
//! The root kind is implicit. Metadata, attribute order, and text whitespace
//! are retained, so decoding and encoding a supported file is byte-exact.
use std::fmt;

pub const MAGIC: &[u8; 8] = b"LMXBOSLB";
pub const MAX_DEPTH: usize = 128;
const KEY: [u8; 16] = [
    0xa4, 0x9f, 0xd8, 0xb3, 0xf6, 0x8e, 0x39, 0xc2, 0x2d, 0xe0, 0x61, 0x75, 0x5c, 0x4b, 0x1a, 0x07,
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Document {
    pub reserved: [u8; 64],
    pub source_flag: u8,
    pub source: String,
    pub root: Element,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Element {
    pub name_flag: u8,
    pub name: String,
    pub attributes: Vec<(String, String)>,
    pub source_line: u32,
    pub children: Vec<Node>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Text {
    pub name_flag: u8,
    pub name: String,
    pub source_line: u32,
    pub value: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Node {
    Element(Element),
    Text(Text),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    /// Byte offset for decode errors; zero for XML rendering errors.
    pub offset: usize,
    pub message: &'static str,
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at byte 0x{:x}", self.message, self.offset)
    }
}
impl std::error::Error for Error {}

struct Reader<'a> {
    bytes: &'a [u8],
    pos: usize,
}
impl<'a> Reader<'a> {
    fn error(&self, message: &'static str) -> Error {
        Error {
            offset: self.pos,
            message,
        }
    }
    fn take(&mut self, n: usize) -> Result<&'a [u8], Error> {
        let end = self
            .pos
            .checked_add(n)
            .ok_or_else(|| self.error("length overflow"))?;
        let bytes = self
            .bytes
            .get(self.pos..end)
            .ok_or_else(|| self.error("truncated input"))?;
        self.pos = end;
        Ok(bytes)
    }
    fn byte(&mut self) -> Result<u8, Error> {
        Ok(self.take(1)?[0])
    }
    fn u32(&mut self) -> Result<u32, Error> {
        let b = self.take(4)?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }
    fn string(&mut self) -> Result<String, Error> {
        let units = self.u32()? as usize;
        let len = units
            .checked_mul(2)
            .ok_or_else(|| self.error("string length overflow"))?;
        let raw = self.take(len)?;
        let decoded = raw.as_chunks::<2>().0.iter().enumerate().map(|(i, b)| {
            u16::from_le_bytes([b[0] ^ KEY[(2 * i) % 16], b[1] ^ KEY[(2 * i + 1) % 16]])
        });
        char::decode_utf16(decoded)
            .collect::<Result<String, _>>()
            .map_err(|_| self.error("invalid UTF-16"))
    }
    fn count(&mut self, minimum_bytes: usize) -> Result<u32, Error> {
        let count = self.u32()?;
        if count as usize > (self.bytes.len() - self.pos) / minimum_bytes {
            return Err(self.error("count exceeds remaining input"));
        }
        Ok(count)
    }
    fn element(&mut self) -> Result<(Element, u32), Error> {
        let count = self.count(8)?;
        let mut attributes = Vec::new();
        for _ in 0..count {
            attributes.push((self.string()?, self.string()?));
        }
        let name_flag = self.byte()?;
        let name = self.string()?;
        let children = self.count(21)?;
        let source_line = self.u32()?;
        Ok((
            Element {
                name_flag,
                name,
                attributes,
                source_line,
                children: Vec::new(),
            },
            children,
        ))
    }
    fn text(&mut self) -> Result<Text, Error> {
        let value = self.string()?;
        let name_flag = self.byte()?;
        let name = self.string()?;
        if self.u32()? != 0 {
            return Err(self.error("text node has children"));
        }
        let source_line = self.u32()?;
        Ok(Text {
            name_flag,
            name,
            source_line,
            value,
        })
    }
}

/// Decode v3. Reject malformed framing, invalid UTF-16, trailing bytes, and
/// nesting deeper than [`MAX_DEPTH`]. No allocation uses an unchecked wire count.
pub fn decode(bytes: &[u8]) -> Result<Document, Error> {
    let mut r = Reader { bytes, pos: 0 };
    if r.take(8)? != MAGIC {
        return Err(r.error("bad magic"));
    }
    if r.u32()? != 3 {
        return Err(r.error("unsupported version"));
    }
    if r.u32()? as usize != bytes.len() {
        return Err(r.error("file size mismatch"));
    }
    let mut reserved = [0; 64];
    reserved.copy_from_slice(r.take(64)?);
    let source_flag = r.byte()?;
    let source = r.string()?;
    let root = r.element()?;
    let mut stack = vec![root];
    loop {
        let Some((_, remaining)) = stack.last_mut() else {
            return Err(r.error("missing root"));
        };
        if *remaining == 0 {
            let Some((element, _)) = stack.pop() else {
                return Err(r.error("missing element"));
            };
            if let Some((parent, _)) = stack.last_mut() {
                parent.children.push(Node::Element(element));
            } else {
                if r.pos != bytes.len() {
                    return Err(r.error("trailing bytes"));
                }
                return Ok(Document {
                    reserved,
                    source_flag,
                    source,
                    root: element,
                });
            }
        } else {
            *remaining -= 1;
            if stack.len() >= MAX_DEPTH {
                return Err(r.error("node depth limit"));
            }
            match r.u32()? {
                1 => stack.push(r.element()?),
                2 => {
                    let text = r.text()?;
                    if let Some((parent, _)) = stack.last_mut() {
                        parent.children.push(Node::Text(text));
                    }
                }
                _ => return Err(r.error("unsupported node kind")),
            }
        }
    }
}

fn put_u32(out: &mut Vec<u8>, n: u32) {
    out.extend_from_slice(&n.to_le_bytes());
}
fn put_string(out: &mut Vec<u8>, value: &str) {
    put_u32(out, value.encode_utf16().count() as u32);
    for (i, unit) in value.encode_utf16().enumerate() {
        let b = unit.to_le_bytes();
        out.push(b[0] ^ KEY[(2 * i) % 16]);
        out.push(b[1] ^ KEY[(2 * i + 1) % 16]);
    }
}
fn put_element(out: &mut Vec<u8>, element: &Element) {
    put_u32(out, element.attributes.len() as u32);
    for (key, value) in &element.attributes {
        put_string(out, key);
        put_string(out, value);
    }
    out.push(element.name_flag);
    put_string(out, &element.name);
    put_u32(out, element.children.len() as u32);
    put_u32(out, element.source_line);
}

/// Encode v3 in wire order. A document must fit the format's u32 file size.
/// Source and name flags are opaque; strings always use the observed XOR form.
pub fn encode(document: &Document) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(MAGIC);
    put_u32(&mut out, 3);
    put_u32(&mut out, 0);
    out.extend_from_slice(&document.reserved);
    out.push(document.source_flag);
    put_string(&mut out, &document.source);
    put_element(&mut out, &document.root);
    let mut stack = vec![document.root.children.iter()];
    while let Some(children) = stack.last_mut() {
        match children.next() {
            Some(Node::Element(element)) => {
                put_u32(&mut out, 1);
                put_element(&mut out, element);
                stack.push(element.children.iter());
            }
            Some(Node::Text(text)) => {
                put_u32(&mut out, 2);
                put_string(&mut out, &text.value);
                out.push(text.name_flag);
                put_string(&mut out, &text.name);
                put_u32(&mut out, 0);
                put_u32(&mut out, text.source_line);
            }
            None => {
                stack.pop();
            }
        }
    }
    let len = out.len() as u32;
    out[12..16].copy_from_slice(&len.to_le_bytes());
    out
}

fn name_start(c: char) -> bool {
    matches!(c, ':' | '_' | 'A'..='Z' | 'a'..='z' | '\u{c0}'..='\u{d6}' |
        '\u{d8}'..='\u{f6}' | '\u{f8}'..='\u{2ff}' | '\u{370}'..='\u{37d}' |
        '\u{37f}'..='\u{1fff}' | '\u{200c}'..='\u{200d}' | '\u{2070}'..='\u{218f}' |
        '\u{2c00}'..='\u{2fef}' | '\u{3001}'..='\u{d7ff}' | '\u{f900}'..='\u{fdcf}' |
        '\u{fdf0}'..='\u{fffd}' | '\u{10000}'..='\u{effff}')
}
fn check_name(name: &str) -> Result<(), Error> {
    let mut chars = name.chars();
    if !chars.next().is_some_and(name_start) || !chars.all(|c| name_start(c) ||
        matches!(c, '-' | '.' | '0'..='9' | '\u{b7}' | '\u{300}'..='\u{36f}' | '\u{203f}'..='\u{2040}')) {
        return Err(Error { offset: 0, message: "invalid XML name" });
    }
    Ok(())
}
fn escape(out: &mut String, value: &str, attribute: bool) -> Result<(), Error> {
    for c in value.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' if attribute => out.push_str("&quot;"),
            '\r' => out.push_str("&#13;"),
            '\n' if attribute => out.push_str("&#10;"),
            '\t' if attribute => out.push_str("&#9;"),
            '\t'
            | '\n'
            | '\u{20}'..='\u{d7ff}'
            | '\u{e000}'..='\u{fffd}'
            | '\u{10000}'..='\u{10ffff}' => out.push(c),
            _ => {
                return Err(Error {
                    offset: 0,
                    message: "invalid XML character",
                })
            }
        }
    }
    Ok(())
}
fn open_element(out: &mut String, element: &Element) -> Result<(), Error> {
    check_name(&element.name)?;
    out.push('<');
    out.push_str(&element.name);
    for (i, (key, value)) in element.attributes.iter().enumerate() {
        check_name(key)?;
        if element.attributes[..i]
            .iter()
            .any(|(prior, _)| prior == key)
        {
            return Err(Error {
                offset: 0,
                message: "duplicate XML attribute",
            });
        }
        out.push(' ');
        out.push_str(key);
        out.push_str("=\"");
        escape(out, value, true)?;
        out.push('"');
    }
    if element.children.is_empty() {
        out.push_str("/>");
    } else {
        out.push('>');
    }
    Ok(())
}

/// Render XML 1.0 without adding whitespace. Binary metadata is not XML content.
/// Reject names, characters, or duplicate attributes that XML cannot represent.
pub fn to_xml_string(document: &Document) -> Result<String, Error> {
    let mut out = String::new();
    open_element(&mut out, &document.root)?;
    let mut stack = Vec::new();
    if !document.root.children.is_empty() {
        stack.push((&document.root, 0));
    }
    while let Some((parent, index)) = stack.last_mut() {
        if let Some(child) = parent.children.get(*index) {
            *index += 1;
            match child {
                Node::Text(text) => escape(&mut out, &text.value, false)?,
                Node::Element(element) => {
                    open_element(&mut out, element)?;
                    if !element.children.is_empty() {
                        stack.push((element, 0));
                    }
                }
            }
        } else {
            out.push_str("</");
            out.push_str(&parent.name);
            out.push('>');
            stack.pop();
        }
    }
    Ok(out)
}

/// Count elements, including the root, without recursion.
pub fn element_count(document: &Document) -> usize {
    let mut count = 1;
    let mut stack = vec![document.root.children.iter()];
    while let Some(children) = stack.last_mut() {
        match children.next() {
            Some(Node::Element(element)) => {
                count += 1;
                stack.push(element.children.iter());
            }
            Some(Node::Text(_)) => {}
            None => {
                stack.pop();
            }
        }
    }
    count
}
