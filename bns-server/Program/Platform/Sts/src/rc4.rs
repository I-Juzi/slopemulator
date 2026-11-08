//! Legacy RC4 for the local 2020 STS wire only. No keystream bytes are dropped.
//! Do not use RC4 to design a new protocol.
pub struct Rc4 {
    state: [u8; 256],
    i: u8,
    j: u8,
}
impl Rc4 {
    pub fn new(key: &[u8]) -> Option<Self> {
        if key.is_empty() || key.len() > 256 {
            return None;
        }
        let mut state = std::array::from_fn(|i| i as u8);
        let mut j = 0u8;
        for i in 0..256 {
            j = j.wrapping_add(state[i]).wrapping_add(key[i % key.len()]);
            state.swap(i, j as usize);
        }
        Some(Self { state, i: 0, j: 0 })
    }
    pub fn apply(&mut self, bytes: &mut [u8]) {
        for byte in bytes {
            self.i = self.i.wrapping_add(1);
            self.j = self.j.wrapping_add(self.state[self.i as usize]);
            self.state.swap(self.i as usize, self.j as usize);
            let index = self.state[self.i as usize].wrapping_add(self.state[self.j as usize]);
            *byte ^= self.state[index as usize];
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rfc_6229_40_bit_key() {
        let mut rc4 = Rc4::new(&[1, 2, 3, 4, 5]).unwrap();
        let mut bytes = [0; 272];
        rc4.apply(&mut bytes);
        assert_eq!(
            &bytes[..16],
            &[
                0xb2, 0x39, 0x63, 0x05, 0xf0, 0x3d, 0xc0, 0x27, 0xcc, 0xc3, 0x52, 0x4a, 0x0a, 0x11,
                0x18, 0xa8
            ]
        );
        assert_eq!(
            &bytes[16..32],
            &[
                0x69, 0x82, 0x94, 0x4f, 0x18, 0xfc, 0x82, 0xd5, 0x89, 0xc4, 0x03, 0xa4, 0x7a, 0x0d,
                0x09, 0x19
            ]
        );
        assert_eq!(
            &bytes[256..272],
            &[
                0x1c, 0xfc, 0xf6, 0x2b, 0x03, 0xed, 0xdb, 0x64, 0x1d, 0x77, 0xdf, 0xcf, 0x7f, 0x8d,
                0x8c, 0x93
            ]
        );
    }
    #[test]
    fn rfc_6229_64_bit_key_and_chunks() {
        let mut rc4 = Rc4::new(&[1, 2, 3, 4, 5, 6, 7, 8]).unwrap();
        let mut bytes = [0; 32];
        for part in bytes.chunks_mut(3) {
            rc4.apply(part);
        }
        assert_eq!(
            &bytes[..16],
            &[
                0x97, 0xab, 0x8a, 0x1b, 0xf0, 0xaf, 0xb9, 0x61, 0x32, 0xf2, 0xf6, 0x72, 0x58, 0xda,
                0x15, 0xa8
            ]
        );
        assert_eq!(
            &bytes[16..],
            &[
                0x82, 0x63, 0xef, 0xdb, 0x45, 0xc4, 0xa1, 0x86, 0x84, 0xef, 0x87, 0xe6, 0xb1, 0x9e,
                0x5b, 0x09
            ]
        );
        assert!(Rc4::new(&[]).is_none());
        assert!(Rc4::new(&[0; 257]).is_none());
    }
}
