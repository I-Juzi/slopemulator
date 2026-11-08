//! Base64 envelopes from section 07. Lengths count bytes, not words.
use base64::{engine::general_purpose::STANDARD, Engine};
use num_traits::Zero;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Base64,
    Length,
    Public,
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "STS KeyData error: {self:?}")
    }
}
impl std::error::Error for Error {}

// The largest local envelope is a 1024-byte salt plus a 128-byte public value.
const MAX_PAYLOAD: usize = 1160;
fn encode(fields: &[&[u8]]) -> Result<String, Error> {
    let mut out = Vec::new();
    for field in fields {
        let len = u32::try_from(field.len()).map_err(|_| Error::Length)?;
        out.extend_from_slice(&len.to_le_bytes());
        out.extend_from_slice(field);
    }
    if out.len() > MAX_PAYLOAD {
        return Err(Error::Length);
    }
    Ok(STANDARD.encode(out))
}
fn decode<const N: usize>(text: &str) -> Result<[Vec<u8>; N], Error> {
    if text.len() > MAX_PAYLOAD.div_ceil(3) * 4 {
        return Err(Error::Length);
    }
    let bytes = STANDARD.decode(text).map_err(|_| Error::Base64)?;
    let mut rest = bytes.as_slice();
    let mut fields = std::array::from_fn(|_| Vec::new());
    for field in &mut fields {
        let prefix: [u8; 4] = rest
            .get(..4)
            .ok_or(Error::Length)?
            .try_into()
            .map_err(|_| Error::Length)?;
        let len = u32::from_le_bytes(prefix) as usize;
        rest = &rest[4..];
        *field = rest.get(..len).ok_or(Error::Length)?.to_vec();
        rest = &rest[len..];
    }
    if !rest.is_empty() {
        return Err(Error::Length);
    }
    Ok(fields)
}
fn check_challenge(salt: &[u8], public: &[u8]) -> Result<(), Error> {
    if salt.is_empty() || salt.len() > 1024 || public.is_empty() || public.len() > 128 {
        return Err(Error::Length);
    }
    crate::srp6::read_number(salt).map_err(|_| Error::Public)?;
    let b = crate::srp6::read_number(public).map_err(|_| Error::Public)?;
    if b.is_zero() || b >= crate::srp6::modulus() {
        return Err(Error::Public);
    }
    Ok(())
}
pub fn encode_challenge(salt: &[u8], public: &[u8]) -> Result<String, Error> {
    check_challenge(salt, public)?;
    encode(&[salt, public])
}
pub fn decode_challenge(text: &str) -> Result<(Vec<u8>, Vec<u8>), Error> {
    let [salt, public] = decode(text)?;
    check_challenge(&salt, &public)?;
    Ok((salt, public))
}
fn check_client(public: &[u8], proof: &[u8]) -> Result<(), Error> {
    if public.is_empty()
        || public.len() > 128
        || proof.len() != 32
        || !public.len().is_multiple_of(4)
    {
        return Err(Error::Length);
    }
    Ok(())
}
pub fn encode_client(public: &[u8], proof: &[u8]) -> Result<String, Error> {
    check_client(public, proof)?;
    encode(&[public, proof])
}
pub fn decode_client(text: &str) -> Result<(Vec<u8>, Vec<u8>), Error> {
    let [public, proof] = decode(text)?;
    check_client(&public, &proof)?;
    Ok((public, proof))
}
pub fn encode_server(proof: &[u8]) -> Result<String, Error> {
    if proof.len() != 32 {
        return Err(Error::Length);
    }
    encode(&[proof])
}
pub fn decode_server(text: &str) -> Result<[u8; 32], Error> {
    let [proof] = decode(text)?;
    proof.try_into().map_err(|_| Error::Length)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_envelopes_round_trip() {
        let salt = [3; 32];
        let b = [2, 0, 0, 0];
        let proof = [7; 32];
        assert_eq!(
            decode_challenge(&encode_challenge(&salt, &b).unwrap()).unwrap(),
            (salt.to_vec(), b.to_vec())
        );
        assert_eq!(
            decode_client(&encode_client(&b, &proof).unwrap()).unwrap(),
            (b.to_vec(), proof.to_vec())
        );
        assert_eq!(
            decode_server(&encode_server(&proof).unwrap()).unwrap(),
            proof
        );
        let bytes = STANDARD.decode(encode_server(&proof).unwrap()).unwrap();
        assert_eq!(&bytes[..4], &32u32.to_le_bytes());
    }
    #[test]
    fn malformed_lengths_and_trailing_data_fail() {
        for bytes in [
            vec![],
            vec![32, 0, 0],
            vec![255; 4],
            vec![32, 0, 0, 0],
            vec![0; 5],
        ] {
            assert!(decode_server(&STANDARD.encode(bytes)).is_err());
        }
        for size in [0usize, 31, 33] {
            let mut bytes = (size as u32).to_le_bytes().to_vec();
            bytes.extend(vec![0; size]);
            assert!(decode_server(&STANDARD.encode(bytes)).is_err());
        }
        let mut bytes = STANDARD.decode(encode_server(&[4; 32]).unwrap()).unwrap();
        bytes.push(0);
        assert!(decode_server(&STANDARD.encode(bytes)).is_err());
        assert!(decode_client("not base64").is_err());
        assert!(decode_challenge(&encode(&[&[1; 32], &[0; 4]]).unwrap()).is_err());
        assert!(decode_challenge(
            &encode(&[
                &[1; 32],
                &crate::srp6::number_bytes(&crate::srp6::modulus())
            ])
            .unwrap()
        )
        .is_err());
    }
    #[test]
    fn every_truncated_payload_is_rejected() {
        for text in [
            encode_challenge(&[3; 32], &[2, 0, 0, 0]).unwrap(),
            encode_client(&[2, 0, 0, 0], &[7; 32]).unwrap(),
        ] {
            let bytes = STANDARD.decode(text).unwrap();
            for end in 0..bytes.len() {
                assert!(decode::<2>(&STANDARD.encode(&bytes[..end])).is_err());
            }
            let mut extra = bytes;
            extra.push(0);
            assert!(decode::<2>(&STANDARD.encode(extra)).is_err());
        }
    }
}
