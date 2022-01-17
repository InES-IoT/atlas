use std::error::Error as StdError;
use std::fmt;

#[cfg(test)]
#[path = "./error_tests.rs"]
mod error_tests;

// Should this be marked as non-exhaustive?
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ErrorKind {
    InvalidSymbol,
    InvalidEnumStr,
    Nm,
    Io,
    TableFormat,
}

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
    pub(crate) fn new(kind: ErrorKind) -> Self {
        Self { kind, cause: None }
    }

    pub(crate) fn with<E>(mut self, error: E) -> Self
    where
        E: Into<Box<dyn StdError + Send + Sync>>,
    {
        self.cause = Some(error.into());
        self
    }

    pub fn kind(&self) -> ErrorKind {
        self.kind
    }

    pub fn into_cause(self) -> Option<Box<dyn StdError + Send + Sync>> {
        self.cause
    }
}

impl StdError for Error {}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Atlas error (kind: {:?}, cause: {:?})", self.kind, self.cause)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Atlas error (kind: {:?})", self.kind)
    }
}
