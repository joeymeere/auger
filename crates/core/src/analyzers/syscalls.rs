use ezbpf_core::opcodes::OpCode;
use log::{debug, info};
use std::collections::HashMap;

use crate::memory::MemoryMap;
use crate::models::{FunctionBlock, ControlFlow, MemoryReference};
use crate::traits::analyzer::AugerAnalyzer;

pub struct SyscallAnalyzer {
    syscall_map: HashMap<i32, &'static str>,
}

impl SyscallAnalyzer {
    pub fn new() -> Self {
        let mut syscall_map = HashMap::new();
        
        syscall_map.insert(0, "entrypoint");
        syscall_map.insert(1, "sol_log_");
        syscall_map.insert(2, "sol_log_64_");
        syscall_map.insert(3, "sol_invoke_signed_c");
        syscall_map.insert(4, "sol_pubkey_");
        syscall_map.insert(5, "sol_alloc_free_");
        syscall_map.insert(6, "sol_keccak256_");
        syscall_map.insert(7, "sol_secp256k1_recover_");
        syscall_map.insert(8, "sol_create_program_address_");
        syscall_map.insert(9, "sol_try_find_program_address_");
        syscall_map.insert(10, "sol_sha256_");
        syscall_map.insert(11, "sol_blake3_");
        
        Self { syscall_map }
    }
    
    /// Get the name of a syscall by its number
    pub fn get_syscall_name(&self, syscall_num: i32) -> Option<&'static str> {
        self.syscall_map.get(&syscall_num).copied()
    }
    
    /// Add a custom syscall mapping
    pub fn add_syscall(&mut self, num: i32, name: &'static str) {
        self.syscall_map.insert(num, name);
    }
}

impl AugerAnalyzer for SyscallAnalyzer {
    fn name(&self) -> &'static str {
        "syscall_analyzer"
    }

    fn find_functions(&self, memory_map: &MemoryMap) -> Vec<FunctionBlock> {
        info!("Finding syscall functions");
        let mut functions = Vec::new();
        
        // Look for syscall patterns
        for instr in memory_map.get_instructions() {
            if instr.opcode == OpCode::Call { // CALL
                if let Some(syscall_name) = self.get_syscall_name(instr.imm) {
                    // Found a syscall
                    functions.push(FunctionBlock {
                        address: instr.address,
                        name: syscall_name.to_string(),
                        size: 8, // Single instruction
                        instructions: vec![instr.clone()],
                    });
                }
            }
        }
        
        debug!("Found {} syscall functions", functions.len());
        functions
    }

    fn map_control_flow(&self, memory_map: &MemoryMap, functions: &[FunctionBlock]) -> Vec<ControlFlow> {
        info!("Mapping syscall control flow");
        let mut control_flow = Vec::new();
        
        // Map calls to syscalls
        for instr in memory_map.get_instructions() {
            if instr.opcode == OpCode::Call { // CALL
                if let Some(_syscall_name) = self.get_syscall_name(instr.imm) {
                    // This is a syscall
                    if let Some(caller) = functions.iter().find(|f| {
                        f.address <= instr.address && 
                        instr.address < f.address + f.size as u64
                    }) {
                        control_flow.push(ControlFlow::Call {
                            from_addr: instr.address,
                            to_addr: instr.imm as u64,
                            from_func: caller.address,
                            to_func: instr.imm as u64,
                        });
                    }
                }
            }
        }
        
        debug!("Found {} syscall control flow edges", control_flow.len());
        control_flow
    }

    fn find_memory_refs(&self, memory_map: &MemoryMap) -> Vec<MemoryReference> {
        info!("Finding syscall memory references");
        let mut references = Vec::new();
        
        for instr in memory_map.get_instructions() {
            if instr.opcode == OpCode::Call { // CALL
                if self.get_syscall_name(instr.imm).is_some() {
                    let start_addr = instr.address.saturating_sub(32); 
                    
                    for prev in memory_map.get_instructions().iter()
                        .filter(|i| i.address >= start_addr && i.address < instr.address)
                    {
                        match prev.opcode {
                            OpCode::Ldxw | OpCode::Ldxh | OpCode::Ldxb | OpCode::Ldxdw => { // LDXW, LDXH, LDXB, LDXDW
                                references.push(MemoryReference {
                                    address: prev.address,
                                    target: prev.dst_reg as u64 + prev.imm as u64,
                                    size: match prev.opcode {
                                        OpCode::Ldxw => 4, // LDXW
                                        OpCode::Ldxh => 2, // LDXH
                                        OpCode::Ldxb => 1, // LDXB
                                        OpCode::Ldxdw => 8, // LDXDW
                                        _ => 0,
                                    },
                                    is_write: false,
                                });
                            },
                            _ => {}
                        }
                    }
                }
            }
        }
        
        debug!("Found {} syscall memory references", references.len());
        references
    }

    fn can_handle(&self, _memory_map: &MemoryMap) -> bool {
        // Syscall analyzer can handle any Solana program
        true
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use ezbpf_core::program::Program;
    use std::path::PathBuf;

    fn create_test_memory_map() -> MemoryMap {
        let test_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("analyzers")
            .join("tests")
            .join("fixtures")
            .join("fib.so");

        let bytes = std::fs::read(test_file).expect("Failed to read test fixture fib.so");
        let program = Program::from_bytes(&bytes).expect("Failed to parse test fixture fib.so");
        
        MemoryMap::new(&program, &bytes)
    }

    #[test]
    fn test_syscall_mapping() {
        let analyzer = SyscallAnalyzer::new();
        
        assert_eq!(analyzer.get_syscall_name(0), Some("entrypoint"));
        assert_eq!(analyzer.get_syscall_name(1), Some("sol_log_"));
        assert_eq!(analyzer.get_syscall_name(2), Some("sol_log_64_"));
        
        assert_eq!(analyzer.get_syscall_name(100), None);
        
        let mut analyzer = SyscallAnalyzer::new();
        analyzer.add_syscall(100, "custom_syscall");
        assert_eq!(analyzer.get_syscall_name(100), Some("custom_syscall"));
    }

    #[test]
    fn test_find_syscall_functions() {
        let analyzer = SyscallAnalyzer::new();
        let memory_map = create_test_memory_map();
        
        let functions = analyzer.find_functions(&memory_map);
        assert!(functions.len() >= 4);
        
        let sol_log = functions.iter().find(|f| f.name == "sol_log_").unwrap();
        assert_eq!(sol_log.address, 0x8);
        assert_eq!(sol_log.size, 8);
        
        let sol_log_64 = functions.iter().find(|f| f.name == "sol_log_64_").unwrap();
        assert_eq!(sol_log_64.address, 0x18);
        assert_eq!(sol_log_64.size, 8);
    }

    #[test]
    fn test_map_syscall_control_flow() {
        let analyzer = SyscallAnalyzer::new();
        let memory_map = create_test_memory_map();
        let functions = analyzer.find_functions(&memory_map);
        println!("{}", functions.len());
        assert!(functions.len() >= 2);
        
        let control_flow = analyzer.map_control_flow(&memory_map, &functions);
        assert!(control_flow.len() > 0);
        
        let sol_log_call = control_flow.iter().find(|cf| match cf {
            ControlFlow::Call { to_addr, .. } => {
                println!("Found call to sol_log: {}", to_addr);
                *to_addr == 1
            },
            _ => false,
        }).unwrap();
        
        match sol_log_call {
            ControlFlow::Call { from_addr, to_addr, .. } => {
                println!("Found call to sol_log: {}", to_addr);
                assert_eq!(*from_addr, 0x8);
                assert_eq!(*to_addr, 1);
            }
            _ => panic!("Expected Call control flow"),
        }
        
        let sol_log_64_call = control_flow.iter().find(|cf| match cf {
            ControlFlow::Call { to_addr, .. } => {
                println!("Found call to sol_log_64: {}", to_addr);
                *to_addr == 2
            },
            _ => false,
        }).unwrap();
        
        match sol_log_64_call {
            ControlFlow::Call { from_addr, to_addr, .. } => {
                println!("Found call to sol_log_64: {}", to_addr);
                assert_eq!(*from_addr, 0x18);
                assert_eq!(*to_addr, 2);
            }
            _ => panic!("Expected Call control flow"),
        }
    }

    #[test]
    fn test_find_syscall_memory_refs() {
        let analyzer = SyscallAnalyzer::new();
        let memory_map = create_test_memory_map();
        
        let refs = analyzer.find_memory_refs(&memory_map);
        assert_eq!(refs.len(), 0); // No memory refs in test data
        
        // TODO: test actual memory refs when we have load/store instructions before syscalls in the test data
    }
}
