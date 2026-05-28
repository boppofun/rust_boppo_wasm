use std::{error::Error as StdError, fmt::Display};

#[repr(i32)]
#[derive(Debug, Clone, Copy)]
pub enum Error {
    NotFound = 1,
    PermissionDenied = 2,
    InvalidPath = 3,
    InvalidParameter = 4,
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
    pub fn as_neg_i32(&self) -> i32 {
        -(*self as i32)
    }
}
