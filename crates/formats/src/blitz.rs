//! Blitz3D binary file primitives.
//!
//! Blitz's `ReadInt`, `ReadFloat` and `ReadShort` read little-endian values;
//! `ReadString` reads a `u32` byte length followed by that many bytes. Strings
//! are single-byte text, decoded here as Latin-1 so no byte is ever lost.
//!
//! Unlike Blitz, which returns zeros when reading past the end of a file,
//! these readers report [`Error::UnexpectedEof`]. [`BlitzWriter`] is the
//! inverse, for files the games write back (saves, options).

use crate::{Error, Result};

/// Sequential reader over a byte slice using Blitz3D's encodings.
#[derive(Debug, Clone)]
pub struct BlitzReader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> BlitzReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    /// Byte offset of the next read.
    pub fn position(&self) -> usize {
        self.pos
    }

    /// True once every byte has been read (Blitz's `Eof`).
    pub fn is_eof(&self) -> bool {
        self.pos >= self.data.len()
    }

    pub fn read_bytes(&mut self, len: usize) -> Result<&'a [u8]> {
        let end = self
            .pos
            .checked_add(len)
            .filter(|&end| end <= self.data.len())
            .ok_or(Error::UnexpectedEof {
                offset: self.pos,
                wanted: len,
            })?;
        let bytes = &self.data[self.pos..end];
        self.pos = end;
        Ok(bytes)
    }

    fn read_array<const N: usize>(&mut self) -> Result<[u8; N]> {
        Ok(self.read_bytes(N)?.try_into().expect("slice has length N"))
    }

    pub fn read_byte(&mut self) -> Result<u8> {
        Ok(self.read_array::<1>()?[0])
    }

    pub fn read_short(&mut self) -> Result<u16> {
        Ok(u16::from_le_bytes(self.read_array()?))
    }

    pub fn read_int(&mut self) -> Result<i32> {
        Ok(i32::from_le_bytes(self.read_array()?))
    }

    pub fn read_float(&mut self) -> Result<f32> {
        Ok(f32::from_le_bytes(self.read_array()?))
    }

    pub fn read_string(&mut self) -> Result<String> {
        let start = self.pos;
        let len = u32::from_le_bytes(self.read_array()?) as usize;
        let bytes = self.read_bytes(len).inspect_err(|_| self.pos = start)?;
        Ok(latin1(bytes))
    }
}

fn latin1(bytes: &[u8]) -> String {
    bytes.iter().map(|&b| char::from(b)).collect()
}

/// Builds a byte buffer using Blitz3D's encodings (`WriteInt`, `WriteString`…).
#[derive(Debug, Clone, Default)]
pub struct BlitzWriter {
    data: Vec<u8>,
}

impl BlitzWriter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.data
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) {
        self.data.extend_from_slice(bytes);
    }

    pub fn write_byte(&mut self, v: u8) {
        self.data.push(v);
    }

    pub fn write_short(&mut self, v: u16) {
        self.write_bytes(&v.to_le_bytes());
    }

    pub fn write_int(&mut self, v: i32) {
        self.write_bytes(&v.to_le_bytes());
    }

    pub fn write_float(&mut self, v: f32) {
        self.write_bytes(&v.to_le_bytes());
    }

    /// Writes a `u32` length and the Latin-1 bytes. Characters outside
    /// Latin-1 cannot be stored and become `?`.
    pub fn write_string(&mut self, s: &str) {
        let bytes: Vec<u8> = s.chars().map(|c| u8::try_from(c).unwrap_or(b'?')).collect();
        self.write_bytes(&(bytes.len() as u32).to_le_bytes());
        self.write_bytes(&bytes);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_little_endian_values() {
        let mut data = vec![];
        data.extend(7i32.to_le_bytes());
        data.extend((-2i32).to_le_bytes());
        data.extend(1.5f32.to_le_bytes());
        data.extend(0xBEEFu16.to_le_bytes());
        data.push(9);
        let mut r = BlitzReader::new(&data);
        assert_eq!(r.read_int().unwrap(), 7);
        assert_eq!(r.read_int().unwrap(), -2);
        assert_eq!(r.read_float().unwrap(), 1.5);
        assert_eq!(r.read_short().unwrap(), 0xBEEF);
        assert_eq!(r.read_byte().unwrap(), 9);
        assert!(r.is_eof());
    }

    #[test]
    fn reads_length_prefixed_latin1_strings() {
        let mut data = vec![];
        data.extend(4u32.to_le_bytes());
        data.extend([b'C', b'a', b'f', 0xE9]);
        data.extend(0u32.to_le_bytes());
        let mut r = BlitzReader::new(&data);
        assert_eq!(r.read_string().unwrap(), "Café");
        assert_eq!(r.read_string().unwrap(), "");
        assert!(r.is_eof());
    }

    #[test]
    fn truncated_string_is_an_error_and_does_not_advance() {
        let mut data = vec![];
        data.extend(10u32.to_le_bytes());
        data.extend(b"short");
        let mut r = BlitzReader::new(&data);
        assert!(matches!(
            r.read_string(),
            Err(Error::UnexpectedEof {
                offset: 4,
                wanted: 10
            })
        ));
        assert_eq!(r.position(), 0);
    }

    #[test]
    fn writer_round_trips_through_reader() {
        let mut w = BlitzWriter::new();
        w.write_int(-7);
        w.write_float(2.25);
        w.write_short(0xBEEF);
        w.write_byte(9);
        w.write_string("Café");
        w.write_string("Ω");
        let data = w.into_bytes();
        let mut r = BlitzReader::new(&data);
        assert_eq!(r.read_int().unwrap(), -7);
        assert_eq!(r.read_float().unwrap(), 2.25);
        assert_eq!(r.read_short().unwrap(), 0xBEEF);
        assert_eq!(r.read_byte().unwrap(), 9);
        assert_eq!(r.read_string().unwrap(), "Café");
        assert_eq!(r.read_string().unwrap(), "?");
        assert!(r.is_eof());
    }

    #[test]
    fn truncated_int_is_an_error() {
        let mut r = BlitzReader::new(&[1, 2, 3]);
        assert!(matches!(
            r.read_int(),
            Err(Error::UnexpectedEof {
                offset: 0,
                wanted: 4
            })
        ));
    }
}
