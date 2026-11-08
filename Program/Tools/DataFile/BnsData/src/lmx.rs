//! LMXBOSLB v3. Strings retain raw UTF-16 code units, including unpaired surrogates.
use crate::{Cursor, Error, Result};

pub type Utf16 = Vec<u16>;
pub type Key = [u8; 16];
const MAX_DEPTH: usize = 256;

/// Read exactly 16 raw bytes or 32 ASCII hexadecimal characters. Never logs key material.
pub fn load_key() -> Result<Key> {
    let path = std::env::var_os("BNSDATA_KEY_FILE").ok_or(Error::Invalid(
        "set BNSDATA_KEY_FILE to an LMX XOR key file",
    ))?;
    decode_key(&std::fs::read(path)?)
}
fn decode_key(bytes: &[u8]) -> Result<Key> {
    if bytes.len() == 16 {
        return Ok(bytes.try_into().unwrap());
    }
    if bytes.len() != 32 {
        return Err(Error::Invalid(
            "LMX key must be 16 raw bytes or 32 hex characters",
        ));
    }
    let mut key = [0; 16];
    for (out, pair) in key.iter_mut().zip(bytes.as_chunks::<2>().0.iter()) {
        let digit = |b: u8| {
            (b as char)
                .to_digit(16)
                .map(|x| x as u8)
                .ok_or(Error::Invalid("LMX key contains a non-hex character"))
        };
        *out = digit(pair[0])? * 16 + digit(pair[1])?;
    }
    Ok(key)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Document {
    pub padding: [u8; 3],
    pub reserved: [u8; 64],
    pub valid: u8,
    pub name: Utf16,
    pub root: Element,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Element {
    pub attributes: Vec<(Utf16, Utf16)>,
    pub node: Node,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Text {
    pub text: Utf16,
    pub node: Node,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Node {
    pub valid: u8,
    pub name: Utf16,
    pub line: u32,
    pub children: Vec<Child>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Child {
    Element(Element),
    Text(Text),
}

fn depth_ok(depth: usize) -> Result<()> {
    if depth > MAX_DEPTH {
        return Err(Error::Invalid("LMX nesting exceeds 256"));
    }
    Ok(())
}
fn header<'a>(bytes: &'a [u8]) -> Result<(Cursor<'a>, [u8; 3], [u8; 64])> {
    let mut c = Cursor::new(bytes);
    if c.take(8)? != b"LMXBOSLB" || c.u8()? != 3 {
        return Err(Error::Invalid("not LMXBOSLB version 3"));
    }
    let padding = c.take(3)?.try_into().unwrap();
    if u64::from(c.u32()?) != bytes.len() as u64 {
        return Err(Error::Invalid("LMX declared size differs from input"));
    }
    let reserved = c.take(64)?.try_into().unwrap();
    Ok((c, padding, reserved))
}
fn string_bytes<'a>(c: &mut Cursor<'a>) -> Result<&'a [u8]> {
    let count = c.u32()?;
    let count = c.count(u64::from(count), 2)?;
    c.take(count * 2)
}
fn read_string(c: &mut Cursor<'_>, key: &Key) -> Result<Utf16> {
    Ok(string_bytes(c)?
        .as_chunks::<2>()
        .0
        .iter()
        .enumerate()
        .map(|(i, p)| u16::from_le_bytes([p[0] ^ key[(i * 2) % 16], p[1] ^ key[(i * 2 + 1) % 16]]))
        .collect())
}
fn read_element(c: &mut Cursor<'_>, key: &Key, depth: usize) -> Result<Element> {
    depth_ok(depth)?;
    let count = c.u32()?;
    let count = c.count(u64::from(count), 8)?;
    let mut attributes = Vec::with_capacity(count);
    for _ in 0..count {
        attributes.push((read_string(c, key)?, read_string(c, key)?));
    }
    Ok(Element {
        attributes,
        node: read_node(c, key, depth)?,
    })
}
fn read_node(c: &mut Cursor<'_>, key: &Key, depth: usize) -> Result<Node> {
    let valid = c.u8()?;
    let name = read_string(c, key)?;
    let count = c.u32()?;
    let line = c.u32()?;
    let count = c.count(u64::from(count), 21)?;
    let mut children = Vec::with_capacity(count);
    for _ in 0..count {
        depth_ok(depth + 1)?;
        children.push(match c.u32()? {
            1 => Child::Element(read_element(c, key, depth + 1)?),
            2 => Child::Text(Text {
                text: read_string(c, key)?,
                node: read_node(c, key, depth + 1)?,
            }),
            _ => return Err(Error::Invalid("unknown LMX child tag")),
        });
    }
    Ok(Node {
        valid,
        name,
        line,
        children,
    })
}
impl Document {
    pub fn parse(bytes: &[u8], key: &Key) -> Result<Self> {
        let (mut c, padding, reserved) = header(bytes)?;
        let valid = c.u8()?;
        let name = read_string(&mut c, key)?;
        let root = read_element(&mut c, key, 0)?;
        c.finish()?;
        Ok(Self {
            padding,
            reserved,
            valid,
            name,
            root,
        })
    }
    pub fn binary_size(&self) -> Result<u32> {
        let size = 81 + string_size(&self.name)? + element_size(&self.root, 0)?;
        u32::try_from(size).map_err(|_| Error::Invalid("LMX exceeds u32 size"))
    }
    pub fn write(&self, key: &Key) -> Result<Vec<u8>> {
        let size = self.binary_size()?;
        let mut out = Vec::with_capacity(size as usize);
        out.extend_from_slice(b"LMXBOSLB\x03");
        out.extend_from_slice(&self.padding);
        out.extend_from_slice(&size.to_le_bytes());
        out.extend_from_slice(&self.reserved);
        out.push(self.valid);
        put_string(&mut out, &self.name, key);
        put_element(&mut out, &self.root, key);
        Ok(out)
    }
}
fn string_size(s: &[u16]) -> Result<u64> {
    let n = u32::try_from(s.len()).map_err(|_| Error::Invalid("LMX string too long"))?;
    Ok(4 + 2 * u64::from(n))
}
fn node_size(n: &Node, depth: usize) -> Result<u64> {
    depth_ok(depth)?;
    u32::try_from(n.children.len()).map_err(|_| Error::Invalid("too many LMX children"))?;
    let mut size = 9 + string_size(&n.name)?;
    for child in &n.children {
        size = size
            .checked_add(
                4 + match child {
                    Child::Element(e) => element_size(e, depth + 1)?,
                    Child::Text(t) => string_size(&t.text)? + node_size(&t.node, depth + 1)?,
                },
            )
            .ok_or(Error::Invalid("LMX size overflow"))?;
    }
    Ok(size)
}
fn element_size(e: &Element, depth: usize) -> Result<u64> {
    depth_ok(depth)?;
    u32::try_from(e.attributes.len()).map_err(|_| Error::Invalid("too many LMX attributes"))?;
    let mut size = 4 + node_size(&e.node, depth)?;
    for (name, value) in &e.attributes {
        size = size
            .checked_add(string_size(name)? + string_size(value)?)
            .ok_or(Error::Invalid("LMX size overflow"))?;
    }
    Ok(size)
}
fn put_string(out: &mut Vec<u8>, text: &[u16], key: &Key) {
    out.extend_from_slice(&(text.len() as u32).to_le_bytes());
    for (i, unit) in text.iter().enumerate() {
        for (j, byte) in unit.to_le_bytes().iter().enumerate() {
            out.push(byte ^ key[(2 * i + j) % 16]);
        }
    }
}
fn put_element(out: &mut Vec<u8>, e: &Element, key: &Key) {
    out.extend_from_slice(&(e.attributes.len() as u32).to_le_bytes());
    for (name, value) in &e.attributes {
        put_string(out, name, key);
        put_string(out, value, key);
    }
    put_node(out, &e.node, key);
}
fn put_node(out: &mut Vec<u8>, n: &Node, key: &Key) {
    out.push(n.valid);
    put_string(out, &n.name, key);
    out.extend_from_slice(&(n.children.len() as u32).to_le_bytes());
    out.extend_from_slice(&n.line.to_le_bytes());
    for child in &n.children {
        match child {
            Child::Element(e) => {
                out.extend_from_slice(&1u32.to_le_bytes());
                put_element(out, e, key);
            }
            Child::Text(t) => {
                out.extend_from_slice(&2u32.to_le_bytes());
                put_string(out, &t.text, key);
                put_node(out, &t.node, key);
            }
        }
    }
}

/// Counts only. The walker does not decode or retain string payloads.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Structure {
    pub elements: usize,
    pub text_nodes: usize,
    pub attributes: usize,
    pub strings: usize,
    pub utf16_units: usize,
    pub binary_size: usize,
}
pub fn walk(bytes: &[u8]) -> Result<Structure> {
    let (mut c, _, _) = header(bytes)?;
    let mut stats = Structure {
        binary_size: bytes.len(),
        ..Structure::default()
    };
    c.u8()?;
    skip_string(&mut c, &mut stats)?;
    walk_record(&mut c, &mut stats, 1, 0)?;
    c.finish()?;
    Ok(stats)
}
fn skip_string(c: &mut Cursor<'_>, s: &mut Structure) -> Result<()> {
    let bytes = string_bytes(c)?;
    s.strings += 1;
    s.utf16_units += bytes.len() / 2;
    Ok(())
}
fn walk_record(c: &mut Cursor<'_>, s: &mut Structure, tag: u32, depth: usize) -> Result<()> {
    depth_ok(depth)?;
    match tag {
        1 => {
            s.elements += 1;
            let count = c.u32()?;
            let count = c.count(u64::from(count), 8)?;
            s.attributes += count;
            for _ in 0..count {
                skip_string(c, s)?;
                skip_string(c, s)?;
            }
        }
        2 => {
            s.text_nodes += 1;
            skip_string(c, s)?;
        }
        _ => return Err(Error::Invalid("unknown LMX child tag")),
    }
    c.u8()?;
    skip_string(c, s)?;
    let count = c.u32()?;
    c.u32()?;
    let count = c.count(u64::from(count), 21)?;
    for _ in 0..count {
        let tag = c.u32()?;
        walk_record(c, s, tag, depth + 1)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{rngs::StdRng, RngCore, SeedableRng};
    fn node() -> Node {
        Node {
            valid: 7,
            name: vec![],
            line: 123,
            children: vec![],
        }
    }
    fn sample() -> Document {
        Document {
            padding: [3, 2, 1],
            reserved: [9; 64],
            valid: 2,
            name: vec![],
            root: Element {
                attributes: vec![],
                node: Node {
                    children: vec![
                        Child::Text(Text {
                            text: vec![0xd800, 0, 0x1234],
                            node: node(),
                        }),
                        Child::Element(Element {
                            attributes: vec![(vec![65], vec![66]), (vec![65], vec![])],
                            node: node(),
                        }),
                    ],
                    ..node()
                },
            },
        }
    }
    #[test]
    fn roundtrip_size_and_structure() {
        let mut key = [0; 16];
        StdRng::seed_from_u64(47).fill_bytes(&mut key);
        let doc = sample();
        let bytes = doc.write(&key).unwrap();
        assert_eq!(
            bytes.len(),
            81 + 4 + 4 + 13 + (4 + 10 + 13) + (4 + 4 + 12 + 10 + 13)
        );
        assert_eq!(doc.binary_size().unwrap() as usize, bytes.len());
        let parsed = Document::parse(&bytes, &key).unwrap();
        assert_eq!(doc, parsed);
        assert_eq!(bytes, parsed.write(&key).unwrap());
        let stats = walk(&bytes).unwrap();
        assert_eq!(
            (stats.elements, stats.text_nodes, stats.attributes),
            (2, 1, 2)
        );
    }
    #[test]
    fn rejects_unknown_tags_sizes_and_truncation() {
        let mut key = [0; 16];
        StdRng::seed_from_u64(81).fill_bytes(&mut key);
        let bytes = sample().write(&key).unwrap();
        let mut bad = bytes.clone();
        bad[102..106].copy_from_slice(&99u32.to_le_bytes());
        assert!(Document::parse(&bad, &key).is_err());
        assert!(walk(&bad).is_err());
        for end in 0..bytes.len() {
            assert!(Document::parse(&bytes[..end], &key).is_err());
        }
        let mut bad = bytes.clone();
        bad.push(0);
        assert!(walk(&bad).is_err());
        let mut bad = bytes;
        bad[12..16].copy_from_slice(&80u32.to_le_bytes());
        assert!(walk(&bad).is_err());
    }
    #[test]
    fn key_encodings() {
        let mut generated = [0; 16];
        StdRng::seed_from_u64(999).fill_bytes(&mut generated);
        let hex: String = generated.iter().map(|b| format!("{b:02x}")).collect();
        assert_eq!(
            decode_key(&generated).unwrap(),
            decode_key(hex.as_bytes()).unwrap()
        );
        assert!(decode_key(b"invalid").is_err());
        assert!(decode_key(&[b'z'; 32]).is_err());
    }
}
