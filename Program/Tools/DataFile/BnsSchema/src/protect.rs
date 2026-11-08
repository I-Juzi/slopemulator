//! Conservative refusal rules. A refusal is a normal outcome.
use crate::pe::Pe;
use std::fmt;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refused(pub String);
impl fmt::Display for Refused {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for Refused {}
pub const PACKER_SECTIONS: &[&str] = &[
    ".themida", ".winlice", ".vmp0", ".vmp1", ".vmp2", ".enigma1", ".enigma2", "upx0", "upx1",
    ".aspack", ".adata", ".mpress1", ".mpress2", ".petite", ".nsp0", ".nsp1", ".perplex", ".boom",
    ".arxan",
];
pub fn check(pe: &Pe<'_>) -> Result<(), Refused> {
    for s in &pe.sections {
        if PACKER_SECTIONS
            .iter()
            .any(|n| s.name.eq_ignore_ascii_case(n))
        {
            return Err(Refused(format!("packer/protector section: {}", s.name)));
        }
    }
    let first = pe.sections.iter().position(|s| s.executable());
    let entry = pe.sections.iter().position(|s| {
        let rva = u64::from(pe.entry_point);
        (u64::from(s.va)..u64::from(s.va) + u64::from(s.vsize.max(s.raw_size))).contains(&rva)
    });
    let Some(index) = entry else {
        return Err(Refused("entry point is not in a section".into()));
    };
    if first != Some(index) {
        return Err(Refused(
            "entry point is not in the first executable section".into(),
        ));
    }
    if pe.sections[index].writable() {
        return Err(Refused("entry-point section is writable".into()));
    }
    for s in &pe.sections {
        if s.raw_size >= 65536 && s.entropy >= 7.4 {
            return Err(Refused(format!(
                "high-entropy section: {} (entropy >= 7.4)",
                s.name
            )));
        }
        if s.executable() && s.raw_size == 0 && s.vsize > 0 {
            return Err(Refused(format!(
                "executable section has no raw bytes: {}",
                s.name
            )));
        }
    }
    Ok(())
}
