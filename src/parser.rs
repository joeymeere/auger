use std::collections::HashSet;

use ezbpf_core::program::Program;
use regex::Regex;

use crate::consts::*;
use crate::model::{ExtractConfig, ExtractResult, ExtractStats, SourceFile};
use crate::ExtractError;

/// Parser for extracting data from BPF binaries
pub struct BpfParser;

impl BpfParser {
    /// Creates a new BpfParser instance
    pub fn new() -> Self {
        Self
    }

    /// Extracts text from a byte slice, and attempts to match instruction names
    pub fn extract_from_bytes(&self, bytes: &[u8], config: ExtractConfig) -> Result<ExtractResult, ExtractError> {
        let program = Program::from_bytes(bytes)
            .map_err(|e| ExtractError::ProgramParseError(format!("{:?}", e)))?;
        
        // check program headers
        if program.program_headers.len() <= config.program_header_index {
            return Err(ExtractError::NotEnoughProgramHeaders);
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
                    println!("null");
                    // replace null bytes with space
                    extracted_text.push(' ');
                } else {
                    // see if ASCII
                    if b.is_ascii() && b.is_ascii_graphic() {
                        println!("{}", b as char);
                        // printable
                        extracted_text.push(b as char);
                    } else {
                        println!("{}", b as char);
                        // non-printable: replace with space
                        extracted_text.push(' ');
                    }
                }
            } else {
                // only printable ascii
                if b.is_ascii() && b.is_ascii_graphic() {
                    println!("{}", b as char);
                    extracted_text.push(b as char);
                }
            }
            
            pos += 1;
        }
        
        if extracted_text.is_empty() {
            return Err(ExtractError::NoTextExtracted);
        }

        // Extract instructions, files, and other data
        let (instructions, protected_instructions, program_type) = self.extract_instructions(&extracted_text);
        let source_files = self.extract_source_files(&extracted_text);
        let syscalls = self.extract_syscalls(&program);
        
        let instructions_vec: Vec<String> = instructions
            .into_iter()
            .filter(|s| s.len() > 1 && s.len() <= 50)
            .filter(|s| !FALSE_POSITIVES.contains(&s.as_str()))
            .collect();

        let protected_instructions_vec: Vec<String> = protected_instructions.into_iter().collect();
        let files_vec: Vec<SourceFile> = source_files.into_iter().collect();
        let syscalls_vec: Vec<String> = syscalls.into_iter().collect();
        
        let program_name = if !files_vec.is_empty() {
            let mut project_counts = std::collections::HashMap::new();
            for file in &files_vec {
                *project_counts.entry(file.project.clone()).or_insert(0) += 1;
            }
            
            // Find the project with the highest count
            project_counts.into_iter()
                .max_by_key(|(_, count)| *count)
                .map(|(project, _)| project)
        } else {
            None
        };
        
        let stats = ExtractStats {
            start_offset: offset,
            end_position: pos,
            bytes_processed: pos - offset,
            instruction_count: instructions_vec.len(),
            file_count: files_vec.len(),
        };
        
        let result = ExtractResult {
            text: extracted_text,
            instructions: instructions_vec,
            protected_instructions: protected_instructions_vec,
            files: files_vec,
            stats,
            program_name,
            program_type,
            syscalls: syscalls_vec,
        };
        
        Ok(result)
    }

    /// Extracts instructions from the text
    fn extract_instructions(&self, text: &str) -> (HashSet<String>, HashSet<String>, String) {
        let mut instructions = HashSet::new();
        let mut protected_instructions = HashSet::new();
        let mut program_type = "anchor".to_string();
        
        // look for "Instruction: " corresponding to logs included w/ anchor programs
        let re = Regex::new(r"Instruction: ([A-Za-z0-9]+)").unwrap();
        
        let mut anchor_matches = false;
        for cap in re.captures_iter(text) {
            if let Some(instruction_name) = cap.get(1) {
                anchor_matches = true;
                println!("Found: {}", instruction_name.as_str());
                let mut name = instruction_name.as_str().to_string();
                if name.len() > 1 && name.len() <= 50 {
                    // Clean up instruction name by removing extra words
                    for keyword in REMOVABLE_KEYWORDS {
                        if name.ends_with(keyword) {
                            name = name[0..name.len() - keyword.len()].to_string();
                        }
                    }
                    
                    // Check if this is a protected instruction
                    let is_protected = PROTECTED_INSTRUCTIONS.contains(&name.as_str()) || 
                                      name.starts_with("Idl");
                    
                    if is_protected {
                        println!("Protected: {}", name);
                        protected_instructions.insert(name);
                    } else {
                        println!("Inserting: {}", name);
                        instructions.insert(name);
                    }
                }
            }
        }
        
        // If no "Instruction: " matches were found, try "IX: " pattern for native programs
        if !anchor_matches {
            let native_re = Regex::new(r"IX: ([A-Za-z0-9]+)").unwrap();
            let mut native_matches = false;
            
            for cap in native_re.captures_iter(text) {
                if let Some(instruction_name) = cap.get(1) {
                    native_matches = true;
                    let name = instruction_name.as_str().to_string();
                    if name.len() > 1 && name.len() <= 50 {
                        println!("Inserting: {}", name);
                        instructions.insert(name);
                    }
                }
            }
            
            if native_matches {
                program_type = "native".to_string();
            }
        }
        
        if anchor_matches {
            // look for instruction patterns without the "Instruction: " prefix
            let alt_re = Regex::new(r": ([A-Za-z0-9]+)Instruction").unwrap();
            for cap in alt_re.captures_iter(text) {
                if let Some(instruction_name) = cap.get(1) {
                    println!("Found: {}", instruction_name.as_str());
                    let mut name = format!("{}Instruction", instruction_name.as_str());
                    if name.len() > 1 && name.len() <= 50 {
                        // cleanup ix name
                        for keyword in REMOVABLE_KEYWORDS {
                            if name.ends_with(keyword) {
                                name = name[0..name.len() - keyword.len()].to_string();
                            }
                        }
                        
                        let is_protected = PROTECTED_INSTRUCTIONS.contains(&name.as_str()) || 
                                          name.starts_with("Idl");
                        
                        if is_protected {
                            protected_instructions.insert(name);
                        } else {
                            instructions.insert(name);
                        }
                    }
                }
            }
            
            // look for words followed by "Instruction"
            let additional_re = Regex::new(r"([A-Za-z0-9]+)Instruction").unwrap();
            for cap in additional_re.captures_iter(text) {
                if let Some(instruction_name) = cap.get(1) {
                    let mut name = format!("{}Instruction", instruction_name.as_str());
                    if name.len() > 1 && name.len() <= 50 {
                        // cleanup ix name
                        for keyword in REMOVABLE_KEYWORDS {
                            if name.ends_with(keyword) {
                                name = name[0..name.len() - keyword.len()].to_string();
                            }
                        }
                        let is_protected = PROTECTED_INSTRUCTIONS.contains(&name.as_str()) || 
                                          name.starts_with("Idl");
                        
                        if is_protected {
                            protected_instructions.insert(name);
                        } else {
                            instructions.insert(name);
                        }
                    }
                }
            }
        }
        
        (instructions, protected_instructions, program_type)
    }

    /// Extracts source files from the text
    fn extract_source_files(&self, text: &str) -> HashSet<SourceFile> {
        let mut source_files = HashSet::new();
        
        // match programs/**/src/**.rs and programs/**/src/**/**.rs
        let file_re = Regex::new(r"programs/([^/]+)/src/([^\s]+\.rs)").unwrap();
        let nested_file_re = Regex::new(r"programs/([^/]+)/src/([^/]+/[^\s]+\.rs)").unwrap();
        
        // Find direct src/*.rs files
        for cap in file_re.captures_iter(text) {
            if let (Some(project_match), Some(file_match)) = (cap.get(1), cap.get(2)) {
                let project = project_match.as_str().to_string();
                let mut relative_path = file_match.as_str().to_string();
                
                if let Some(rs_pos) = relative_path.find(".rs") {
                    relative_path = relative_path[0..rs_pos+3].to_string();
                }
                
                let path = format!("programs/{}/src/{}", project, relative_path);
                
                source_files.insert(SourceFile {
                    path,
                    project,
                    relative_path,
                });
            }
        }
        
        // Find nested src/**/*.rs files
        for cap in nested_file_re.captures_iter(text) {
            if let (Some(project_match), Some(file_match)) = (cap.get(1), cap.get(2)) {
                let project = project_match.as_str().to_string();
                let mut relative_path = file_match.as_str().to_string();
                
                if let Some(rs_pos) = relative_path.find(".rs") {
                    relative_path = relative_path[0..rs_pos+3].to_string();
                }
                
                let path = format!("programs/{}/src/{}", project, relative_path);
                
                source_files.insert(SourceFile {
                    path,
                    project,
                    relative_path,
                });
            }
        }
        
        source_files
    }

    /// Extracts syscalls from the program
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
}

/// Extracts text from a byte slice, and attempts to match instruction names
pub fn extract_from_bytes(bytes: &[u8], config: ExtractConfig) -> Result<ExtractResult, ExtractError> {
    let parser = BpfParser::new();
    parser.extract_from_bytes(bytes, config)
} 