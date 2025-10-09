use thiserror::Error;

#[derive(Error, Debug)]
pub enum SpatioLiteError {
    #[error("Transaction is not writable")]
    TxNotWritable,

    #[error("Transaction is closed")]
    TxClosed,

    #[error("Key not found")]
    NotFound,

    #[error("Invalid operation")]
    Invalid,

    #[error("Database is closed")]
    DatabaseClosed,

    #[error("Index '{0}' already exists")]
    IndexExists(String),

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    #[error("Invalid sync policy")]
    InvalidSyncPolicy,

    #[error("Shrink operation in process")]
    ShrinkInProcess,

    #[error("Persistence is active")]
    PersistenceActive,

    #[error("Transaction is currently iterating")]
    TxIterating,

    #[error("Index '{0}' not found")]
    IndexNotFound(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Deserialization error: {0}")]
    Deserialization(String),

    #[error("Pattern matching error: {0}")]
    PatternMatch(String),

    #[error("Expired item")]
    Expired,

    #[error("Invalid key")]
    InvalidKey,

    #[error("Invalid value")]
    InvalidValue,

    #[error("Database corruption detected")]
    Corruption,

    #[error("Lock error: {0}")]
    Lock(String),
}

pub type Result<T> = std::result::Result<T, SpatioLiteError>;
