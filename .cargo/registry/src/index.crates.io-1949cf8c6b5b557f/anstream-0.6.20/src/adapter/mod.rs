//! Gracefully degrade styled output

mod strip;
mod wincon;

pub use strip::{
    strip_bytes,
    strip_str,
    StripBytes,
    StripBytesIter,
    StripStr,
    StripStrIter,
    StrippedBytes,
    StrippedStr,
};
pub use wincon::{
    WinconBytes,
    WinconBytesIter,
};
