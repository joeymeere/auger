use std::fs;
use std::path::Path;

use ezbpf_core::program::Program;
use serde::{Deserialize, Serialize};

use crate::AugerResult;
use crate::AugerError;

#[derive(Serialize, Deserialize)]
pub struct Manifest {
    pub program_name: String,
    pub program_type: String,
    pub instructions: Vec<String>,
    pub protected_instructions: Vec<String>,
    pub syscalls: Vec<String>,
    pub source_files: Vec<String>,
    pub custom_linker: Option<String>,
    pub disassembly: Vec<String>,
    pub string_references: Vec<StringReference>,
}

#[derive(Serialize, Deserialize)]
pub struct StringReference {
    pub address: u64,
    pub content: String,
    pub referenced_by: Vec<u64>,
}

pub struct FileWriter;

impl Default for FileWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl FileWriter {
    pub fn new() -> Self {
        Self
    }

    pub fn dump_elf_meta(&self, file_bytes: &[u8], base_path: &Path) -> Result<(), AugerError> {
        let program = Program::from_bytes(file_bytes)
            .map_err(|e| AugerError::ProgramParseError(format!("{:?}", e)))?;

        let json = serde_json::to_string_pretty(&program)
            .map_err(|e| AugerError::ProgramParseError(format!("{:?}", e)))?;

        fs::write(base_path.join("elf-meta.json"), json)?;

        Ok(())
    }

    pub fn write_results(
        &self,
        result: &AugerResult,
        base_path: &Path,
    ) -> Result<(), AugerError> {
        fs::create_dir_all(base_path)?;

        let prefix = match &result.program_name {
            Some(name) => format!("{}_", name),
            None => String::new(),
        };

        fs::write(
            base_path.join(format!("{}text_dump.txt", prefix)),
            &result.text,
        )?;

        self.write_manifest(result, base_path, &prefix)?;

        let full_json = serde_json::to_string_pretty(result)?;
        fs::write(base_path.join(format!("{}result.json", prefix)), full_json)?;
        
        // Write type report if available
        if let Some(type_report) = &result.type_report {
            fs::write(
                base_path.join(format!("{}type_report.md", prefix)),
                type_report,
            )?;
        }

        Ok(())
    }

    fn write_manifest(
        &self,
        result: &AugerResult,
        base_path: &Path,
        prefix: &str,
    ) -> Result<(), AugerError> {
        let program_name = match &result.program_name {
            Some(name) => name.to_string(),
            None => String::new(),
        };

        let string_references = result.strings.iter().map(|sr| {
            StringReference {
                address: sr.address,
                content: sr.content.clone(),
                referenced_by: sr.referenced_by.clone(),
            }
        }).collect();

        let manifest = Manifest {
            program_name,
            program_type: result.program_type.clone(),
            instructions: result.instructions.clone(),
            protected_instructions: result.protected_instructions.clone(),
            syscalls: result.syscalls.clone(),
            source_files: result.files.iter().map(|f| f.path.clone()).collect(),
            custom_linker: result.custom_linker.clone(),
            disassembly: result.disassembly.clone(),
            string_references,
        };

        let manifest_json = serde_json::to_string_pretty(&manifest)?;
        fs::write(
            base_path.join(format!("{}manifest.json", prefix)),
            manifest_json,
        )?;
        Ok(())
    }
}

pub fn dump_elf_meta(file_bytes: &[u8], base_path: &Path) -> Result<(), AugerError> {
    let writer = FileWriter::new();
    writer.dump_elf_meta(file_bytes, base_path)
}

pub fn write_results(result: &AugerResult, base_path: &Path) -> Result<(), AugerError> {
    let writer = FileWriter::new();
    writer.write_results(result, base_path)
}
