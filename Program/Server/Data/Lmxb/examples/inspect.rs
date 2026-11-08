use std::{
    env, fs,
    path::{Path, PathBuf},
};
const KEY: [u8; 16] = [
    0xa4, 0x9f, 0xd8, 0xb3, 0xf6, 0x8e, 0x39, 0xc2, 0x2d, 0xe0, 0x61, 0x75, 0x5c, 0x4b, 0x1a, 0x07,
];
fn files(path: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    if path.is_dir() {
        for e in fs::read_dir(path)? {
            files(&e?.path(), out)?;
        }
    } else if path.extension().is_some_and(|s| s == "xml") {
        out.push(path.to_owned());
    }
    Ok(())
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    for arg in env::args().skip(1) {
        let p = Path::new(&arg);
        let mut list = Vec::new();
        files(p, &mut list)?;
        list.sort_by_key(|p| fs::metadata(p).map(|m| m.len()).unwrap_or(0));
        println!("{} files={}", p.display(), list.len());
        for path in list.iter().take(2).chain(list.iter().rev().take(1)) {
            let b = fs::read(path)?;
            println!(
                "{} size={} header={:02x?}",
                path.display(),
                b.len(),
                &b[..b.len().min(80)]
            );
            // Print plausible transformed strings, and every gap as bytes.
            let mut i = 80.min(b.len());
            let mut gap = i;
            while i + 4 <= b.len().min(1500) {
                let n = u32::from_le_bytes(b[i..i + 4].try_into()?) as usize;
                if n > 0 && n < 200 && i + 4 + 2 * n <= b.len() {
                    let v: Vec<_> = b[i + 4..i + 4 + 2 * n]
                        .as_chunks::<2>()
                        .0
                        .iter()
                        .enumerate()
                        .map(|(j, c)| {
                            u16::from_le_bytes([
                                c[0] ^ KEY[(j * 2) % 16],
                                c[1] ^ KEY[(j * 2 + 1) % 16],
                            ])
                        })
                        .collect();
                    if let Ok(s) = String::from_utf16(&v) {
                        if s.chars().all(|c| !c.is_control() || "\r\n\t".contains(c))
                            && s.chars().any(|c| c.is_ascii())
                        {
                            println!("{gap:04x} gap {:02x?}", &b[gap..i]);
                            println!("{i:04x} str({n}) {s:?}");
                            i += 4 + 2 * n;
                            gap = i;
                            continue;
                        }
                    }
                }
                i += 1;
            }
            println!("{gap:04x} end {:02x?}", &b[gap..i.min(b.len())]);
        }
    }
    Ok(())
}
