//! Goods reply conversion. Spec 05 §8.6–8.10. No shop decisions occur here.
use crate::wire::invalid;
use bloodlib::packet::{compose, decode_fields, Value};
use packet_ids::LobbyCmd;
use std::io;

fn p(e: impl std::fmt::Display) -> io::Error {
    invalid(&e.to_string())
}

/// A field-aligned output builder. Fixed item rows can be one atomic field.
struct Fragments {
    cmd: i16,
    total: usize,
    packets: Vec<Vec<u8>>,
}
impl Fragments {
    fn new(cmd: i16) -> Self {
        Self {
            cmd,
            total: 0,
            packets: vec![vec![0; 8]],
        }
    }
    fn push(&mut self, bytes: &[u8]) -> io::Result<()> {
        if bytes.len() > 3992 {
            return Err(invalid("goods field exceeds fragment capacity"));
        }
        self.total += bytes.len();
        if self.packets.last().unwrap().len() + bytes.len() > 4000 {
            self.packets.push(vec![0; 8]);
        }
        self.packets.last_mut().unwrap().extend_from_slice(bytes);
        Ok(())
    }
    fn value(&mut self, code: char, value: &Value) -> io::Result<()> {
        let bytes = compose(&code.to_string(), std::slice::from_ref(value)).map_err(p)?;
        self.push(&bytes[2..])
    }
    fn finish(mut self) -> Vec<Vec<u8>> {
        let last = self.packets.len() - 1;
        for (i, packet) in self.packets.iter_mut().enumerate() {
            let size = packet.len() as i16;
            packet[..2].copy_from_slice(&size.to_le_bytes());
            packet[2..4].copy_from_slice(&self.cmd.to_le_bytes());
            let mark = if i == 0 {
                self.total as i32
            } else if i == last {
                0
            } else {
                -1
            };
            packet[4..8].copy_from_slice(&mark.to_le_bytes());
        }
        self.packets
    }
}

struct Input<'a>(&'a [u8]);
impl Input<'_> {
    fn values(&mut self, format: &str) -> io::Result<Vec<Value>> {
        let mut values = Vec::new();
        for code in format.chars() {
            let format = code.to_string();
            let mut field = decode_fields(&format, self.0, Default::default()).map_err(p)?;
            let used = compose(&format, &field).map_err(p)?.len() - 2;
            self.0 = &self.0[used..];
            values.push(field.remove(0));
        }
        Ok(values)
    }
    fn count(&mut self) -> io::Result<usize> {
        let values = self.values("h")?;
        let Value::I16(count) = values[0] else {
            unreachable!()
        };
        Ok(count.max(0) as usize)
    }
    fn key(&mut self) -> io::Result<u64> {
        let values = self.values("q")?;
        let Value::U64(key) = values[0] else {
            unreachable!()
        };
        Ok(key)
    }
}
pub fn service_entries(data: &[u8]) -> io::Result<(u64, Vec<Vec<u8>>)> {
    fixed_items(data, LobbyCmd::L_GetServiceEntryListOk.id(), "", "qQqohq")
}
pub fn service_items(data: &[u8]) -> io::Result<(u64, Vec<Vec<u8>>)> {
    fixed_items(
        data,
        LobbyCmd::L_ListLobbyServiceItemInfoOk.id(),
        "Qi",
        "qqqiQ",
    )
}
pub fn take_goods(data: &[u8]) -> io::Result<(u64, Vec<Vec<u8>>)> {
    fixed_items(data, LobbyCmd::L_TakeGoodsOk.id(), "q", "qqqiQ")
}
fn fixed_items(
    data: &[u8],
    cmd: i16,
    header_format: &str,
    row_format: &str,
) -> io::Result<(u64, Vec<Vec<u8>>)> {
    let mut input = Input(data);
    let key = input.key()?;
    let header = input.values(header_format)?;
    let count = input.count()?;
    let mut out = Fragments::new(cmd);
    for (code, value) in header_format.chars().zip(&header) {
        out.value(code, value)?;
    }
    out.value('h', &Value::U16(count as u16))?;
    for _ in 0..count {
        let row = input.values(row_format)?;
        out.push(&compose(row_format, &row).map_err(p)?[2..])?;
    }
    Ok((key, out.finish()))
}
pub fn list_goods(data: &[u8]) -> io::Result<(u64, Vec<Vec<u8>>)> {
    let mut input = Input(data);
    let key = input.key()?;
    let count = input.count()?;
    let mut rows = Vec::new();
    for _ in 0..count {
        let mut row = input.values("qiQQqqssissbb")?;
        let count = input.count()?;
        let mut items = Vec::new();
        for _ in 0..count {
            items.push(input.values("iiibb")?);
        }
        row.push(Value::Repeat(items));
        rows.push(row);
    }
    let mut out = Fragments::new(LobbyCmd::L_ListGoodsOk.id());
    out.value('h', &Value::U16(rows.len() as u16))?;
    for row in &rows {
        for (index, (code, value)) in "qiQQqqssissbb".chars().zip(row).enumerate() {
            if let Value::String(text) = value {
                let capacity = match index {
                    6 => 51,
                    7 | 9 => 81,
                    10 => 201,
                    _ => unreachable!(),
                };
                if text.len() >= capacity {
                    return Err(invalid("goods string exceeds destination"));
                }
                let end = text.iter().position(|u| *u == 0).unwrap_or(text.len());
                out.value(code, &Value::String(text[..end].to_vec()))?;
            } else {
                out.value(code, value)?;
            }
        }
        let Value::Repeat(items) = &row[13] else {
            unreachable!()
        };
        out.value('h', &Value::U16(items.len() as u16))?;
        for item in items {
            for (code, value) in "iiibb".chars().zip(item) {
                out.value(code, value)?;
            }
        }
    }
    Ok((key, out.finish()))
}
