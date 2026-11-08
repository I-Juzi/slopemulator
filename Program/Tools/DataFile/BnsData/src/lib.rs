//! Binary data formats. The format contracts are in `spec/datafile/`.

pub mod dat;
pub mod lmx;
pub mod table;
pub mod tabledef;

use std::{fmt, io};

#[derive(Debug)]
pub enum Error {
    Invalid(&'static str),
    Io(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid(message) => f.write_str(message),
            Self::Io(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for Error {}
impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub(crate) struct Cursor<'a> {
    bytes: &'a [u8],
    pos: usize,
}
impl<'a> Cursor<'a> {
    pub(crate) fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, pos: 0 }
    }
    pub(crate) fn remaining(&self) -> usize {
        self.bytes.len() - self.pos
    }
    pub(crate) fn take(&mut self, size: usize) -> Result<&'a [u8]> {
        let end = self
            .pos
            .checked_add(size)
            .ok_or(Error::Invalid("size overflow"))?;
        let result = self
            .bytes
            .get(self.pos..end)
            .ok_or(Error::Invalid("truncated input"))?;
        self.pos = end;
        Ok(result)
    }
    pub(crate) fn u8(&mut self) -> Result<u8> {
        Ok(self.take(1)?[0])
    }
    pub(crate) fn u16(&mut self) -> Result<u16> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into().unwrap()))
    }
    pub(crate) fn u32(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    pub(crate) fn u64(&mut self) -> Result<u64> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }
    pub(crate) fn count(&self, count: u64, minimum_size: usize) -> Result<usize> {
        let count = usize::try_from(count).map_err(|_| Error::Invalid("count overflow"))?;
        if count > self.remaining() / minimum_size {
            return Err(Error::Invalid("count exceeds input"));
        }
        Ok(count)
    }
    pub(crate) fn finish(self) -> Result<()> {
        if self.remaining() != 0 {
            return Err(Error::Invalid("trailing bytes or size mismatch"));
        }
        Ok(())
    }
}
