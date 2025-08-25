use std::io::{self, BufWriter, Write};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{Entry, GmaError, HEADER, VERSION};

/// Builder for writing `.gma` archives.
///
/// Collects metadata + entries
/// and writes them to a writer with `write_to`.
pub struct Builder {
    name: String,
    steam_id64: i64,
    author: String,
    description: String,
    entries: Vec<Entry>,
}

impl Builder {
    pub fn new(name: impl Into<String>, steam_id64: i64) -> Self {
        Self {
            name: name.into(),
            steam_id64,
            author: "unknown".into(),
            description: String::new(),
            entries: Vec::new(),
        }
    }

    pub fn set_description(&mut self, desc: impl Into<String>) {
        self.description = desc.into();
    }

    pub fn set_author(&mut self, author: impl Into<String>) {
        self.author = author.into();
    }

    pub fn file_from_bytes(&mut self, name: impl Into<String>, bytes: Vec<u8>) {
        let name = name.into();
        let size = bytes.len() as u64;
        self.entries.push(Entry {
            name,
            content: bytes,
            size,
        });
    }

    pub fn file_from_string(&mut self, name: impl Into<String>, content: impl Into<String>) {
        self.file_from_bytes(name, content.into().into_bytes());
    }

    /// Write the archive to a writer.
    pub fn write_to<W: Write>(&self, mut w: W) -> Result<(), GmaError> {
        let mut bw = BufWriter::new(&mut w);

        // Header
        bw.write_all(HEADER)?;

        // Version
        bw.write_all(&VERSION.to_le_bytes())?;

        // SteamID64
        bw.write_all(&self.steam_id64.to_le_bytes())?;

        // Timestamp
        let unix_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        bw.write_all(&unix_time.to_le_bytes())?;

        // Required content (unused)
        bw.write_all(&[0u8])?;

        // Addon strings
        write_cstring(&mut bw, &self.name)?;
        write_cstring(&mut bw, &self.description)?;
        write_cstring(&mut bw, &self.author)?;

        // Version (unused, int32 = 1)
        bw.write_all(&1i32.to_le_bytes())?;

        // Metadata for each file entry
        for (i, e) in self.entries.iter().enumerate() {
            // File index (1-based)
            bw.write_all(&(i as u32 + 1).to_le_bytes())?;
            // Name
            write_cstring(&mut bw, &e.name)?;
            // Size (int64)
            bw.write_all(&(e.content.len() as i64).to_le_bytes())?;
            // CRC (unused, write 0)
            bw.write_all(&0u32.to_le_bytes())?;
        }

        // End of metadata
        bw.write_all(&0u32.to_le_bytes())?;

        // File contents
        for e in &self.entries {
            bw.write_all(&e.content)?;
        }

        // End of file marker
        bw.write_all(&0u32.to_le_bytes())?;

        bw.flush()?;
        Ok(())
    }
}

fn write_cstring<W: Write>(mut w: W, s: &str) -> Result<(), GmaError> {
    if s.bytes().any(|b| b == 0) {
        return Err(
            io::Error::new(io::ErrorKind::InvalidInput, "string contains null byte").into(),
        );
    }
    w.write_all(s.as_bytes())?;
    w.write_all(&[0u8])?;
    Ok(())
}
