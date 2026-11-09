//! Hex helpers.

pub fn encode(data: &[u8]) -> String {
    const D: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(data.len() * 2);
    for b in data {
        s.push(D[(b >> 4) as usize] as char);
        s.push(D[(b & 15) as usize] as char);
    }
    s
}

pub fn decode(s: &str) -> Result<Vec<u8>, String> {
    let s: Vec<u8> = s.bytes().filter(|c| !c.is_ascii_whitespace()).collect();
    if !s.len().is_multiple_of(2) {
        return Err("odd hex length".into());
    }
    let nib = |c: u8| -> Result<u8, String> {
        match c {
            b'0'..=b'9' => Ok(c - b'0'),
            b'a'..=b'f' => Ok(c - b'a' + 10),
            b'A'..=b'F' => Ok(c - b'A' + 10),
            _ => Err("bad hex digit".into()),
        }
    };
    s.chunks(2)
        .map(|p| Ok((nib(p[0])? << 4) | nib(p[1])?))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_roundtrip() {
        assert_eq!(encode(&[0, 0xab, 0xff]), "00abff");
        assert_eq!(decode("00AB ff").unwrap(), vec![0, 0xab, 0xff]);
        assert!(decode("0").is_err());
        assert!(decode("zz").is_err());
    }
}
