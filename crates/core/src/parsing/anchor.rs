use std::collections::HashSet;
use regex::Regex;

use crate::{consts::{PROTECTED_INSTRUCTIONS, REMOVABLE_KEYWORDS}, models::{Definition, SourceFile}};

use crate::traits::AugerParser;

pub struct AnchorParser;

impl AnchorParser {
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

impl AugerParser for AnchorParser {
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
        // no definition extracting yet
        HashSet::new()
    }
}