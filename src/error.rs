use std::{error::Error as StdError, fmt::Display};

/// Error type returned by Boppo host API calls.
#[non_exhaustive]
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// The resource was not found.
    NotFound = 1,
    /// The operation was denied due to insufficient permissions.
    PermissionDenied = 2,
    /// The provided path was malformed or invalid.
    InvalidPath = 3,
    /// One or more parameters were out of range or otherwise invalid.
    InvalidParameter = 4,
    /// An unexpected error occurred.
    Unknown = 255,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidPath => f.write_str("invalid path"),
            Error::PermissionDenied => f.write_str("permission denied"),
            Error::NotFound => f.write_str("not found"),
            Error::InvalidParameter => f.write_str("invalid parameter"),
            Error::Unknown => f.write_str("unknown error"),
        }
    }
}

impl StdError for Error {}

impl From<i32> for Error {
    fn from(value: i32) -> Self {
        match value {
            1 => Error::NotFound,
            2 => Error::PermissionDenied,
            3 => Error::InvalidPath,
            4 => Error::InvalidParameter,
            _ => Error::Unknown,
        }
    }
}

impl Error {
    /// Encode this error as a negative i32 for host API return values.
    #[must_use]
    pub fn as_neg_i32(&self) -> i32 {
        -(*self as i32)
    }

    /// Convert a host API return value into `Ok(n)` or `Err(Error)`.
    ///
    /// Negative values are treated as errors; non-negative values are returned as-is.
    pub fn result_from_i32(n: i32) -> Result<i32, Self> {
        if n < 0 { Err(Error::from(-n)) } else { Ok(n) }
    }
}
