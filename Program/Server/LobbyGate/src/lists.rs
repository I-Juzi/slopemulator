//! Shared broadcast caches. Spec 05 §§2.2–4.
use crate::wire::invalid;
use bloodlib::{
    crypto::SessionCrypt,
    fragments::compose_fragments,
    frames::encrypted_header,
    packet::{compose, decode_fields, DecodeLimits, Value},
};
use std::io;

pub const WORLD: &str = "ChhsQQiihhiQQQQQQQ;";
pub const SCHEDULE: &str = "QQqCqqiQiic;";
#[derive(Default)]
pub struct Lists {
    pub worlds: Option<Vec<Vec<u8>>>,
    pub schedule: Option<Vec<Vec<u8>>>,
}
fn p(e: impl std::fmt::Display) -> io::Error {
    invalid(&e.to_string())
}
pub fn broadcast_packets(packets: &[Vec<u8>], key: &[u8]) -> io::Result<Vec<Vec<u8>>> {
    let crypt = SessionCrypt::new(key[..16].try_into().map_err(p)?);
    packets
        .iter()
        .map(|packet| {
            let body = crypt.encrypt(packet, 8).map_err(p)?;
            let mut frame = encrypted_header(0xe, body.len())
                .map_err(p)?
                .to_le_bytes()
                .to_vec();
            frame.extend(body);
            Ok(frame)
        })
        .collect()
}
impl Lists {
    pub fn update(&mut self, data: &[u8], hello: bool, key: &[u8]) -> io::Result<()> {
        let v = decode_fields(
            &if hello {
                format!("{WORLD}{SCHEDULE}")
            } else {
                WORLD.to_owned()
            },
            data,
            DecodeLimits {
                string_units: 17,
                ..Default::default()
            },
        )
        .map_err(p)?;
        let Value::Repeat(rows) = &v[0] else {
            unreachable!()
        };
        let rows = rows
            .iter()
            .map(|row| {
                row.iter()
                    .enumerate()
                    .filter(|(i, _)| !matches!(i, 6 | 8))
                    .map(|(_, v)| v.clone())
                    .collect()
            })
            .collect();
        let worlds = broadcast_packets(
            &compose_fragments(25, "hiChhsQQihiQQQQQQQ;", &[Value::Repeat(rows)]).map_err(p)?,
            key,
        )?;
        let schedule = if hello {
            let mut values = v[1..].to_vec();
            for value in &mut values[..2] {
                let Value::I8(flag) = value else {
                    unreachable!()
                };
                *flag = i8::from(*flag == 1);
            }
            Some(broadcast_packets(
                &compose_fragments(104, "hiQQqCqqiQiic;", &values).map_err(p)?,
                key,
            )?)
        } else {
            None
        };
        self.worlds = Some(worlds);
        if hello {
            self.schedule = schedule;
        }
        Ok(())
    }
    pub fn announcement(data: &[u8], key: &[u8]) -> io::Result<Vec<Vec<u8>>> {
        let mut v =
            bloodlib::packet::decompose("hs", data, Default::default()).map_err(p)?;
        v[0] = Value::I16(11);
        broadcast_packets(&[compose("hs", &v).map_err(p)?], key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bloodlib::{fragments::FragmentReader, packet::decode_fields};
    fn plain(frame: &[u8]) -> Vec<u8> {
        assert_eq!(frame[1] >> 4, 0xe);
        let crypt = SessionCrypt::new([7; 16]);
        let b = crypt.decrypt(&frame[2..], 8).unwrap();
        b[..i16::from_le_bytes([b[0], b[1]]) as usize].to_vec()
    }
    #[test]
    fn hello_world_filter_flags_empty_and_update() {
        let row = vec![
            Value::I16(1),
            Value::I16(2),
            Value::String(vec![65]),
            Value::I8(3),
            Value::I8(4),
            Value::U32(5),
            Value::U32(6),
            Value::I16(7),
            Value::I16(8),
            Value::U32(9),
            Value::I8(10),
            Value::I8(11),
            Value::I8(12),
            Value::I8(13),
            Value::I8(14),
            Value::I8(15),
            Value::I8(16),
        ];
        let values = vec![
            Value::Repeat(vec![row.clone()]),
            Value::I8(2),
            Value::I8(1),
            Value::U64(123),
            Value::Repeat(vec![]),
        ];
        let input = compose_fragments(2, &format!("hi{WORLD}{SCHEDULE}"), &values).unwrap();
        let mut lists = Lists::default();
        lists.update(&input[0][8..], true, &[7; 17]).unwrap();
        let mut reader = FragmentReader::default();
        let packet = plain(&lists.worlds.as_ref().unwrap()[0]);
        let joined = reader.add(&packet).unwrap().unwrap();
        let got = decode_fields("ChhsQQihiQQQQQQQ;", &joined.data, Default::default()).unwrap();
        let expected: Vec<_> = row
            .into_iter()
            .enumerate()
            .filter(|(i, _)| !matches!(i, 6 | 8))
            .map(|(_, v)| v)
            .collect();
        assert_eq!(got, vec![Value::Repeat(vec![expected])]);
        let schedule = lists.schedule.clone();
        let packet = plain(&schedule.as_ref().unwrap()[0]);
        assert_eq!(
            decode_fields(SCHEDULE, &packet[8..], Default::default()).unwrap(),
            vec![
                Value::I8(0),
                Value::I8(1),
                Value::U64(123),
                Value::Repeat(vec![])
            ]
        );
        let empty = compose_fragments(4, &format!("hi{WORLD}"), &[Value::Repeat(vec![])]).unwrap();
        lists.update(&empty[0][8..], false, &[7; 17]).unwrap();
        assert_eq!(lists.schedule, schedule);
        let before = lists.worlds.clone();
        assert!(lists.update(&[], false, &[7; 17]).is_err());
        assert_eq!(lists.worlds, before);
    }
}
