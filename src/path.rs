// Copyright 2024 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::dir_entry::DirEntryName;
use crate::format::format_bytes_debug;
use alloc::vec::Vec;
use core::fmt::{self, Debug, Formatter};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PathError {
    /// Path contains a component longer than 255 bytes.
    ComponentTooLong,

    /// Path contains a null byte.
    ContainsNull,
}

/// Reference path type.
///
/// Paths are mostly arbitrary sequences of bytes, with two restrictions:
/// * The path cannot contain any null bytes.
/// * Each component of the path must be no longer than 255 bytes.
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Path<'a>(
    // Use `&[u8]` rather than `[u8]` so that we don't have to use any
    // unsafe code. Unfortunately that means we can't impl `Deref` to
    // convert from `PathBuf` to `Path`.
    &'a [u8],
);

impl<'a> Path<'a> {
    pub const SEPARATOR: u8 = b'/';

    /// Root path, equivalent to `/`.
    pub const ROOT: Path<'static> = Path(&[Self::SEPARATOR]);

    /// Create a new `Path`.
    ///
    /// This panics if the input is invalid, use [`Path::try_from`] if
    /// error handling is desired.
    ///
    /// # Panics
    ///
    /// Panics if the path contains any null bytes or if a component of
    /// the path is longer than 255 bytes.
    #[track_caller]
    pub fn new<P>(p: &'a P) -> Self
    where
        P: AsRef<[u8]> + ?Sized,
    {
        Self::try_from(p.as_ref()).unwrap()
    }
}

impl<'a> Debug for Path<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        format_bytes_debug(self.0, f)
    }
}

impl<'a> TryFrom<&'a str> for Path<'a> {
    type Error = PathError;

    fn try_from(s: &'a str) -> Result<Self, PathError> {
        Self::try_from(s.as_bytes())
    }
}

impl<'a> TryFrom<&'a [u8]> for Path<'a> {
    type Error = PathError;

    fn try_from(s: &'a [u8]) -> Result<Self, PathError> {
        if s.contains(&0) {
            return Err(PathError::ContainsNull);
        }

        for component in s.split(|b| *b == Path::SEPARATOR) {
            if component.len() > DirEntryName::MAX_LEN {
                return Err(PathError::ComponentTooLong);
            }
        }

        Ok(Self(s))
    }
}

impl<'a, const N: usize> TryFrom<&'a [u8; N]> for Path<'a> {
    type Error = PathError;

    fn try_from(a: &'a [u8; N]) -> Result<Self, PathError> {
        Self::try_from(a.as_slice())
    }
}

#[cfg(all(feature = "std", unix))]
impl<'a> From<Path<'a>> for &'a std::path::Path {
    fn from(p: Path<'a>) -> &'a std::path::Path {
        use std::os::unix::ffi::OsStrExt;

        let s = std::ffi::OsStr::from_bytes(p.0);
        std::path::Path::new(s)
    }
}

/// Owned path type.
///
/// Paths are mostly arbitrary sequences of bytes, with two restrictions:
/// * The path cannot contain any null bytes.
/// * Each component of the path must be no longer than 255 bytes.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct PathBuf(Vec<u8>);

impl PathBuf {
    /// Create a new `PathBuf`.
    ///
    /// This panics if the input is invalid, use [`Path::try_from`] if
    /// error handling is desired.
    ///
    /// # Panics
    ///
    /// Panics if the path contains any null bytes or if a component of
    /// the path is longer than 255 bytes.
    #[track_caller]
    pub fn new<P>(p: &P) -> Self
    where
        P: AsRef<[u8]> + ?Sized,
    {
        Self::try_from(p.as_ref()).unwrap()
    }

    /// Borrow as a `Path`.
    pub fn as_path(&self) -> Path {
        Path(&self.0)
    }
}

impl Debug for PathBuf {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.as_path().fmt(f)
    }
}

impl TryFrom<&str> for PathBuf {
    type Error = PathError;

    fn try_from(s: &str) -> Result<Self, PathError> {
        Self::try_from(s.as_bytes().to_vec())
    }
}

impl TryFrom<&[u8]> for PathBuf {
    type Error = PathError;

    fn try_from(s: &[u8]) -> Result<Self, PathError> {
        Self::try_from(s.to_vec())
    }
}

impl<const N: usize> TryFrom<&[u8; N]> for PathBuf {
    type Error = PathError;

    fn try_from(a: &[u8; N]) -> Result<Self, PathError> {
        Self::try_from(a.as_slice().to_vec())
    }
}

impl TryFrom<Vec<u8>> for PathBuf {
    type Error = PathError;

    fn try_from(s: Vec<u8>) -> Result<Self, PathError> {
        // Validate the input.
        Path::try_from(s.as_slice())?;

        Ok(Self(s))
    }
}

#[cfg(all(feature = "std", unix))]
impl From<PathBuf> for std::path::PathBuf {
    fn from(p: PathBuf) -> std::path::PathBuf {
        use std::os::unix::ffi::OsStringExt;

        let s = std::ffi::OsString::from_vec(p.0);
        std::path::PathBuf::from(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_construction() {
        let expected_path = Path(b"abc");
        let expected_path_buf = PathBuf(b"abc".as_slice().to_vec());

        // Successful construction from a string.
        let src: &str = "abc";
        assert_eq!(Path::try_from(src).unwrap(), expected_path);
        assert_eq!(Path::new(src), expected_path);
        assert_eq!(PathBuf::try_from(src).unwrap(), expected_path_buf);
        assert_eq!(PathBuf::new(src), expected_path_buf);

        // Successful construction from a byte slice.
        let src: &[u8] = b"abc";
        assert_eq!(Path::try_from(src).unwrap(), expected_path);
        assert_eq!(Path::new(src), expected_path);
        assert_eq!(PathBuf::try_from(src).unwrap(), expected_path_buf);
        assert_eq!(PathBuf::new(src), expected_path_buf);

        // Successful construction from a byte array.
        let src: &[u8; 3] = b"abc";
        assert_eq!(Path::try_from(src).unwrap(), expected_path);
        assert_eq!(Path::new(src), expected_path);
        assert_eq!(PathBuf::try_from(src).unwrap(), expected_path_buf);
        assert_eq!(PathBuf::new(src), expected_path_buf);

        // Successful construction from a vector (only for PathBuf).
        let src: Vec<u8> = b"abc".to_vec();
        assert_eq!(PathBuf::try_from(src).unwrap(), expected_path_buf);

        // Error: contains null.
        let src: &str = "\0";
        assert_eq!(Path::try_from(src), Err(PathError::ContainsNull));
        assert_eq!(PathBuf::try_from(src), Err(PathError::ContainsNull));

        // Error: invalid component (too long).
        let src = &[b'a'; 256];
        assert_eq!(Path::try_from(src), Err(PathError::ComponentTooLong));
        assert_eq!(PathBuf::try_from(src), Err(PathError::ComponentTooLong));
    }

    #[test]
    fn test_path_debug() {
        let src = "abc😁\n".as_bytes();
        let expected = "abc😁\\n"; // Note the escaped slash.
        assert_eq!(format!("{:?}", Path(src)), expected);
        assert_eq!(format!("{:?}", PathBuf(src.to_vec())), expected);
    }

    #[cfg(all(feature = "std", unix))]
    #[test]
    fn test_to_std() {
        let p = Path(b"abc");
        assert_eq!(<&std::path::Path>::from(p), std::path::Path::new("abc"));

        let p = PathBuf(b"abc".to_vec());
        assert_eq!(
            std::path::PathBuf::from(p),
            std::path::PathBuf::from("abc")
        );
    }
}
