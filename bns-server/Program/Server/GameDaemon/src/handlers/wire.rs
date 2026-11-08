//! Small body-prefix reader. It deliberately permits trailing bytes.
//! Packet's separate gr2 codec supplies structural round trips. No crypto here.
pub(super) struct Reader<'a>(pub &'a [u8]);
impl Reader<'_> {
    pub fn take<const N: usize>(&mut self) -> Option<[u8; N]> {
        let bytes = self.0.get(..N)?.try_into().ok()?;
        self.0 = &self.0[N..];
        Some(bytes)
    }
    pub fn byte(&mut self) -> Option<u8> {
        Some(self.take::<1>()?[0])
    }
    pub fn short(&mut self) -> Option<u16> {
        Some(u16::from_le_bytes(self.take()?))
    }
    pub fn int(&mut self) -> Option<u32> {
        Some(u32::from_le_bytes(self.take()?))
    }
    pub fn quad(&mut self) -> Option<u64> {
        Some(u64::from_le_bytes(self.take()?))
    }
    pub fn blob(&mut self, capacity: usize) -> Option<Vec<u8>> {
        let n = self.short()? as i16;
        if n < 0 || n as usize > capacity {
            return None;
        }
        let bytes = self.0.get(..n as usize)?.to_vec();
        self.0 = &self.0[n as usize..];
        Some(bytes)
    }
    pub fn string(&mut self, capacity: usize) -> Option<Vec<u16>> {
        let n = self.short()? as i16;
        if n < 0 || n as usize >= capacity {
            return None;
        }
        (0..n).map(|_| self.short()).collect()
    }
}
