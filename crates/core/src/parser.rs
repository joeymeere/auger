use std::collections::HashSet;

use ezbpf_core::program::Program;
use regex::Regex;

use crate::consts::*;
use crate::model::{Definition, ExtractConfig, ExtractResult, ExtractStats, SourceFile};
use crate::ExtractError;

pub enum ProgramType {
    Anchor,
    Native,
    Custom,
}

pub trait ProgramParser {
    fn parse_instructions(&self, text: &str) -> HashSet<String>;
    fn can_handle(&self, text: &str) -> bool;
    fn program_type(&self) -> &str;
    fn get_protected_instructions(&self, instructions: &HashSet<String>) -> HashSet<String>;
    fn extract_source_files(&self, text: &str) -> HashSet<SourceFile>;
    fn extract_standard_paths(&self, text: &str, source_files: &mut HashSet<SourceFile>);
    fn extract_definitions(&self, text: &str) -> HashSet<Definition>;
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

        // look for "Instruction: "
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

    fn extract_source_files(&self, text: &str) -> HashSet<SourceFile> {
        let mut source_files = HashSet::new();

        self.extract_standard_paths(text, &mut source_files);

        source_files
    }

    fn extract_standard_paths(&self, text: &str, source_files: &mut HashSet<SourceFile>) {
        let file_re = Regex::new(r"programs/[^.]+\.rs").unwrap();
        let project_re = Regex::new(r"programs/([^/]+)/").unwrap();

        let mut process_matches = |regex: &Regex| {
            for cap in regex.captures_iter(text) {
                if let Some(path_match) = cap.get(0) {
                    if let Some(project_match) = project_re.captures(path_match.as_str()) {
                        let project = project_match
                            .get(1)
                            .map(|m| m.as_str().to_string())
                            .unwrap_or_default();
                        let mut relative_path = path_match.as_str().to_string();

                        if let Some(rs_pos) = relative_path.find(".rs") {
                            relative_path = relative_path[0..rs_pos + 3].to_string();
                        }

                        relative_path = crate::utils::normalize_source_path(&relative_path);

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

    fn extract_definitions(&self, _text: &str) -> HashSet<Definition> {
        // AnchorProgramParser doesn't extract definitions
        HashSet::new()
    }
}

pub struct NativeProgramParser;

impl NativeProgramParser {
    pub fn new() -> Self {
        Self
    }
}

impl ProgramParser for NativeProgramParser {
    fn parse_instructions(&self, text: &str) -> HashSet<String> {
        let mut instructions = HashSet::new();

        // try "IX: " pattern for native programs
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

    // Should probably always return true
    fn can_handle(&self, text: &str) -> bool {
        // match IX: pattern
        let re = Regex::new(r"IX: ([A-Za-z0-9]+)").unwrap();
        // match <program_name>/src/<file_name>.rs
        let file_re = Regex::new(r"[a-zA-Z0-9_-]+/src/[a-zA-Z0-9_/-]+\.rs").unwrap();
        re.is_match(text) || file_re.is_match(text)
    }

    // ???? (programs/[^.]+\.rs|[a-zA-Z0-9_-]+/src/[^.]+\.rs)
    fn extract_source_files(&self, text: &str) -> HashSet<SourceFile> {
        let mut source_files = HashSet::new();

        self.extract_standard_paths(text, &mut source_files);

        source_files
    }

    fn extract_standard_paths(&self, text: &str, source_files: &mut HashSet<SourceFile>) {
        let file_re = Regex::new(r"[a-zA-Z0-9_-]+/src/[a-zA-Z0-9_/-]+\.rs").unwrap();

        for match_result in file_re.find_iter(text) {
            let path = match_result.as_str().to_string();

            if path.starts_with("programs/") {
                continue;
            }

            let parts: Vec<&str> = path.split("/src/").collect();
            if parts.len() >= 2 {
                if STD_LIB_NAMES.contains(&parts[0]) {
                    continue;
                }

                let project = parts[0].to_string();
                
                let mut relative_path = format!("src/{}", parts[1]);

                relative_path = crate::utils::normalize_source_path(&relative_path);
                source_files.insert(SourceFile {
                    path: format!("{}/{}", project, relative_path),
                    project,
                    relative_path,
                });
            }
        }
    }

    fn program_type(&self) -> &str {
        "native"
    }

    fn get_protected_instructions(&self, _instructions: &HashSet<String>) -> HashSet<String> {
        // native programs don't have idls, therefore no protected instructions
        HashSet::new()
    }

    fn extract_definitions(&self, _text: &str) -> HashSet<Definition> {
        // NativeProgramParser doesn't extract definitions
        HashSet::new()
    }
}

/// Parse binaries that use the LLVM linker, [LLD](https://lld.llvm.org/)
/// 
/// • Anchor, and some native programs use [`cargo build-bpf`](https://github.com/anza-xyz/agave/blob/6c86238f6486c7d95b0a3406dce1a09e620205ac/CHANGELOG.md?plain=1#L77), which leverages system linkers 
/// 
/// • Newer programs (using things like Pinnochio) use [`cargo build-sbf`](https://github.com/anza-xyz/agave/blob/a7092a20bb2f5d16375bdc531b71d2a164b43b93/platform-tools-sdk/sbf/c/sbf.mk#L37), which uses LLD by default.
/// 
/// • Even newer versions of Anchor [still seem to use `cargo build-bpf`](https://github.com/coral-xyz/anchor/blob/c509618412e004415c7b090e469a9e4d5177f642/cli/src/config.rs#L475), so this should not be needed in those cases. (?)
/// 
/// • `cargo build-bpf` has been considered *deprecated* since [July 11th 2024](https://github.com/anza-xyz/agave/commits/6c86238f6486c7d95b0a3406dce1a09e620205ac/CHANGELOG.md?after=6c86238f6486c7d95b0a3406dce1a09e620205ac+34).
/// 
/// These binaries will include a `.<program_name>` section in the ELF, after the (normally) last string table. 
/// This extra section includes another UTF-8 blob (not present in older binaries) containing imports, compliant with the [Itanium C++ ABI](https://itanium-cxx-abi.github.io/cxx-abi/abi.html).
/// 
/// ### Example:
/// 
/// `_ZN7program6module6module4file13DataStructure6method17h13871ae2612c8829E`
/// 
/// ### Structure:
/// 
/// - `_ZN` (mangling prefix)
/// - `<length_of_next_component>` (length of the program namespace)
/// - `<program_namespace>` (or program name, in our case)
/// - `<length_of_next_component>` (length of the module namespace)
/// - `<module_namespace>`
/// - `<length_of_next_component>` (length of the file name)
/// - `<file_name>`
/// - `<length_of_next_component>` (length of the data structure name)
/// - `<data_struct_name>` (camel cased for structs, otherwise snake_case)
/// - `<length_of_next_component>` (length of the method name)
/// - *`<method_name>`* (in the case that the previous component was a struct)
/// - `17h<hash>` (17 char hexadecimal hash from a Rust ext. of the ABI. Present for versioning and to prevent collisions.)
/// - `E` (denoting end)
/// - `<null_byte>` (null terminator)
/// 
/// ### Representation:
/// 
/// `<program>::<module>::<module?>::<file>::<DataStructure>::<method?>`

pub struct LLDProgramParser {
    program_name: Option<String>,
}

impl LLDProgramParser {
    pub fn new(program_name: Option<String>) -> Self {
        Self { program_name }
    }
    
    fn extract_demangled_symbols(&self, text: &str) -> Vec<crate::demangler::DemangledSymbol> {
        let mangled_names = crate::demangler::extract_mangled_names(text);
        
        mangled_names
            .iter()
            .filter_map(|name| {
                match crate::demangler::demangle(name) {
                    Ok(symbol) => Some(symbol),
                    Err(_) => None,
                }
            })
            .collect()
    }
    
    #[allow(dead_code)]
    fn extract_source_files_from_symbols(
        &self, 
        symbols: &[crate::demangler::DemangledSymbol]
    ) -> HashSet<SourceFile> {
        let mut source_files = HashSet::new();
        
        for symbol in symbols {
            if symbol.path.is_empty() {
                continue;
            }
            
            let project = symbol.path[0].clone();
            
            if let Some(ref expected_program) = self.program_name {
                if project != *expected_program {
                    continue;
                }
            }
            
            if symbol.path.len() > 1 {
                if STD_LIB_NAMES.contains(&project.as_str()) {
                    continue;
                }
                
                let path_str = symbol.path.join("::");
                if path_str.contains("core::") || path_str.contains("std::") {
                    continue;
                }

                let module_path = symbol.path[1..].join("::");
                let relative_path = format!("src/{}.rs", module_path.replace("::", "/"));
                
                let normalized_path = crate::utils::normalize_source_path(&relative_path);
                
                source_files.insert(SourceFile {
                    path: format!("{}/{}", project, normalized_path),
                    project,
                    relative_path: normalized_path,
                });
            }
        }
        
        source_files
    }
}

impl ProgramParser for LLDProgramParser {
    fn parse_instructions(&self, _text: &str) -> HashSet<String> {
        HashSet::<String>::new()
    }

    fn program_type(&self) -> &str {
        "sbf"
    }

    fn can_handle(&self, _text: &str) -> bool {
        true
    }

    fn extract_source_files(&self, _text: &str) -> HashSet<SourceFile> {
        HashSet::<SourceFile>::new()
    }

    fn extract_standard_paths(&self, _text: &str, _source_files: &mut HashSet<SourceFile>) {}

    fn get_protected_instructions(&self, _instructions: &HashSet<String>) -> HashSet<String> {
        HashSet::new()
    }

    fn extract_definitions(&self, text: &str) -> HashSet<Definition> {
        let mut definitions = HashSet::new();
        
        let mut extra_libs = vec![];
        let symbols = self.extract_demangled_symbols(text);
        
        for symbol in symbols {
            if symbol.path.is_empty() {
                continue;
            }

            // skip if it's an external lib
            if STD_LIB_NAMES.iter().any(|lib| symbol.path[0].starts_with(lib)) || ANCILLARY_LIB_NAMES.iter().any(|lib| symbol.path[0].starts_with(lib)) {
                extra_libs.push(symbol.path);
                continue;
            }
            
            let project = symbol.path[0].clone();
            
            if let Some(ref expected_program) = self.program_name {
                if project != *expected_program {
                    continue;
                }
            }
            
            let ident = if symbol.path.len() > 1 {
                let path_str = symbol.path.join("::");
                if !symbol.name.is_empty() {
                    format!("{}", path_str)
                } else {
                    path_str
                }
            } else if !symbol.name.is_empty() {
                project.clone()
            } else {
                project.clone()
            };
            
            let kind = symbol.symbol_type;
            
            let definition = Definition {
                ident,
                kind: kind.to_string(),
                hash: Some(symbol.name.clone()),
            };
            
            definitions.insert(definition);
        }
        
        definitions
    }
}

pub struct BpfParser {
    parsers: Vec<Box<dyn ProgramParser>>,
}

impl Default for BpfParser {
    fn default() -> Self {
        Self::new()
    }
}

impl BpfParser {
    pub fn new() -> Self {
        let parsers: Vec<Box<dyn ProgramParser>> = vec![
            Box::new(LLDProgramParser::new(None)),
            Box::new(AnchorProgramParser::new()),
            Box::new(NativeProgramParser::new()),
        ];

        Self { parsers }
    }

    pub fn with_parsers(parsers: Vec<Box<dyn ProgramParser>>) -> Self {
        Self { parsers }
    }

    pub fn register_parser(&mut self, parser: Box<dyn ProgramParser>) {
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
        config: &ExtractConfig,
    ) -> Result<ExtractResult, ExtractError> {
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
            definitions: definitions_vec,
            files: files_vec,
            stats,
            program_name,
            program_type,
            syscalls: syscalls_vec,
            custom_linker,
        };

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

pub fn extract_from_bytes(
    bytes: &[u8],
    config: ExtractConfig,
) -> Result<ExtractResult, ExtractError> {
    let parser = BpfParser::new();
    parser.extract_from_bytes(bytes, &config)
}

pub fn extract_from_bytes_with_parsers(
    bytes: &[u8],
    config: ExtractConfig,
    parsers: Vec<Box<dyn ProgramParser>>,
) -> Result<ExtractResult, ExtractError> {
    let parser = BpfParser::with_parsers(parsers);
    parser.extract_from_bytes(bytes, &config)
}
