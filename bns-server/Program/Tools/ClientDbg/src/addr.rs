//! IDA address <-> live address conversion. The IDA image base is 0x400000.

/// Image base used in the IDA database.
pub const IDA_BASE: u32 = 0x40_0000;

/// Convert an IDA address to a live address for a given image base.
pub fn ida_to_live(ida: u32, base: u32) -> u32 {
    ida.wrapping_sub(IDA_BASE).wrapping_add(base)
}

/// Convert a live address to an IDA address for a given image base.
pub fn live_to_ida(live: u32, base: u32) -> u32 {
    live.wrapping_sub(base).wrapping_add(IDA_BASE)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn addr_convert() {
        assert_eq!(ida_to_live(0x5fd770, 0x400000), 0x5fd770);
        // Card says 0xdfd770, but its own formula `addr - 0x400000 + base` gives 0x11fd770.
        assert_eq!(ida_to_live(0x5fd770, 0x1000000), 0x11fd770);
        assert_eq!(live_to_ida(0x5fd770, 0x400000), 0x5fd770);
        assert_eq!(live_to_ida(0x11fd770, 0x1000000), 0x5fd770);
    }
}
