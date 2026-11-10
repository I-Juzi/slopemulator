//! Non-goods reply conversion. Spec 05 §7.
use crate::wire::invalid;
use bloodlib::packet::{compose, decode_fields, decompose, DecodeLimits, Value};
use packet_ids::LobbyCmd;
use std::io;
fn p(e: impl std::fmt::Display) -> io::Error {
    invalid(&e.to_string())
}
pub fn token(data: &[u8]) -> io::Result<(u64, Vec<Vec<u8>>)> {
    let v = decompose("hqqh", data, Default::default()).map_err(p)?;
    let Value::U64(key) = v[1] else {
        unreachable!()
    };
    Ok((
        key,
        vec![compose(
            "hq",
            &[
                Value::I16(LobbyCmd::L_GetEnterWorldTokenOk.id()),
                v[2].clone(),
            ],
        )
        .map_err(p)?],
    ))
}
pub fn characters(data: &[u8]) -> io::Result<(u64, Vec<Vec<u8>>)> {
    // Rust safety policy: reject malformed entries. Do not reproduce the C++
    // swallowed decoder errors or assertion paths (spec 05 §7.3.4).
    let mut input = data.to_vec();
    if input.len() >= 20 && i16::from_le_bytes(input[18..20].try_into().unwrap()) < 0 {
        input[18..20].fill(0);
    }
    let mut v = decode_fields(
        "qqhCqohsbqQQi;",
        &input,
        DecodeLimits {
            string_units: 33,
            blob_bytes: 1024,
            ..Default::default()
        },
    )
    .map_err(p)?;
    let Value::U64(key) = v.remove(0) else {
        unreachable!()
    };
    if let Value::Repeat(rows) = &mut v[2] {
        for row in rows {
            if let Value::String(text) = &mut row[3] {
                if let Some(end) = text.iter().position(|u| *u == 0) {
                    text.truncate(end);
                }
            }
        }
    }
    let mut fields = Vec::new();
    for (code, value) in ['q', 'h'].into_iter().zip(&v[..2]) {
        fields.push((
            compose(&code.to_string(), std::slice::from_ref(value)).map_err(p)?[2..].to_vec(),
            0,
        ));
    }
    let Value::Repeat(rows) = &v[2] else {
        unreachable!()
    };
    fields.push(((rows.len() as u16).to_le_bytes().to_vec(), 0));
    for row in rows {
        for (code, value) in "qohsbqQQi".chars().zip(row) {
            fields.push((
                compose(&code.to_string(), std::slice::from_ref(value)).map_err(p)?[2..].to_vec(),
                if code == 's' { 68 } else { 0 },
            ));
        }
    }
    let total: usize = fields.iter().map(|(f, _)| f.len()).sum();
    let mut out = vec![vec![0; 8]];
    for (field, reserve) in fields {
        if out.last().unwrap().len() + field.len().max(reserve) > 4000 {
            out.push(vec![0; 8]);
        }
        out.last_mut().unwrap().extend(field);
    }
    let last = out.len() - 1;
    for (i, packet) in out.iter_mut().enumerate() {
        let size = packet.len() as i16;
        packet[..2].copy_from_slice(&size.to_le_bytes());
        packet[2..4].copy_from_slice(&LobbyCmd::L_GetCharacterInfoListOk.id().to_le_bytes());
        packet[4..8].copy_from_slice(
            &(if i == 0 {
                total as i32
            } else if i == last {
                0
            } else {
                -1
            })
            .to_le_bytes(),
        );
    }
    Ok((key, out))
}
#[cfg(test)]
mod tests {
    use super::*;
    use packet::fragments::compose_fragments;
    #[test]
    fn token_omits_key_world_and_trailing_bytes() {
        let mut packet = compose(
            "hqqhi",
            &[
                Value::I16(25),
                Value::U64(9),
                Value::U64(42),
                Value::I16(123),
                Value::I32(55),
            ],
        )
        .unwrap();
        let (key, output) = token(&packet).unwrap();
        assert_eq!(key, 9);
        assert_eq!(
            decompose("hq", &output[0], Default::default()).unwrap(),
            vec![Value::I16(14), Value::U64(42)]
        );
        packet.truncate(5);
        assert!(token(&packet).is_err());
    }
    #[test]
    fn characters_empty_and_nul_and_byte_passthrough() {
        let row = vec![
            Value::U64(1),
            Value::Id([2; 16]),
            Value::I16(3),
            Value::String(vec![65, 0, 66]),
            Value::Bytes(vec![4; 1024]),
            Value::U64(5),
            Value::I8(-1),
            Value::I8(2),
            Value::I32(6),
        ];
        let packet = compose_fragments(
            36,
            "hiqqhCqohsbqQQi;",
            &[
                Value::U64(9),
                Value::U64(10),
                Value::I16(-1),
                Value::Repeat(vec![row]),
            ],
        )
        .unwrap();
        let (key, out) = characters(&packet[0][8..]).unwrap();
        assert_eq!(key, 9);
        let v = decode_fields("qhCqohsbqQQi;", &out[0][8..], Default::default()).unwrap();
        let Value::Repeat(rows) = &v[2] else { panic!() };
        assert_eq!(rows[0][3], Value::String(vec![65]));
        assert_eq!(rows[0][6..8], [Value::I8(-1), Value::I8(2)]);
        let empty = compose_fragments(
            36,
            "hiqqhCqohsbqQQi;",
            &[
                Value::U64(9),
                Value::U64(10),
                Value::I16(0),
                Value::Repeat(vec![]),
            ],
        )
        .unwrap();
        let (_, out) = characters(&empty[0][8..]).unwrap();
        assert_eq!(out[0].len(), 20);
    }
}
