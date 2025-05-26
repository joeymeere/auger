use std::path::Path;

use anyhow::Result;

pub mod models;
// pub mod syn;
pub mod utils;
pub mod consts;
pub mod error;
pub mod traits;
pub mod memory;
pub mod parsing;
pub mod analyzers;
pub mod resolvers;
pub mod demangler;

pub use models::{AugerResult, AugerStats, AugerConfig, Instruction, SourceFile};
pub use parsing::{BaseSBFParser, AnchorParser, LLDParser, NativeParser, SolanaProgramType};
pub use utils::writer::{FileWriter, dump_elf_meta as dump_elf, write_results as compile_results};
pub use traits::AugerParser;
pub use memory::MemoryMap;
pub use error::AugerError;

pub fn extract_from_bytes(
    file_bytes: &[u8],
    config: Option<models::AugerConfig>,
) -> Result<AugerResult, AugerError> {
    let config = config.unwrap_or_default();
    parsing::extract_from_bytes_handler(file_bytes, config)
}

/// Extracts valid text from a Solana binary, and attempts to match instruction names
pub fn extract_from_file(
    file_path: &Path,
    config: Option<AugerConfig>,
) -> Result<AugerResult, AugerError> {
    let config = config.unwrap_or_default();

    if file_path.extension().unwrap() != "so" {
        return Err(AugerError::InvalidFileExtension);
    }

    let file_bytes = std::fs::read(file_path)?;
    parsing::extract_from_bytes_handler(file_bytes.as_slice(), config)
}

/// Extracts valid text from an sBPF binary using custom parsers
pub fn extract_from_file_with_parsers(
    file_path: &Path,
    config: Option<AugerConfig>,
    parsers: Vec<Box<dyn AugerParser>>,
) -> Result<AugerResult, AugerError> {
    let config = config.unwrap_or_default();

    if file_path.extension().unwrap() != "so" {
        return Err(AugerError::InvalidFileExtension);
    }

    let file_bytes = std::fs::read(file_path)?;
    parsing::extract_from_bytes_with_parsers_handler(file_bytes.as_slice(), config, parsers)
}

/// Dumps the ELF metadata to a JSON file
pub fn dump_elf_meta(file_bytes: &[u8], base_path: &Path) -> Result<(), AugerError> {
    dump_elf(file_bytes, base_path)
}

/// Writes extraction results to files
pub fn write_results(result: &AugerResult, base_path: &Path) -> Result<(), AugerError> {
    compile_results(result, base_path)
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

        println!(
            "Starting extraction from offset: {}",
            result.stats.start_offset
        );
        println!(
            "Extraction ended at position: {}",
            result.stats.end_position
        );
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

        assert!(
            !result.instructions.is_empty(),
            "No instructions were found"
        );
    }
}
