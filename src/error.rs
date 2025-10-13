use std::fmt;

/// Simplified error types for Spatio
#[derive(Debug)]
pub enum SpatioError {
    /// Database is closed
    DatabaseClosed,
    /// Lock acquisition failed
    LockError,
    /// Invalid geohash
    InvalidGeohash,
    /// Serialization/deserialization error
    SerializationError,
    /// Serialization error with context
    SerializationErrorWithContext(String),
    /// I/O error from persistence layer
    Io(std::io::Error),
    /// Generic error with message
    Other(String),
}

impl fmt::Display for SpatioError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SpatioError::DatabaseClosed => write!(f, "Database is closed"),
            SpatioError::LockError => write!(f, "Failed to acquire lock"),
            SpatioError::InvalidGeohash => write!(f, "Invalid geohash"),
            SpatioError::SerializationError => write!(f, "Serialization error"),
            SpatioError::SerializationErrorWithContext(context) => {
                write!(f, "Serialization error: {}", context)
            }
            SpatioError::Io(err) => write!(f, "I/O error: {}", err),
            SpatioError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for SpatioError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            SpatioError::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for SpatioError {
    fn from(err: std::io::Error) -> Self {
        SpatioError::Io(err)
    }
}

/// Result type alias for Spatio operations
pub type Result<T> = std::result::Result<T, SpatioError>;
