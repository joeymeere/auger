use std::collections::HashSet;

use ezbpf_core::program::Program;
use regex::Regex;

use crate::consts::*;
use crate::model::{ExtractConfig, ExtractResult, ExtractStats, SourceFile};
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
        use std::collections::HashMap;

        if source_files.is_empty() {
            return source_files;
        }

        let filtered_files: HashSet<SourceFile> = source_files
            .into_iter()
            .filter(|file| !crate::consts::STD_LIB_NAMES.contains(&file.project.as_str()))
            .collect();

        if filtered_files.is_empty() {
            return filtered_files;
        }

        let files_vec: Vec<SourceFile> = filtered_files.into_iter().collect();
        let main_project = match program_name {
            Some(name) => name,
            None => crate::utils::find_main_project(&files_vec, |f| &f.project).unwrap_or_default(),
        };

        let mut normalized_files = HashSet::new();
        let mut path_map: HashMap<String, Vec<SourceFile>> = HashMap::new();

        for file in files_vec {
            let normalized_project =
                crate::utils::normalize_project_name(&file.project, &main_project);
            let normalized_rel_path = crate::utils::normalize_source_path(&file.relative_path);

            let normalized_file = SourceFile {
                path: if normalized_project != file.project {
                    format!("{}/{}", normalized_project, normalized_rel_path)
                } else if normalized_rel_path != file.relative_path {
                    format!("{}/{}", normalized_project, normalized_rel_path)
                } else {
                    file.path
                },
                project: normalized_project,
                relative_path: normalized_rel_path,
            };

            path_map
                .entry(normalized_file.relative_path.clone())
                .or_insert_with(Vec::new)
                .push(normalized_file);
        }

        for (_, mut files) in path_map {
            if files.len() == 1 {
                normalized_files.insert(files.pop().unwrap());
            } else {
                let main_project_idx = files.iter().position(|f| f.project == main_project);
                if let Some(idx) = main_project_idx {
                    normalized_files.insert(files.remove(idx));
                } else {
                    let shortest_project_idx = files
                        .iter()
                        .enumerate()
                        .min_by_key(|(_, f)| f.project.len())
                        .map(|(idx, _)| idx);

                    if let Some(idx) = shortest_project_idx {
                        normalized_files.insert(files.remove(idx));
                    } else {
                        normalized_files.insert(files.pop().unwrap());
                    }
                }
            }
        }

        normalized_files
    }

    pub fn extract_from_bytes(
        &self,
        bytes: &[u8],
        config: ExtractConfig,
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

        let instructions_vec: Vec<String> = instructions
            .into_iter()
            .filter(|s| s.len() > 1 && s.len() <= 50)
            .filter(|s| !FALSE_POSITIVES.contains(&s.as_str()))
            .collect();

        let protected_instructions_vec: Vec<String> = protected_instructions.into_iter().collect();

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
        for parser in &self.parsers {
            if parser.can_handle(text) {
                let instructions = parser.parse_instructions(text);
                let protected_instructions = parser.get_protected_instructions(&instructions);

                let filtered_instructions: HashSet<String> = instructions
                    .difference(&protected_instructions)
                    .cloned()
                    .collect();

                return (
                    filtered_instructions,
                    protected_instructions,
                    parser.program_type().to_string(),
                );
            }
        }

        (HashSet::new(), HashSet::new(), "unknown".to_string())
    }

    fn extract_source_files(&self, text: &str) -> HashSet<SourceFile> {
        let mut source_files = HashSet::new();

        for parser in &self.parsers {
            if parser.can_handle(text) {
                let paths = parser.extract_source_files(text);

                source_files.extend(paths);

                return source_files;
            }
        }

        source_files
    }

    /*
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
    */

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

    /// reverse searches for the ".comment" section in the ELF file's string tables
    /// informs a further search for a custom linker name
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
    parser.extract_from_bytes(bytes, config)
}

pub fn extract_from_bytes_with_parsers(
    bytes: &[u8],
    config: ExtractConfig,
    parsers: Vec<Box<dyn ProgramParser>>,
) -> Result<ExtractResult, ExtractError> {
    let parser = BpfParser::with_parsers(parsers);
    parser.extract_from_bytes(bytes, config)
}
