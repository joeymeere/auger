use serde::{Deserialize, Serialize};

/// Config for text extraction
#[derive(Debug, Clone)]
pub struct ExtractConfig {
    /// Consecutive 0xFF bytes to consider as EOT
    pub ff_sequence_length: usize,
    /// Program header index to use for offset (default is 1)
    pub program_header_index: usize,
    /// Replace null bytes and non-printable characters with spaces
    pub replace_non_printable: bool,
}

impl Default for ExtractConfig {
    fn default() -> Self {
        Self {
            ff_sequence_length: 8,
            program_header_index: 1,
            replace_non_printable: true,
        }
    }
}

/// Represents an instruction found in the binary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instruction {
    /// Name of the instruction
    pub name: String,
    /// Position in the extracted text where the instruction was found
    pub position: usize,
}

/// Represents a source file reference found in the binary
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SourceFile {
    /// Full path of the file
    pub path: String,
    /// Project name (extracted from the path)
    pub project: String,
    /// Relative path within the project
    pub relative_path: String,
}

/// Results of the extraction process
#[derive(Debug, Serialize, Deserialize)]
pub struct ExtractResult {
    /// The extracted text
    pub text: String,
    /// List of unique instructions found
    pub instructions: Vec<String>,
    /// List of protected instructions found (e.g., Idl* instructions)
    pub protected_instructions: Vec<String>,
    /// List of source file references found
    pub files: Vec<SourceFile>,
    /// Statistics about the extraction
    pub stats: ExtractStats,
    /// Name of the program (derived from file paths)
    pub program_name: Option<String>,
    /// Type of program (anchor or native)
    pub program_type: String,
    /// List of syscalls found in .dynstr section
    pub syscalls: Vec<String>,
    /// Custom linker information if found in .comment section
    pub custom_linker: Option<String>,
}

/// Statistics about the extraction process
#[derive(Debug, Serialize, Deserialize)]
pub struct ExtractStats {
    /// Offset where extraction started
    pub start_offset: usize,
    /// Position where extraction ended
    pub end_position: usize,
    /// Total bytes processed
    pub bytes_processed: usize,
    /// Number of unique instructions found
    pub instruction_count: usize,
    /// Number of unique source files found
    pub file_count: usize,
} 