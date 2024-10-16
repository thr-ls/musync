use reqwest::Error as ReqwestError;
use std::{fmt, io};

#[derive(Debug)]
pub enum CompareError {
    IoError(io::Error),
    JsonParseError(serde_json::Error),
    ApiError { code: i32, message: String },
    ReqwestError(ReqwestError),
    Other(String),
    DatabaseError(sled::Error),
}

impl fmt::Display for CompareError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CompareError::IoError(e) => write!(f, "IO error: {}", e),
            CompareError::JsonParseError(e) => write!(f, "JSON parse error: {}", e),
            CompareError::ApiError { code, message } => {
                write!(f, "API error ({}): {}", code, message)
            }
            CompareError::ReqwestError(e) => write!(f, "Reqwest error: {}", e),
            CompareError::Other(s) => write!(f, "Other error: {}", s),
            CompareError::DatabaseError(s) => write!(f, "Database error: {}", s),
        }
    }
}

impl From<io::Error> for CompareError {
    fn from(error: io::Error) -> Self {
        CompareError::IoError(error)
    }
}

impl From<serde_json::Error> for CompareError {
    fn from(error: serde_json::Error) -> Self {
        CompareError::JsonParseError(error)
    }
}

impl From<ReqwestError> for CompareError {
    fn from(error: ReqwestError) -> Self {
        CompareError::ReqwestError(error)
    }
}

impl From<sled::Error> for CompareError {
    fn from(err: sled::Error) -> Self {
        CompareError::DatabaseError(err)
    }
}
