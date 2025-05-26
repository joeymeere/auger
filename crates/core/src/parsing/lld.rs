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
use std::collections::HashSet;

use crate::traits::AugerParser;
use crate::{consts::{ANCILLARY_LIB_NAMES, STD_LIB_NAMES}, models::{Definition, SourceFile}};

pub struct LLDParser {
    program_name: Option<String>,
}

impl LLDParser {
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

impl AugerParser for LLDParser {
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