//! The game data pack: `res1.dat` (index) and `res2.dat` (data).
//!
//! `res1.dat` is a sequence of entries, each a Blitz string (the file's path,
//! e.g. `graphics\rainbow.jpg`) followed by two ints: the byte offset and
//! length of that file inside `res2.dat`. There is no header or count; the
//! index runs to the end of the file.
//!
//! The original games lowercased names on load and took the first match when
//! looking a file up, so lookups here are case-insensitive and the first entry
//! wins if a name repeats.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::blitz::BlitzReader;
use crate::error::io_error;
use crate::{Error, Result};

pub const INDEX_FILE: &str = "res1.dat";
pub const DATA_FILE: &str = "res2.dat";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackEntry {
    /// Path as stored in the index (original case, `\` separators).
    pub name: String,
    pub offset: u32,
    pub len: u32,
}

impl PackEntry {
    fn range(&self) -> std::ops::Range<usize> {
        let start = self.offset as usize;
        start..start + self.len as usize
    }
}

#[derive(Debug, Clone, Default)]
pub struct PackIndex {
    entries: Vec<PackEntry>,
    by_name: HashMap<String, usize>,
}

impl PackIndex {
    pub fn parse(data: &[u8]) -> Result<Self> {
        let mut reader = BlitzReader::new(data);
        let mut entries = Vec::new();
        let mut by_name = HashMap::new();
        while !reader.is_eof() {
            let at = reader.position();
            let name = reader.read_string()?;
            let offset = reader.read_int()?;
            let len = reader.read_int()?;
            let (Ok(offset), Ok(len)) = (u32::try_from(offset), u32::try_from(len)) else {
                return Err(Error::Invalid(format!(
                    "pack entry {name:?} at index offset {at} has negative offset or length"
                )));
            };
            by_name.entry(normalize(&name)).or_insert(entries.len());
            entries.push(PackEntry { name, offset, len });
        }
        Ok(Self { entries, by_name })
    }

    /// All entries in index order, including any repeated names.
    pub fn entries(&self) -> &[PackEntry] {
        &self.entries
    }

    /// Looks up a file by path, ignoring case and accepting `/` or `\`.
    pub fn get(&self, name: &str) -> Option<&PackEntry> {
        self.by_name
            .get(&normalize(name))
            .map(|&i| &self.entries[i])
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// A pack loaded into memory.
#[derive(Debug, Clone)]
pub struct Pack {
    index: PackIndex,
    data: Vec<u8>,
}

impl Pack {
    /// Loads `res1.dat` and `res2.dat` from a game folder.
    pub fn open(dir: impl AsRef<Path>) -> Result<Self> {
        let dir = dir.as_ref();
        let index_path = dir.join(INDEX_FILE);
        let data_path = dir.join(DATA_FILE);
        let index = fs::read(&index_path).map_err(io_error(&index_path))?;
        let data = fs::read(&data_path).map_err(io_error(&data_path))?;
        Self::from_bytes(&index, data)
    }

    /// Builds a pack from index bytes and data, checking every entry lies
    /// within the data.
    pub fn from_bytes(index: &[u8], data: Vec<u8>) -> Result<Self> {
        let index = PackIndex::parse(index)?;
        if let Some(bad) = index.entries().iter().find(|e| e.range().end > data.len()) {
            return Err(Error::Invalid(format!(
                "pack entry {:?} (offset {}, length {}) extends past the end of {DATA_FILE} ({} bytes)",
                bad.name,
                bad.offset,
                bad.len,
                data.len()
            )));
        }
        Ok(Self { index, data })
    }

    pub fn index(&self) -> &PackIndex {
        &self.index
    }

    /// Contents of a packed file, looked up as in [`PackIndex::get`].
    pub fn read(&self, name: &str) -> Option<&[u8]> {
        self.index.get(name).map(|e| self.entry_data(e))
    }

    pub fn entry_data(&self, entry: &PackEntry) -> &[u8] {
        &self.data[entry.range()]
    }
}

/// Canonical form of a game path: lowercase with `\` separators.
pub(crate) fn normalize(name: &str) -> String {
    name.replace('/', "\\").to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn index_bytes(entries: &[(&str, i32, i32)]) -> Vec<u8> {
        let mut out = vec![];
        for &(name, offset, len) in entries {
            out.extend((name.len() as u32).to_le_bytes());
            out.extend(name.as_bytes());
            out.extend(offset.to_le_bytes());
            out.extend(len.to_le_bytes());
        }
        out
    }

    #[test]
    fn reads_entries_case_insensitively() {
        let index = index_bytes(&[("graphics\\A.jpg", 0, 3), ("levels\\b.lev", 3, 2)]);
        let pack = Pack::from_bytes(&index, b"abcde".to_vec()).unwrap();
        assert_eq!(pack.index().len(), 2);
        assert_eq!(pack.read("graphics\\a.JPG"), Some(&b"abc"[..]));
        assert_eq!(pack.read("levels/b.lev"), Some(&b"de"[..]));
        assert_eq!(pack.read("missing"), None);
    }

    #[test]
    fn first_entry_wins_for_repeated_names() {
        let index = index_bytes(&[("x", 0, 1), ("X", 1, 1)]);
        let pack = Pack::from_bytes(&index, b"12".to_vec()).unwrap();
        assert_eq!(pack.read("x"), Some(&b"1"[..]));
        assert_eq!(pack.index().entries().len(), 2);
    }

    #[test]
    fn rejects_entries_outside_data() {
        let index = index_bytes(&[("x", 2, 5)]);
        assert!(matches!(
            Pack::from_bytes(&index, vec![0; 4]),
            Err(Error::Invalid(_))
        ));
    }

    #[test]
    fn rejects_negative_offsets() {
        let index = index_bytes(&[("x", -1, 1)]);
        assert!(matches!(PackIndex::parse(&index), Err(Error::Invalid(_))));
    }

    #[test]
    fn rejects_truncated_index() {
        let mut index = index_bytes(&[("x", 0, 1)]);
        index.pop();
        assert!(matches!(
            PackIndex::parse(&index),
            Err(Error::UnexpectedEof { .. })
        ));
    }
}
