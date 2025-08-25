use std::io::{self, BufRead, BufReader, Read};

use crate::{GMAFile, GmaError, HEADER, VERSION};

/// Read a GMA from any `Read`. Returns the list of entries with names and contents.
pub fn read<R: Read>(reader: R) -> Result<Vec<GMAFile>, GmaError> {
    let mut r = BufReader::new(reader);

    // Header
    let mut hdr = [0u8; 4];
    r.read_exact(&mut hdr)?;
    if &hdr != HEADER {
        return Err(GmaError::InvalidHeader(hdr));
    }

    // Version (int8)
    let v = read_i8(&mut r)?;
    if v != VERSION {
        return Err(GmaError::InvalidVersion(v));
    }

    // SteamID64 (i64) — discard
    discard_exact(&mut r, 8)?;

    // Timestamp (u64) — discard
    discard_exact(&mut r, 8)?;

    // Required content (u8) — discard
    discard_exact(&mut r, 1)?;

    // Addon name / description / author — discard their values but still parse
    read_c_string(&mut r)?; // name
    read_c_string(&mut r)?; // description
    read_c_string(&mut r)?; // author

    // Addon version (i32) — discard
    discard_exact(&mut r, 4)?;

    // Metadata loop
    let mut entries_meta: Vec<(String, u64)> = Vec::with_capacity(10);
    loop {
        let idx = read_u32(&mut r)?;
        if idx == 0 {
            break;
        }

        let name = read_c_string(&mut r)?;
        let size_i64 = read_i64(&mut r)?;
        if size_i64 < 0 {
            return Err(GmaError::SizeOutOfRange(size_i64));
        }
        let size = size_i64 as u64;

        // CRC32 (u32) — discard
        discard_exact(&mut r, 4)?;

        entries_meta.push((name, size));
    }

    // Contents — read in the same order
    let mut entries = Vec::with_capacity(entries_meta.len());
    for (name, size) in entries_meta {
        let mut content = vec![0u8; size as usize];
        r.read_exact(&mut content)?;
        entries.push(GMAFile {
            name,
            size,
            content,
        });
    }

    // Final trailing u32 zero
    let trailing = read_u32(&mut r)?;
    if trailing != 0 {
        return Err(GmaError::TrailingMarkerMismatch(trailing));
    }

    Ok(entries)
}

fn discard_exact<R: Read>(r: &mut R, n: u64) -> Result<(), GmaError> {
    let copied = io::copy(&mut r.take(n), &mut io::sink())?;
    if copied == n {
        Ok(())
    } else {
        Err(io::Error::from(io::ErrorKind::UnexpectedEof).into())
    }
}

fn read_i8<R: Read>(r: &mut R) -> Result<i8, GmaError> {
    let mut b = [0u8; 1];
    r.read_exact(&mut b)?;
    Ok(i8::from_le_bytes(b))
}

fn read_i64<R: Read>(r: &mut R) -> Result<i64, GmaError> {
    let mut b = [0u8; 8];
    r.read_exact(&mut b)?;
    Ok(i64::from_le_bytes(b))
}

fn read_u32<R: Read>(r: &mut R) -> Result<u32, GmaError> {
    let mut b = [0u8; 4];
    r.read_exact(&mut b)?;
    Ok(u32::from_le_bytes(b))
}

fn read_c_string<R: BufRead>(r: &mut R) -> Result<String, GmaError> {
    let mut buf = Vec::with_capacity(32);
    let n = r.read_until(0, &mut buf)?; // includes the 0 delimiter if found
    if n == 0 || *buf.last().unwrap_or(&1) != 0 {
        // EOF or no null terminator found
        return Err(GmaError::MissingNullTerminator);
    }
    buf.pop(); // drop the '\0'
    // Per writer, strings shouldn't contain interior nulls; if present, they'd have truncated here.
    Ok(String::from_utf8_lossy(&buf).into_owned())
}
