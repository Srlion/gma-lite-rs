//! Minimal GMA Library.
//!
//! Format:
//! - "GMAD" header (4 bytes)
//! - VERSION (int8) == 3
//! - steam_id64 (little-endian i64) [ignored]
//! - timestamp (little-endian u64) [ignored]
//! - required content (u8 = 0) [ignored]
//! - addon name (C string)
//! - addon description (C string)
//! - addon author (C string)
//! - addon version (little-endian i32) [ignored]
//! - Repeated file metadata entries until idx == 0:
//!     * idx (u32, 1-based; 0 terminates the list)
//!     * name (C string)
//!     * size (i64)
//!     * crc32 (u32) [ignored]
//! - File contents, concatenated in metadata order
//! - trailing u32 zero
//!

use std::fmt;
use std::io::{self};

/// Magic header for GMA files.
pub const HEADER: &[u8; 4] = b"GMAD";

/// File format version.
pub const VERSION: i8 = 3;

mod reader;
pub use reader::read;

mod builder;
pub use builder::Builder;

/// One entry (file) contained in a GMA.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct GMAFile {
    pub name: String,
    pub content: Vec<u8>,
    pub size: i64,
}

/// Errors that can occur while reading a GMA.
#[derive(Debug)]
pub enum GmaError {
    Io(io::Error),
    InvalidHeader([u8; 4]),
    InvalidVersion(i8),
    MissingNullTerminator, // for C-strings
    SizeOutOfRange(i64),
    TrailingMarkerMismatch(u32),
}

impl fmt::Display for GmaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GmaError::Io(e) => write!(f, "io error: {e}"),
            GmaError::InvalidHeader(got) => {
                write!(f, "invalid header: {:?}", String::from_utf8_lossy(got))
            }
            GmaError::InvalidVersion(v) => write!(f, "invalid version: {v}"),
            GmaError::MissingNullTerminator => write!(f, "missing null terminator in C string"),
            GmaError::SizeOutOfRange(sz) => write!(f, "negative or invalid size: {sz}"),
            GmaError::TrailingMarkerMismatch(v) => {
                write!(f, "expected trailing 0 u32 marker, got {v}")
            }
        }
    }
}

impl std::error::Error for GmaError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        if let GmaError::Io(e) = self {
            Some(e)
        } else {
            None
        }
    }
}

impl From<io::Error> for GmaError {
    fn from(e: io::Error) -> Self {
        GmaError::Io(e)
    }
}
