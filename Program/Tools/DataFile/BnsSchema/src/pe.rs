//! Checked, passive PE file access. No executable memory is created.
use crate::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Width32or64 {
    Width32,
    Width64,
}

#[derive(Debug)]
pub struct Section {
    pub name: String,
    pub va: u32,
    pub vsize: u32,
    pub raw_offset: u32,
    pub raw_size: u32,
    pub characteristics: u32,
    pub entropy: f64,
}
impl Section {
    pub fn executable(&self) -> bool {
        self.characteristics & 0x2000_0000 != 0
    }
    pub fn writable(&self) -> bool {
        self.characteristics & 0x8000_0000 != 0
    }
    pub fn initialized(&self) -> bool {
        self.characteristics & 0x40 != 0
    }
}

pub struct Pe<'a> {
    pub bytes: &'a [u8],
    pub machine: u16,
    pub width: Width32or64,
    pub image_base: u64,
    pub entry_point: u32,
    pub sections: Vec<Section>,
}
fn slice(bytes: &[u8], offset: usize, size: usize) -> Result<&[u8], Error> {
    let end = offset
        .checked_add(size)
        .ok_or_else(|| Error::Format("file offset overflow".into()))?;
    bytes
        .get(offset..end)
        .ok_or_else(|| Error::Format("truncated PE file".into()))
}
fn word(bytes: &[u8], offset: usize, size: usize) -> Result<u64, Error> {
    let mut value = 0;
    for (i, b) in slice(bytes, offset, size)?.iter().enumerate() {
        value |= u64::from(*b) << (8 * i);
    }
    Ok(value)
}
pub fn entropy(bytes: &[u8]) -> f64 {
    if bytes.is_empty() {
        return 0.0;
    }
    let mut bins = [0usize; 256];
    for b in bytes {
        bins[usize::from(*b)] += 1;
    }
    bins.iter()
        .filter(|&&n| n != 0)
        .map(|&n| {
            let p = n as f64 / bytes.len() as f64;
            -p * p.log2()
        })
        .sum()
}
impl<'a> Pe<'a> {
    pub fn parse(bytes: &'a [u8]) -> Result<Self, Error> {
        if slice(bytes, 0, 2)? != b"MZ" {
            return Err(Error::Format("missing DOS signature".into()));
        }
        let pe = word(bytes, 0x3c, 4)? as usize;
        // Validate the complete COFF header before adding offsets.
        let coff = slice(bytes, pe, 24)?;
        if &coff[..4] != b"PE\0\0" {
            return Err(Error::Format("missing PE signature".into()));
        }
        let machine = word(coff, 4, 2)? as u16;
        let count = word(coff, 6, 2)? as usize;
        let opt_size = word(coff, 20, 2)? as usize;
        let opt_start = pe
            .checked_add(24)
            .ok_or_else(|| Error::Format("PE offset overflow".into()))?;
        let opt = slice(bytes, opt_start, opt_size)?;
        let (width, image_base, min_size) = match word(opt, 0, 2)? {
            0x10b => (Width32or64::Width32, word(opt, 28, 4)?, 96),
            0x20b => (Width32or64::Width64, word(opt, 24, 8)?, 112),
            _ => return Err(Error::Format("unsupported optional header".into())),
        };
        slice(opt, 0, min_size)?;
        if !matches!(
            (machine, width),
            (0x14c, Width32or64::Width32) | (0x8664, Width32or64::Width64)
        ) {
            return Err(Error::Format(
                "unsupported PE machine or width mismatch".into(),
            ));
        }
        let entry_point = word(opt, 16, 4)? as u32;
        let start = opt_start
            .checked_add(opt_size)
            .ok_or_else(|| Error::Format("section offset overflow".into()))?;
        let headers = slice(bytes, start, count * 40)?;
        let mut sections = Vec::new();
        for h in headers.as_chunks::<40>().0 {
            let end = h[..8].iter().position(|&b| b == 0).unwrap_or(8);
            let name: String = h[..end].iter().map(|&b| char::from(b)).collect();
            let vsize = word(h, 8, 4)? as u32;
            let va = word(h, 12, 4)? as u32;
            let raw_size = word(h, 16, 4)? as u32;
            let raw_offset = word(h, 20, 4)? as u32;
            let characteristics = word(h, 36, 4)? as u32;
            let raw = if raw_size == 0 {
                &[][..]
            } else {
                slice(bytes, raw_offset as usize, raw_size as usize)?
            };
            if u64::from(va) + u64::from(vsize.max(raw_size)) > u64::from(u32::MAX) + 1 {
                return Err(Error::Format("section RVA overflow".into()));
            }
            sections.push(Section {
                name,
                va,
                vsize,
                raw_offset,
                raw_size,
                characteristics,
                entropy: entropy(raw),
            });
        }
        Ok(Self {
            bytes,
            machine,
            width,
            image_base,
            entry_point,
            sections,
        })
    }
    /// Translate only file-backed section bytes, not zero-filled virtual tails.
    pub fn va_to_offset(&self, va: u64) -> Option<usize> {
        self.range(va, 1).ok().map(|(off, _)| off)
    }
    fn range(&self, va: u64, size: usize) -> Result<(usize, usize), Error> {
        if va == 0 {
            return Err(Error::Format("null VA".into()));
        }
        let rva = va
            .checked_sub(self.image_base)
            .ok_or_else(|| Error::Format("VA below image base".into()))?;
        for s in &self.sections {
            if let Some(delta) = rva.checked_sub(u64::from(s.va)) {
                if delta < u64::from(s.raw_size) && (size as u64) <= u64::from(s.raw_size) - delta {
                    let off = usize::try_from(u64::from(s.raw_offset) + delta)
                        .map_err(|_| Error::Format("file offset overflow".into()))?;
                    slice(self.bytes, off, size)?;
                    return Ok((off, size));
                }
            }
        }
        Err(Error::Format(format!("VA {va:#x} is not file-backed")))
    }
    pub(crate) fn data(&self, va: u64, size: usize) -> Result<&[u8], Error> {
        let (off, size) = self.range(va, size)?;
        slice(self.bytes, off, size)
    }
    pub(crate) fn uint(&self, va: u64, offset: usize, size: usize) -> Result<u64, Error> {
        let at = va
            .checked_add(offset as u64)
            .ok_or_else(|| Error::Format("VA overflow".into()))?;
        word(self.data(at, size)?, 0, size)
    }
    pub(crate) fn wstr(&self, va: u64) -> Result<Option<String>, Error> {
        if self.va_to_offset(va).is_none() {
            return Ok(None);
        }
        let mut units = Vec::new();
        for i in 0..=4096 {
            let unit = self.uint(va, i * 2, 2)? as u16;
            if unit == 0 {
                return String::from_utf16(&units)
                    .map(Some)
                    .map_err(|_| Error::Format("invalid UTF-16 string".into()));
            }
            if i == 4096 {
                break;
            }
            units.push(unit);
        }
        Err(Error::Format("UTF-16 string exceeds 4096 units".into()))
    }
}
