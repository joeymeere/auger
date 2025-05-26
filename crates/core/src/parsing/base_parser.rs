use std::collections::HashSet;
use log::{debug, info};

use ezbpf_core::program::Program;

use super::{AnchorParser, LLDParser, NativeParser};
use crate::{
    consts::FALSE_POSITIVES, error::AugerError, memory::MemoryMap, models::{
        AugerConfig,
        AugerResult, 
        AugerStats, 
        Definition, 
        SourceFile, 
        StringReference
    }, traits::AugerParser
};

#[derive(Debug, Clone)]
pub enum SolanaProgramType {
    Anchor,
    Native,
    Custom,
}

pub struct BaseSBFParser {
    parsers: Vec<Box<dyn AugerParser>>,
}

impl Default for BaseSBFParser {
    fn default() -> Self {
        Self::new()
    }
}

impl BaseSBFParser {
    pub fn new() -> Self {
        let parsers: Vec<Box<dyn AugerParser>> = vec![
            Box::new(LLDParser::new(None)),
            Box::new(AnchorParser::new()),
            Box::new(NativeParser::new()),
        ];

        Self { parsers }
    }

    pub fn with_parsers(parsers: Vec<Box<dyn AugerParser>>) -> Self {
        Self { parsers }
    }

    pub fn register_parser(&mut self, parser: Box<dyn AugerParser>) {
        self.parsers.push(parser);
    }

    fn normalize_source_files(
        &self,
        source_files: HashSet<SourceFile>,
        program_name: Option<String>,
    ) -> HashSet<SourceFile> {
        let mut normalized = HashSet::new();

        for file in source_files {
            let mut normalized_file = file.clone();

            if let Some(ref program) = program_name {
                if file.project != *program {
                    normalized_file.project = program.clone();
                    normalized_file.path = format!("{}/{}", program, file.relative_path);
                }
            }

            normalized.insert(normalized_file);
        }

        normalized
    }

    pub fn extract_from_bytes(
        &self,
        bytes: &[u8],
        config: &AugerConfig,
    ) -> Result<AugerResult, AugerError> {
        let program = Program::from_bytes(bytes)
            .map_err(|e| AugerError::ProgramParseError(format!("{:?}", e)))?;

        // check program headers
        if program.program_headers.len() <= config.program_header_index {
            return Err(AugerError::NotEnoughProgramHeaders);
        }

        // get offset from the specified program header
        let offset = program.program_headers[config.program_header_index].p_offset as usize;
        let mut extracted_text = String::new();

        let mut pos = offset;
        let mut consecutive_ff_count = 0;

        // 0xFF appears in sequence for padding
        while pos < bytes.len() && consecutive_ff_count < config.ff_sequence_length {
            let b = bytes[pos];

            if b == 0xFF {
                consecutive_ff_count += 1;
            } else {
                consecutive_ff_count = 0;
            }

            if config.replace_non_printable {
                if b == 0 {
                    // replace null bytes with space
                    extracted_text.push(' ');
                } else {
                    // see if ASCII
                    if b.is_ascii() && b.is_ascii_graphic() {
                        // printable
                        extracted_text.push(b as char);
                    } else {
                        // non-printable: replace with space
                        extracted_text.push(' ');
                    }
                }
            } else {
                // only printable ascii
                if b.is_ascii() && b.is_ascii_graphic() {
                    extracted_text.push(b as char);
                }
            }

            pos += 1;
        }

        if extracted_text.is_empty() {
            return Err(AugerError::NoTextExtracted);
        }

        let (instructions, protected_instructions, program_type) =
            self.extract_instructions(&extracted_text);
        let mut source_files = self.extract_source_files(&extracted_text);
        let syscalls = self.extract_syscalls(&program);
        let custom_linker = self.extract_custom_linker(&program);
        let mut definitions = HashSet::new();
        for parser in &self.parsers {
            if parser.can_handle(&extracted_text) {
                let parser_definitions = parser.extract_definitions(&extracted_text);
                definitions.extend(parser_definitions);
            }
        }

        // Create memory map for string references and disassembly
        let memory_map = MemoryMap::new(&program, bytes);
        // there is no `.disassemble()` method
        let disassembly = memory_map.get_instructions();
        
        // Convert string references to our format
        let mut string_references = Vec::new();
        for (addr, content) in memory_map.get_strings() {
            let referenced_by = memory_map.get_references()
                .get(addr)
                .cloned()
                .unwrap_or_default();
            
            string_references.push(StringReference {
                address: *addr,
                content: content.clone(),
                referenced_by,
            });
        }

        let instructions_vec: Vec<String> = instructions
            .into_iter()
            .filter(|s| s.len() > 1 && s.len() <= 50)
            .filter(|s| !FALSE_POSITIVES.contains(&s.as_str()))
            .collect();

        let protected_instructions_vec: Vec<String> = protected_instructions.into_iter().collect();
        let definitions_vec: Vec<Definition> = definitions.into_iter().collect();
        let source_files_vec: Vec<SourceFile> = source_files.into_iter().collect();
        let program_name = crate::utils::find_main_project(&source_files_vec, |f| &f.project);

        source_files = self
            .normalize_source_files(source_files_vec.into_iter().collect(), program_name.clone());

        let files_vec: Vec<SourceFile> = source_files.into_iter().collect();
        let syscalls_vec: Vec<String> = syscalls.into_iter().collect();

        let stats = AugerStats {
            start_offset: offset,
            end_position: pos,
            bytes_processed: pos - offset,
            instruction_count: instructions_vec.len(),
            file_count: files_vec.len(),
        };

        let result = AugerResult {
            text: extracted_text,
            instructions: instructions_vec,
            protected_instructions: protected_instructions_vec,
            definitions: definitions_vec,
            files: files_vec,
            stats,
            program_name,
            program_type,
            syscalls: syscalls_vec,
            custom_linker,
            disassembly: vec![],
            strings: string_references,
            type_report: None,
        };

        // Perform type recovery if enabled
        if config.recover_types {
            info!("Type recovery enabled, starting recovery process");
            // Create a memory map for the binary
            debug!("Creating memory map for type recovery");
            let program = ezbpf_core::program::Program::from_bytes(bytes)
                .map_err(|e| AugerError::ProgramParseError(format!("{:?}", e)))?;
            
            let memory_map = crate::memory::MemoryMap::new(&program, bytes);
            
            // Perform type recovery
            /*
            debug!("Initializing type recovery system");
            let mut type_recovery = Type::new(bytes, &memory_map)
                .map_err(|e| AugerError::ProgramParseError(format!("Failed to initialize type recovery: {}", e)))?;
            
            // Recover types and handle any errors
            debug!("Starting type recovery process");
            let _type_registry = type_recovery.recover_types();
            
            // Generate and add type report to the result
            debug!("Generating type recovery report");
            let report = type_recovery.generate_report();
            info!("Type recovery complete, generated report of {} bytes", report.len());
            result.type_report = Some(report);
                        */
        }
        
        Ok(result)
    }

    fn extract_instructions(&self, text: &str) -> (HashSet<String>, HashSet<String>, String) {
        let mut all_instructions = HashSet::new();
        let mut all_protected_instructions = HashSet::new();
        let mut program_type = "unknown".to_string();
        let mut found_parser = false;
        
        for parser in &self.parsers {
            if parser.can_handle(text) {
                let instructions = parser.parse_instructions(text);
                let protected_instructions = parser.get_protected_instructions(&instructions);
                
                if !found_parser {
                    program_type = parser.program_type().to_string();
                    found_parser = true;
                }
                
                all_instructions.extend(instructions);
                all_protected_instructions.extend(protected_instructions);
            }
        }
        
        if !found_parser {
            println!("BpfParser: No parser could handle the text, using unknown type");
        }
        
        let filtered_instructions: HashSet<String> = all_instructions
            .difference(&all_protected_instructions)
            .cloned()
            .collect();
        
        (filtered_instructions, all_protected_instructions, program_type)
    }

    fn extract_source_files(&self, text: &str) -> HashSet<SourceFile> {
        let mut all_source_files = HashSet::new();
        let mut found_parser = false;
        
        for parser in &self.parsers {
            if parser.can_handle(text) {
                let paths = parser.extract_source_files(text);
                
                all_source_files.extend(paths);
                found_parser = true;
            }
        }
        
        if !found_parser {
            println!("BpfParser: No parser could handle the text for source files");
        }
        
        all_source_files
    }

    fn extract_syscalls(&self, program: &Program) -> HashSet<String> {
        let mut syscalls = HashSet::new();

        for section in &program.section_header_entries {
            if section.label.contains(".dynstr") {
                let entries: Vec<&str> = section.utf8.split('\u{0000}').collect();

                for entry in entries {
                    if !entry.is_empty() && entry.len() <= 30 {
                        syscalls.insert(entry.to_string());
                    }
                }
            }
        }

        syscalls
    }

    fn extract_custom_linker(&self, program: &Program) -> Option<String> {
        for section in program.section_header_entries.iter().rev() {
            if section.label.contains(".comment") || section.label.contains(".strtab") {
                if let Some(linker_pos) = section.utf8.rfind("Linker: ") {
                    let linker_info = &section.utf8[linker_pos + "Linker: ".len()..];
                    let end_pos = linker_info.find('\0').unwrap_or(linker_info.len());
                    let linker = linker_info[..end_pos].trim().to_string();
                    if !linker.is_empty() {
                        return Some(linker);
                    }
                }
            }
        }

        None
    }
}

pub fn extract_from_bytes_with_parsers_handler(
    bytes: &[u8],
    config: AugerConfig,
    parsers: Vec<Box<dyn AugerParser>>,
) -> Result<AugerResult, AugerError> {
    let parser = BaseSBFParser::with_parsers(parsers);
    let result = parser.extract_from_bytes(bytes, &config)?;
    
    Ok(result)
}

pub fn extract_from_bytes_handler(
    bytes: &[u8],
    config: AugerConfig,
) -> Result<AugerResult, AugerError> {
    let parser = BaseSBFParser::default();
    Ok(extract_from_bytes_with_parsers_handler(bytes, config, parser.parsers)?)
}