//! Field-aligned fragment writer and synchronous reader (spec 03 section 7).

use bloodlib::packet::{fields, packet_size, PacketError, Value, MAX_PACKET};
use std::borrow::Cow;
use thiserror::Error;

pub fn compose_fragments(
    cmd: i16,
    format: &str,
    values: &[Value],
) -> Result<Vec<Vec<u8>>, PacketError> {
    let format = format
        .strip_prefix("hi")
        .ok_or(PacketError::FragmentFormat)?;
    let fields = fields(format, values, MAX_PACKET - 8)?;
    let mut fragments = vec![vec![0; 8]];
    let mut total = 0usize;
    for field in fields {
        total = total
            .checked_add(field.len())
            .filter(|n| *n <= i32::MAX as usize)
            .ok_or(PacketError::ComposeOverflow)?;
        if fragments.last().unwrap().len() + field.len() > MAX_PACKET {
            fragments.push(vec![0; 8]);
        }
        fragments.last_mut().unwrap().extend(field);
    }
    let last = fragments.len() - 1;
    for (index, fragment) in fragments.iter_mut().enumerate() {
        let size = fragment.len() as i16;
        fragment[..2].copy_from_slice(&size.to_le_bytes());
        fragment[2..4].copy_from_slice(&cmd.to_le_bytes());
        let mark = if index == 0 {
            total as i32
        } else if index == last {
            0
        } else {
            -1
        };
        fragment[4..8].copy_from_slice(&mark.to_le_bytes());
    }
    Ok(fragments)
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum FragmentError {
    #[error(transparent)]
    Packet(#[from] PacketError),
    #[error("total < 0: {0}")]
    NegativeTotal(i32),
    #[error("Size = 0: continuation has no buffered data")]
    EmptyContinuation,
    #[error("add failed: command differs")]
    CommandMismatch,
    #[error("add failed: data exceeds total")]
    ExceedsTotal,
    #[error("Size != total: got {actual}, expected {total}")]
    IncompleteTotal { actual: usize, total: usize },
}

#[derive(Debug, PartialEq, Eq)]
pub struct Joined<'a> {
    pub cmd: i16,
    /// Single fragments borrow their input. Completed chains own their data.
    pub data: Cow<'a, [u8]>,
}

/// One reader per ordered stream. Errors leave its state unchanged. Clear it
/// before reusing a failed stream. Positive marks set the total, including when
/// data is already buffered; they do not discard that data (03 section 7).
#[derive(Default, Debug)]
pub struct FragmentReader {
    cmd: Option<i16>,
    total: usize,
    data: Vec<u8>,
}
impl FragmentReader {
    pub fn clear(&mut self) {
        *self = Self::default();
    }

    pub fn add<'a>(&mut self, packet: &'a [u8]) -> Result<Option<Joined<'a>>, FragmentError> {
        packet_size(packet, 8)?;
        let cmd = i16::from_le_bytes(packet[2..4].try_into().unwrap());
        let mark = i32::from_le_bytes(packet[4..8].try_into().unwrap());
        let data = &packet[8..];
        if mark < -1 {
            return Err(FragmentError::NegativeTotal(mark));
        }
        if self.cmd.is_some_and(|stored| stored != cmd) {
            return Err(FragmentError::CommandMismatch);
        }
        if mark == -1 && self.data.is_empty() {
            return Err(FragmentError::EmptyContinuation);
        }
        let total = if mark > 0 { mark as usize } else { self.total };
        let actual = self
            .data
            .len()
            .checked_add(data.len())
            .ok_or(FragmentError::ExceedsTotal)?;
        if actual > total {
            return Err(FragmentError::ExceedsTotal);
        }
        if mark == 0 && actual != total {
            return Err(FragmentError::IncompleteTotal { actual, total });
        }
        let single = mark > 0 && self.data.is_empty() && data.len() == total;
        if single {
            self.clear();
            return Ok(Some(Joined {
                cmd,
                data: Cow::Borrowed(data),
            }));
        }
        if mark == 0 {
            let mut joined = std::mem::take(&mut self.data);
            joined.extend_from_slice(data);
            self.clear();
            return Ok(Some(Joined {
                cmd,
                data: Cow::Owned(joined),
            }));
        }
        self.cmd = Some(cmd);
        self.total = total;
        self.data.extend_from_slice(data);
        Ok(None)
    }
}
