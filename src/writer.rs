use std::fs;
use std::path::Path;

use ezbpf_core::program::Program;
use serde::{Deserialize, Serialize};

use crate::model::ExtractResult;
use crate::ExtractError;

/// Writer for BPF extraction results
#[derive(Serialize, Deserialize)]
pub struct Manifest {
    pub program_name: String,
    pub program_type: String,
    pub instructions: Vec<String>,
    pub protected_instructions: Vec<String>,
    pub syscalls: Vec<String>,
    pub source_files: Vec<String>,
}

pub struct FileWriter;

impl FileWriter {
    /// Creates a new BpfWriter instance
    pub fn new() -> Self {
        Self
    }

    /// Dumps the ELF metadata to a JSON file
    pub fn dump_elf_meta(&self, file_bytes: &[u8], base_path: &Path) -> Result<(), ExtractError> {
        let program = Program::from_bytes(file_bytes)
            .map_err(|e| ExtractError::ProgramParseError(format!("{:?}", e)))?;
        
        let json = program.to_json()
            .map_err(|e| ExtractError::ProgramParseError(format!("{:?}", e)))?;
        
        fs::write(base_path.join("elf-meta.json"), json)?;

        Ok(())
    }

    /// Writes extraction results to files
    pub fn write_results(&self, result: &ExtractResult, base_path: &Path) -> Result<(), ExtractError> {
        fs::create_dir_all(base_path)?;
        
        let prefix = match &result.program_name {
            Some(name) => format!("{}_", name),
            None => String::new(),
        };
        
        fs::write(
            base_path.join(format!("{}text_dump.txt", prefix)), 
            &result.text
        )?;

        self.write_manifest(result, base_path, &prefix)?;
        
        let full_json = serde_json::to_string_pretty(result)?;
        fs::write(
            base_path.join(format!("{}result.json", prefix)), 
            full_json
        )?;
        
        Ok(())
    }

    fn write_manifest(&self, result: &ExtractResult, base_path: &Path, prefix: &str) -> Result<(), ExtractError> {
        let program_name = match &result.program_name {
            Some(name) => name.to_string(),
            None => String::new(),
        };

        let manifest = Manifest {
            program_name,
            program_type: result.program_type.clone(),
            instructions: result.instructions.clone(),
            protected_instructions: result.protected_instructions.clone(),
            syscalls: result.syscalls.clone(),
            source_files: result.files.iter().map(|f| f.path.clone()).collect(),
        };

        let manifest_json = serde_json::to_string_pretty(&manifest)?;
        fs::write(base_path.join(format!("{}manifest.json", prefix)), manifest_json)?;
        Ok(())
    }
}

/// Dumps the ELF metadata to a JSON file
pub fn dump_elf_meta(file_bytes: &[u8], base_path: &Path) -> Result<(), ExtractError> {
    let writer = FileWriter::new();
    writer.dump_elf_meta(file_bytes, base_path)
}

/// Writes extraction results to files
pub fn write_results(result: &ExtractResult, base_path: &Path) -> Result<(), ExtractError> {
    let writer = FileWriter::new();
    writer.write_results(result, base_path)
} 