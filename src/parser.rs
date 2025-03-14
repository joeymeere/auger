use std::collections::HashSet;

use ezbpf_core::program::Program;
use regex::Regex;

use crate::consts::*;
use crate::model::{ExtractConfig, ExtractResult, ExtractStats, SourceFile};
use crate::ExtractError;

// Map to strings
pub enum ProgramType {
    Anchor,
    Native,
    Custom,
}

/// Framework-specific instruction parsers
pub trait ProgramParser {
    fn parse_instructions(&self, text: &str) -> HashSet<String>;
    fn can_handle(&self, text: &str) -> bool;
    fn program_type(&self) -> &str;
    fn get_protected_instructions(&self, instructions: &HashSet<String>) -> HashSet<String>;
}

pub struct AnchorProgramParser;

impl AnchorProgramParser {
    pub fn new() -> Self {
        Self
    }
    
    fn clean_instruction_name(&self, name: &str) -> String {
        let mut cleaned_name = name.to_string();
        for keyword in REMOVABLE_KEYWORDS {
            if cleaned_name.ends_with(keyword) {
                cleaned_name = cleaned_name[0..cleaned_name.len() - keyword.len()].to_string();
            }
        }
        cleaned_name
    }
    
    fn is_protected(&self, name: &str) -> bool {
        PROTECTED_INSTRUCTIONS.contains(&name) || name.starts_with("Idl")
    }
}

impl ProgramParser for AnchorProgramParser {
    fn parse_instructions(&self, text: &str) -> HashSet<String> {
        let mut instructions = HashSet::new();
        
        // look for "Instruction: " corresponding to logs included w/ anchor programs
        let re = Regex::new(r"Instruction: ([A-Za-z0-9]+)").unwrap();
        
        for cap in re.captures_iter(text) {
            if let Some(instruction_name) = cap.get(1) {
                let name = instruction_name.as_str().to_string();
                if name.len() > 1 && name.len() <= 50 {
                    let cleaned_name = self.clean_instruction_name(&name);
                    instructions.insert(cleaned_name);
                }
            }
        }
        
        // look for instruction patterns without the "Instruction: " prefix
        let alt_re = Regex::new(r": ([A-Za-z0-9]+)Instruction").unwrap();
        for cap in alt_re.captures_iter(text) {
            if let Some(instruction_name) = cap.get(1) {
                let name = format!("{}Instruction", instruction_name.as_str());
                if name.len() > 1 && name.len() <= 50 {
                    let cleaned_name = self.clean_instruction_name(&name);
                    instructions.insert(cleaned_name);
                }
            }
        }
        
        // look for words followed by "Instruction"
        let additional_re = Regex::new(r"([A-Za-z0-9]+)Instruction").unwrap();
        for cap in additional_re.captures_iter(text) {
            if let Some(instruction_name) = cap.get(1) {
                let name = format!("{}Instruction", instruction_name.as_str());
                if name.len() > 1 && name.len() <= 50 {
                    let cleaned_name = self.clean_instruction_name(&name);
                    instructions.insert(cleaned_name);
                }
            }
        }
        
        instructions
    }
    
    fn can_handle(&self, text: &str) -> bool {
        let re = Regex::new(r"Instruction: ([A-Za-z0-9]+)").unwrap();
        re.is_match(text)
    }
    
    fn program_type(&self) -> &str {
        "anchor"
    }
    
    fn get_protected_instructions(&self, instructions: &HashSet<String>) -> HashSet<String> {
        instructions
            .iter()
            .filter(|name| self.is_protected(name))
            .cloned()
            .collect()
    }
}

/// Parser for Native programs
pub struct NativeProgramParser;

impl NativeProgramParser {
    pub fn new() -> Self {
        Self
    }
}

impl ProgramParser for NativeProgramParser {
    fn parse_instructions(&self, text: &str) -> HashSet<String> {
        let mut instructions = HashSet::new();
        
        // Try "IX: " pattern for native programs
        let native_re = Regex::new(r"IX: ([A-Za-z0-9]+)").unwrap();
        
        for cap in native_re.captures_iter(text) {
            if let Some(instruction_name) = cap.get(1) {
                let name = instruction_name.as_str().to_string();
                if name.len() > 1 && name.len() <= 50 {
                    instructions.insert(name);
                }
            }
        }
        
        instructions
    }
    
    fn can_handle(&self, text: &str) -> bool {
        let re = Regex::new(r"IX: ([A-Za-z0-9]+)").unwrap();
        re.is_match(text)
    }
    
    fn program_type(&self) -> &str {
        "native"
    }
    
    fn get_protected_instructions(&self, _instructions: &HashSet<String>) -> HashSet<String> {
        // Native programs don't have protected instructions in the same way Anchor does
        HashSet::new()
    }
}

/// Parser for extracting data from BPF binaries
pub struct BpfParser {
    parsers: Vec<Box<dyn ProgramParser>>,
}

impl BpfParser {
    /// Creates a new BpfParser instance
    pub fn new() -> Self {
        let mut parsers: Vec<Box<dyn ProgramParser>> = Vec::new();
        parsers.push(Box::new(AnchorProgramParser::new()));
        parsers.push(Box::new(NativeProgramParser::new()));
        
        Self { parsers }
    }
    
    /// Creates a new BpfParser instance with only the specified parsers
    pub fn with_parsers(parsers: Vec<Box<dyn ProgramParser>>) -> Self {
        Self { parsers }
    }
    
    /// Register a new program parser
    pub fn register_parser(&mut self, parser: Box<dyn ProgramParser>) {
        self.parsers.push(parser);
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
        // Try each parser in order
        for parser in &self.parsers {
            if parser.can_handle(text) {
                let instructions = parser.parse_instructions(text);
                let protected_instructions = parser.get_protected_instructions(&instructions);
                
                // Filter out protected instructions from the main set
                let filtered_instructions: HashSet<String> = instructions
                    .difference(&protected_instructions)
                    .cloned()
                    .collect();
                
                return (filtered_instructions, protected_instructions, parser.program_type().to_string());
            }
        }
        
        // Default to empty sets and "unknown" type if no parser can handle the text
        (HashSet::new(), HashSet::new(), "unknown".to_string())
    }

    /// Extracts source files from the text
    fn extract_source_files(&self, text: &str) -> HashSet<SourceFile> {
        let mut source_files = HashSet::new();
        
        // First pass: standard regex patterns for well-formed paths
        self.extract_standard_paths(text, &mut source_files);
        
        source_files
    }
    
    /// Extract standard well-formed paths
    fn extract_standard_paths(&self, text: &str, source_files: &mut HashSet<SourceFile>) {
        // Enhanced regex patterns to better capture file paths
        // This pattern looks for any occurrence of programs/*/src/*.rs with optional text before/after
        // programs/([^/]+)/
        let file_re = Regex::new(r"programs/[^.]+\.rs").unwrap();
        let project_re = Regex::new(r"programs/([^/]+)/").unwrap();
        
        // Find all file paths in the text
        let mut process_matches = |regex: &Regex| {
            for cap in regex.captures_iter(text) {
                if let Some(path_match) = cap.get(0) {
                    if let Some(project_match) = project_re.captures(path_match.as_str()) {
                        let project = project_match.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
                        let mut relative_path = path_match.as_str().to_string();
                        
                        if let Some(rs_pos) = relative_path.find(".rs") {
                            relative_path = relative_path[0..rs_pos+3].to_string();
                        }
                        
                        let path = format!("programs/{}/src/{}", project, relative_path);
                    
                        source_files.insert(SourceFile {
                                path,
                                project: project.clone(),
                                relative_path,
                        });
                    }
                }
            }
        };
        
        process_matches(&file_re);
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

pub fn extract_from_bytes(bytes: &[u8], config: ExtractConfig) -> Result<ExtractResult, ExtractError> {
    let parser = BpfParser::new();
    parser.extract_from_bytes(bytes, config)
}

pub fn extract_from_bytes_with_parsers(
    bytes: &[u8], 
    config: ExtractConfig,
    parsers: Vec<Box<dyn ProgramParser>>
) -> Result<ExtractResult, ExtractError> {
    let parser = BpfParser::with_parsers(parsers);
    parser.extract_from_bytes(bytes, config)
} 