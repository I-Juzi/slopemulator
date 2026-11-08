//! Find a null slot followed by consecutive, named table definitions.
use crate::{layout::Layout, pe::Pe, Error};
pub const MAX_SLOTS: usize = 4096;
#[derive(Debug)]
pub struct Array {
    pub va: u64,
    pub definitions: Vec<u64>,
}
pub(crate) fn safe_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'_' || c == b'-')
}
fn slot(pe: &Pe<'_>, l: &Layout, array: u64, i: usize) -> Result<u64, Error> {
    pe.uint(array, i * l.p, l.p)
}
fn valid(pe: &Pe<'_>, l: &Layout, td: u64, i: usize) -> bool {
    let test = || -> Result<bool, Error> {
        if pe.uint(td, l.td.kind, 2)? as i16 != i as i16 {
            return Ok(false);
        }
        let name = pe.wstr(pe.uint(td, l.td.name, l.p)?)?;
        Ok(name.as_deref().is_some_and(safe_name))
    };
    test().unwrap_or(false)
}
fn walk(pe: &Pe<'_>, l: &Layout, va: u64) -> Result<Array, Error> {
    if slot(pe, l, va, 0)? != 0 {
        return Err(Error::Format("array slot 0 is not null".into()));
    }
    let mut definitions = Vec::new();
    for i in 1..MAX_SLOTS {
        let Ok(td) = slot(pe, l, va, i) else {
            break;
        };
        if !valid(pe, l, td, i) {
            break;
        }
        definitions.push(td);
    }
    if definitions.len() == MAX_SLOTS - 1
        && slot(pe, l, va, MAX_SLOTS).is_ok_and(|td| valid(pe, l, td, MAX_SLOTS))
    {
        return Err(Error::Format("definition array exceeds 4096 slots".into()));
    }
    Ok(Array { va, definitions })
}
pub fn find(pe: &Pe<'_>, l: &Layout, override_va: Option<u64>) -> Result<Array, Error> {
    if let Some(va) = override_va {
        let a = walk(pe, l, va)?;
        return if a.definitions.is_empty() {
            Err(Error::NotFound(
                "no definitions at explicit array VA".into(),
            ))
        } else {
            Ok(a)
        };
    }
    let mut best = None;
    for section in &pe.sections {
        if section.executable() || !section.initialized() {
            continue;
        }
        let start = pe
            .image_base
            .checked_add(u64::from(section.va))
            .ok_or_else(|| Error::Format("section VA overflow".into()))?;
        let align = (l.p as u64 - start % l.p as u64) % l.p as u64;
        let mut offset = align;
        while offset + (2 * l.p) as u64 <= u64::from(section.raw_size) {
            let va = start
                .checked_add(offset)
                .ok_or_else(|| Error::Format("scan VA overflow".into()))?;
            if slot(pe, l, va, 0)? == 0 && valid(pe, l, slot(pe, l, va, 1)?, 1) {
                let a = walk(pe, l, va)?;
                if a.definitions.len() >= 16
                    && best
                        .as_ref()
                        .is_none_or(|b: &Array| a.definitions.len() > b.definitions.len())
                {
                    best = Some(a);
                }
            }
            offset += l.p as u64;
        }
    }
    best.ok_or_else(|| Error::NotFound("no definition array with at least 16 tables".into()))
}
