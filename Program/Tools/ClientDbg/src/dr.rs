//! Debug register (DR7) helpers. No OS calls, so Linux tests can check them.

use crate::proto::Kind;

pub const SLOTS: u8 = 4;

/// Check breakpoint arguments.
pub fn validate(addr: u32, kind: Kind, len: u8, slot: u8) -> Result<(), String> {
    if slot >= SLOTS {
        return Err("slot must be 0 to 3".into());
    }
    if kind == Kind::Exec {
        return Ok(()); // length is ignored for exec
    }
    if !matches!(len, 1 | 2 | 4) {
        return Err("len must be 1, 2 or 4".into());
    }
    if !addr.is_multiple_of(len as u32) {
        return Err("address is not aligned to len".into());
    }
    Ok(())
}

/// DR7 bits (local enable, RW, LEN) for one slot. Call `validate` first.
pub fn dr7_bits(slot: u8, kind: Kind, len: u8) -> u32 {
    let s = slot as u32;
    let rw: u32 = match kind {
        Kind::Exec => 0b00,
        Kind::Write => 0b01,
        Kind::Rw => 0b11,
    };
    let ln: u32 = if kind == Kind::Exec {
        0b00
    } else {
        match len {
            2 => 0b01,
            4 => 0b11,
            _ => 0b00,
        }
    };
    (1 << (s * 2)) | (rw << (16 + s * 4)) | (ln << (18 + s * 4))
}

/// All DR7 bits that belong to one slot.
pub fn dr7_slot_mask(slot: u8) -> u32 {
    let s = slot as u32;
    (0b11 << (s * 2)) | (0b1111 << (16 + s * 4))
}

/// Lowest slot that DR6 reports as hit.
pub fn dr6_slot(dr6: u32) -> Option<u8> {
    (0..SLOTS).find(|s| dr6 & (1 << s) != 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dr7_encoding() {
        assert_eq!(dr7_bits(0, Kind::Exec, 1), 0x1);
        assert_eq!(dr7_bits(0, Kind::Write, 4), 0x1 | (1 << 16) | (3 << 18));
        assert_eq!(dr7_bits(3, Kind::Rw, 2), (1 << 6) | (3 << 28) | (1 << 30));
        assert_eq!(dr7_bits(1, Kind::Exec, 4) & dr7_slot_mask(1), 1 << 2);
    }

    #[test]
    fn dr_validate() {
        assert!(validate(0x1000, Kind::Rw, 4, 0).is_ok());
        assert!(validate(0x1001, Kind::Rw, 4, 0).is_err());
        assert!(validate(0x1001, Kind::Exec, 1, 0).is_ok());
        assert!(validate(0x1000, Kind::Write, 3, 0).is_err());
        assert!(validate(0x1000, Kind::Write, 4, 4).is_err());
    }

    #[test]
    fn dr6_decode() {
        assert_eq!(dr6_slot(0xffff0ff0), None);
        assert_eq!(dr6_slot(0xffff0ff4), Some(2));
    }
}
