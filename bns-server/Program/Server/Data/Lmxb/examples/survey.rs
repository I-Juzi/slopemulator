use std::{
    collections::BTreeMap,
    env, fs,
    path::{Path, PathBuf},
};
const KEY: [u8; 16] = [
    0xa4, 0x9f, 0xd8, 0xb3, 0xf6, 0x8e, 0x39, 0xc2, 0x2d, 0xe0, 0x61, 0x75, 0x5c, 0x4b, 0x1a, 0x07,
];
struct Reader<'a> {
    b: &'a [u8],
    p: usize,
}
impl Reader<'_> {
    fn take(&mut self, n: usize) -> Result<&[u8], String> {
        let end = self.p.checked_add(n).ok_or("overflow")?;
        let x = self
            .b
            .get(self.p..end)
            .ok_or_else(|| format!("EOF at {:x}, need {n}", self.p))?;
        self.p = end;
        Ok(x)
    }
    fn u8(&mut self) -> Result<u8, String> {
        Ok(self.take(1)?[0])
    }
    fn u32(&mut self) -> Result<u32, String> {
        Ok(u32::from_le_bytes(
            self.take(4)?.try_into().map_err(|_| "u32")?,
        ))
    }
    fn string(&mut self) -> Result<String, String> {
        let n = self.u32()? as usize;
        let b = self.take(n.checked_mul(2).ok_or("length")?)?;
        let v: Vec<_> = b
            .as_chunks::<2>()
            .0
            .iter()
            .enumerate()
            .map(|(j, c)| {
                u16::from_le_bytes([c[0] ^ KEY[(j * 2) % 16], c[1] ^ KEY[(j * 2 + 1) % 16]])
            })
            .collect();
        String::from_utf16(&v).map_err(|e| e.to_string())
    }
}
#[derive(Default, Debug)]
struct Stats {
    files: u64,
    bytes: u64,
    elements: u64,
    text: u64,
    max_depth: usize,
    flags: BTreeMap<u8, u64>,
    versions: BTreeMap<u32, u64>,
    nonzero_reserved: u64,
    lines_decrease: u64,
    text_names: BTreeMap<String, u64>,
}
fn walk(path: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    if path.is_dir() {
        for e in fs::read_dir(path)? {
            walk(&e?.path(), out)?;
        }
    } else if path.extension().is_some_and(|x| x == "xml") {
        out.push(path.to_owned());
    }
    Ok(())
}
fn parse(b: &[u8], s: &mut Stats) -> Result<(), String> {
    let mut r = Reader { b, p: 0 };
    if r.take(8)? != b"LMXBOSLB" {
        return Err("magic".into());
    }
    *s.versions.entry(r.u32()?).or_default() += 1;
    if r.u32()? as usize != b.len() {
        return Err("file length".into());
    }
    if r.take(64)?.iter().any(|&x| x != 0) {
        s.nonzero_reserved += 1;
    }
    *s.flags.entry(r.u8()?).or_default() += 1;
    let _source = r.string()?;
    let mut pending = vec![1u32];
    let mut root = true;
    let mut last_line = 0;
    while !pending.is_empty() {
        if pending.last() == Some(&0) {
            pending.pop();
            continue;
        }
        if let Some(n) = pending.last_mut() {
            *n -= 1;
        }
        let kind = if root {
            root = false;
            1
        } else {
            r.u32()?
        };
        if kind == 1 {
            s.elements += 1;
            let count = r.u32()?;
            if count as usize > b.len() / 8 {
                return Err("attr count".into());
            }
            for _ in 0..count {
                r.string()?;
                r.string()?;
            }
        } else if kind == 2 {
            s.text += 1;
            r.string()?;
        } else {
            return Err(format!("kind {kind} at {:x}", r.p - 4));
        }
        *s.flags.entry(r.u8()?).or_default() += 1;
        let name = r.string()?;
        if kind == 2 {
            *s.text_names.entry(name).or_default() += 1;
        }
        let children = r.u32()?;
        if kind == 2 && children != 0 {
            return Err("text children".into());
        }
        let line = r.u32()?;
        if line < last_line {
            s.lines_decrease += 1;
        }
        last_line = line;
        pending.push(children);
        s.max_depth = s.max_depth.max(pending.len() - 1);
        if pending.len() > 1024 {
            return Err("depth".into());
        }
    }
    if r.p != b.len() {
        return Err(format!("trailing {} at {:x}", b.len() - r.p, r.p));
    }
    s.files += 1;
    s.bytes += b.len() as u64;
    Ok(())
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = env::args().nth(1).ok_or("folder argument required")?;
    let mut files = Vec::new();
    walk(Path::new(&path), &mut files)?;
    files.sort();
    let mut stats = Stats::default();
    let mut failures = 0;
    for p in &files {
        let b = fs::read(p)?;
        if let Err(e) = parse(&b, &mut stats) {
            failures += 1;
            if failures <= 20 {
                println!("FAIL {}: {e}", p.display());
            }
        }
    }
    println!("attempted={} failures={failures} {stats:#?}", files.len());
    Ok(())
}
