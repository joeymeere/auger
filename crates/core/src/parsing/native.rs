use std::collections::HashSet;
use regex::Regex;

use crate::traits::AugerParser;
use crate::{consts::STD_LIB_NAMES, models::{Definition, SourceFile}};

pub struct NativeParser;

impl NativeParser {
    pub fn new() -> Self {
        Self
    }
}

impl AugerParser for NativeParser {
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