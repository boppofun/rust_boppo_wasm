use std::{error::Error, fmt::Display};

#[repr(i32)]
#[derive(Debug, Clone, Copy)]
pub enum BoppoError {
    NotFound = 1,
    PermissionDenied = 2,
    InvalidPath = 3,
    InvalidParameter = 4,
    Unknown = 255,
}

impl Display for BoppoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BoppoError::InvalidPath => f.write_str("Invalid file path"),
            BoppoError::PermissionDenied => f.write_str("Permission denied"),
            BoppoError::NotFound => f.write_str("Resource not found"),
            BoppoError::InvalidParameter => f.write_str("Invalid audio parameter"),
            BoppoError::Unknown => f.write_str("Unknown Boppo error"),
        }
    }
}

impl Error for BoppoError {}

impl From<i32> for BoppoError {
    fn from(value: i32) -> Self {
        match value {
            1 => BoppoError::NotFound,
            2 => BoppoError::PermissionDenied,
            3 => BoppoError::InvalidPath,
            4 => BoppoError::InvalidParameter,
            _ => BoppoError::Unknown,
        }
    }
}

impl BoppoError {
    pub fn as_neg_i32(&self) -> i32 {
        -(*self as i32)
    }
}
