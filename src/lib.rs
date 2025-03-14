use std::path::Path;

use anyhow::Result;
use thiserror::Error;

pub mod consts;
pub mod hash;
pub mod model;
pub mod parser;
pub mod writer;
pub mod utils;

pub use model::{ExtractConfig, ExtractResult, ExtractStats, Instruction, SourceFile};
pub use parser::{BpfParser, ProgramParser, AnchorProgramParser, NativeProgramParser, ProgramType};
pub use writer::FileWriter;

#[derive(Error, Debug)]
pub enum ExtractError {
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

/// Extracts valid text from an sBPF binary, and attempts to match instruction names
pub fn extract_from_file(file_path: &Path, config: Option<ExtractConfig>) -> Result<ExtractResult, ExtractError> {
    let config = config.unwrap_or_default();

    if file_path.extension().unwrap() != "so" {
        return Err(ExtractError::InvalidFileExtension);
    }

    let file_bytes = std::fs::read(file_path)?;
    parser::extract_from_bytes(&file_bytes, config)
}

/// Extracts valid text from an sBPF binary using custom parsers
pub fn extract_from_file_with_parsers(
    file_path: &Path, 
    config: Option<ExtractConfig>,
    parsers: Vec<Box<dyn ProgramParser>>
) -> Result<ExtractResult, ExtractError> {
    let config = config.unwrap_or_default();

    if file_path.extension().unwrap() != "so" {
        return Err(ExtractError::InvalidFileExtension);
    }

    let file_bytes = std::fs::read(file_path)?;
    parser::extract_from_bytes_with_parsers(&file_bytes, config, parsers)
}

/// Dumps the ELF metadata to a JSON file
pub fn dump_elf_meta(file_bytes: &[u8], base_path: &Path) -> Result<(), ExtractError> {
    writer::dump_elf_meta(file_bytes, base_path)
}

/// Writes extraction results to files
pub fn write_results(result: &ExtractResult, base_path: &Path) -> Result<(), ExtractError> {
    writer::write_results(result, base_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_extract_from_file() {
        let so_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("spaceman.so");
        
        let result = extract_from_file(&so_path, None).unwrap();
        
        println!("Starting extraction from offset: {}", result.stats.start_offset);
        println!("Extraction ended at position: {}", result.stats.end_position);
        println!("Total bytes processed: {}", result.stats.bytes_processed);
        
        println!("\nFound {} unique instructions:", result.instructions.len());
        for instruction in &result.instructions {
            println!("- {}", instruction);
        }
        
        println!("\nFound {} source files:", result.files.len());
        for file in &result.files {
            println!("- {} (project: {})", file.path, file.project);
        }
        
        write_results(&result, Path::new(".")).unwrap();
        
        assert!(!result.instructions.is_empty(), "No instructions were found");
    }
} 