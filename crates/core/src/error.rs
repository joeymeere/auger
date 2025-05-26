use thiserror::Error;

#[derive(Error, Debug)]
pub enum AugerError {
    #[error("Failed to read file: {0}")]
    FileReadError(#[from] std::io::Error),
    #[error("Failed to parse program: {0}")]
    ProgramParseError(String),
    #[error("Not enough program headers")]
    NotEnoughProgramHeaders,
    #[error("No text was extracted")]
    NoTextExtracted,
    #[error("Invalid file extension")]
    InvalidFileExtension,
    #[error("Failed to serialize to JSON: {0}")]
    SerializationError(#[from] serde_json::Error),
}