use std::collections::HashSet;

use crate::models::{Definition, SourceFile};

pub trait AugerParser {
    fn parse_instructions(&self, text: &str) -> HashSet<String>;
    fn can_handle(&self, text: &str) -> bool;
    fn program_type(&self) -> &str;
    fn get_protected_instructions(&self, instructions: &HashSet<String>) -> HashSet<String>;
    fn extract_source_files(&self, text: &str) -> HashSet<SourceFile>;
    fn extract_standard_paths(&self, text: &str, source_files: &mut HashSet<SourceFile>);
    fn extract_definitions(&self, text: &str) -> HashSet<Definition>;
}