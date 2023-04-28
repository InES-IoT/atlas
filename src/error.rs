//! Module containing a generic error type for the atlas crate.

use std::error::Error as StdError;
use std::fmt;

#[cfg(test)]
#[path = "./error_tests.rs"]
mod error_tests;

/// A list specifying general categories of atlas errors.
// Should this be marked as non-exhaustive?
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ErrorKind {
    /// The provided symbol is invalid (e.g. cannot be parsed, processed, ...).
    InvalidSymbol,
    /// The provided &str could not be matched to any of the possible enum
    /// values.
    InvalidEnumStr,
    /// Invoking the nm utility returned an error.
    Nm,
    /// Generic IO error.
    Io,
    /// The table could not be formatted (e.g. terminal width to small to fit
    /// all data, ...).
    TableFormat,
}

/// Error struct containing the kind of the error and an optional boxed trait
/// object to the original error that caused this atlas specific error.
// TODO:
// Check if boxing everything into an ErrorImpl struct would significantly
// increase performance. The size of the Error struct is currently 24 bytes and
// gets pushed around a lot during the analysis of the symbols.
// #[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    cause: Option<Box<dyn StdError + Send + Sync>>,
}

impl Error {
    /// Creates an error of a specific kind. Can be chained with [`Error::with`]
    /// to add an underlying cause.
    pub(crate) fn new(kind: ErrorKind) -> Self {
        Self { kind, cause: None }
    }

    /// Adds an underlying cause to an error.
    pub(crate) fn with<E>(mut self, error: E) -> Self
    where
        E: Into<Box<dyn StdError + Send + Sync>>,
    {
        self.cause = Some(error.into());
        self
    }

    /// Returns  the kind of the error.
    pub fn kind(&self) -> ErrorKind {
        self.kind
    }

    /// Consumes the error and returns the boxed trait object to the underlying
    /// cause. This can then be downcast to the original error type.
    /// ```
    /// # use std::io;
    /// # use atlas::Atlas;
    /// // Cause an error by trying to open non-existing files.
    /// let err = Atlas::new("/foo", "/bar").unwrap_err();
    /// let cause = err.into_cause().unwrap();
    /// let original_error = cause.downcast::<io::Error>().unwrap();
    /// assert_eq!(original_error.kind(), io::ErrorKind::NotFound);
    /// ```
    pub fn into_cause(self) -> Option<Box<dyn StdError + Send + Sync>> {
        self.cause
    }
}

impl StdError for Error {}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Atlas error (kind: {:?}, cause: {:?})",
            self.kind, self.cause
        )
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Atlas error (kind: {:?})", self.kind)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::new(ErrorKind::Io).with(e)
    }
}

impl From<std::convert::Infallible> for Error {
    /// See explanation in [`crate::sym::Symbol::from_rawsymbols`].
    fn from(_: std::convert::Infallible) -> Self {
        unreachable!()
    }
}
