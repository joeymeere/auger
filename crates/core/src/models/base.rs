use serde::{Deserialize, Serialize};

use super::RichInstruction;

#[derive(Debug, Clone)]
pub struct AugerConfig {
    /// Consecutive 0xFF bytes to consider as EOT
    pub ff_sequence_length: usize,
    /// Program header index to use for offset (default is 1)
    pub program_header_index: usize,
    /// Replace null bytes and non-printable characters with spaces
    pub replace_non_printable: bool,
    /// Attempt to recover type information from the binary
    pub recover_types: bool,
}

impl Default for AugerConfig {
    fn default() -> Self {
        Self {
            ff_sequence_length: 8,
            program_header_index: 1,
            replace_non_printable: true,
            recover_types: false,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AugerResult {
    /// The extracted text
    pub text: String,
    /// List of unique instructions found
    pub instructions: Vec<String>,
    /// List of protected instructions found (e.g., Idl* instructions)
    pub protected_instructions: Vec<String>,
    /// List of functions, structs, enums, and traits found
    pub definitions: Vec<Definition>,
    /// List of source file references found
    pub files: Vec<SourceFile>,
    /// Statistics about the extraction
    pub stats: AugerStats,
    /// Name of the program (derived from file paths)
    pub program_name: Option<String>,
    /// Type of program (anchor or native)
    pub program_type: String,
    /// List of syscalls found in .dynstr section
    pub syscalls: Vec<String>,
    /// Custom linker information if found in .comment section
    pub custom_linker: Option<String>,
    /// Disassembly of the program with resolved references
    pub disassembly: Vec<String>,
    /// Strings found in the binary (address -> string)
    pub strings: Vec<StringReference>,
    /// Type recovery report (if enabled)
    pub type_report: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AugerStats {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instruction {
    /// Name of the instruction
    pub name: String,
    /// Position in the extracted text where the instruction was found
    pub position: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SourceFile {
    /// Full path of the file
    pub path: String,
    /// Project name (extracted from the path)
    pub project: String,
    /// Relative path within the project
    pub relative_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StringReference {
    /// Address of the string in memory
    pub address: u64,
    /// The string content
    pub content: String,
    /// List of addresses that reference this string
    pub referenced_by: Vec<u64>,
}

/// Represents a definition found in the binary (function, struct, enum, trait)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Definition {
    /// Full identifier path (e.g., "name::dex::phoenix::swap")
    pub ident: String,
    /// Type of definition (fn, struct, enum, trait)
    pub kind: String,
    /// Hash value from the mangled name
    pub hash: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum DefinitionKind {
    Function,
    Struct,
    Enum,
    Trait,
}

#[derive(Debug, Clone)]
pub struct FunctionBlock {
    /// Starting address of the function
    pub address: u64,
    /// Name of the function (if known)
    pub name: String,
    /// Size of the function in bytes
    pub size: usize,
    /// Instructions in this function
    pub instructions: Vec<RichInstruction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ControlFlow {
    /// Function call
    Call {
        /// Address of the call instruction
        from_addr: u64,
        /// Target address being called
        to_addr: u64,
        /// Function containing the call
        from_func: u64,
        /// Function being called
        to_func: u64,
    },
    /// Jump between functions
    Jump {
        /// Address of the jump instruction
        from_addr: u64,
        /// Target address being jumped to
        to_addr: u64,
        /// Function containing the jump
        from_func: u64,
        /// Function being jumped to
        to_func: u64,
        /// Whether this is a conditional jump
        conditional: bool,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryReference {
    /// Address of the instruction making the reference
    pub address: u64,
    /// Target address being referenced
    pub target: u64,
    /// Size of the memory access in bytes
    pub size: usize,
    /// Whether this is a write operation
    pub is_write: bool,
}