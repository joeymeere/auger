use std::collections::HashMap;
use ezbpf_core::opcodes::OpCode;
use log::{debug, info};

use crate::memory::MemoryMap;
use crate::models::{FunctionBlock, ControlFlow, MemoryReference};
use crate::traits::analyzer::AugerAnalyzer;

pub struct BaseAnalyzer {
    function_cache: HashMap<u64, FunctionBlock>,
    control_flow_cache: HashMap<u64, Vec<ControlFlow>>,
    memory_ref_cache: HashMap<u64, Vec<MemoryReference>>,
}

impl BaseAnalyzer {
    pub fn new() -> Self {
        Self {
            function_cache: HashMap::new(),
            control_flow_cache: HashMap::new(),
            memory_ref_cache: HashMap::new(),
        }
    }

    pub fn clear_caches(&mut self) {
        self.function_cache.clear();
        self.control_flow_cache.clear();
        self.memory_ref_cache.clear();
    }

    pub fn get_cached_functions(&self) -> Vec<&FunctionBlock> {
        self.function_cache.values().collect()
    }

    pub fn get_cached_control_flow(&self, function_addr: u64) -> Option<&Vec<ControlFlow>> {
        self.control_flow_cache.get(&function_addr)
    }

    pub fn get_cached_memory_refs(&self) -> Vec<&MemoryReference> {
        self.memory_ref_cache.values().flatten().collect()
    }
}

impl AugerAnalyzer for BaseAnalyzer {
    fn name(&self) -> &'static str {
        "base_analyzer"
    }

    fn find_functions(&self, memory_map: &MemoryMap) -> Vec<FunctionBlock> {
        info!("Finding function blocks");
        let mut functions = Vec::new();
        let instructions = memory_map.get_instructions();
        
        // - ixs after unconditional jumps
        // - ixs referenced by call instructions
        // - ixs at the start of sections
        let mut current_block = None;
        
        for (i, instr) in instructions.iter().enumerate() {
            let is_func_start = if i == 0 {
                true // first ix is function start
            } else {
                let prev_instr = &instructions[i-1];
                // after unconditional jump
                prev_instr.opcode == OpCode::Ja ||
                // after return
                prev_instr.opcode == OpCode::Exit ||
                // target of a call
                instructions.iter().any(|i| {
                    i.opcode == OpCode::Call && 
                    i.imm as u64 == instr.address
                })
            };

            if is_func_start {
                if let Some(block) = current_block.take() {
                    functions.push(block);
                }
                
                current_block = Some(FunctionBlock {
                    address: instr.address,
                    name: format!("func_{:x}", instr.address),
                    size: 0,
                    instructions: vec![instr.clone()],
                });
            } else if let Some(block) = &mut current_block {
                block.instructions.push(instr.clone());
                block.size += 8;
            }
        }

        if let Some(block) = current_block {
            functions.push(block);
        }

        debug!("Found {} function blocks", functions.len());
        functions
    }

    fn map_control_flow(&self, _memory_map: &MemoryMap, functions: &[FunctionBlock]) -> Vec<ControlFlow> {
        info!("Mapping control flow");
        let mut control_flow = Vec::new();

        for function in functions {
            for instr in &function.instructions {
                match instr.opcode {
                    OpCode::Call => {
                        let target_addr = instr.imm as u64;
                        println!("Found call to {:x}", target_addr);
                        if let Some(target) = functions.iter().find(|f| {
                            println!("Comparing {:x} == {:x}", f.address, target_addr);
                            f.address == target_addr
                        }) {
                            println!("Adding...");
                            control_flow.push(ControlFlow::Call {
                                from_addr: instr.address,
                                to_addr: target_addr,
                                from_func: function.address,
                                to_func: target.address,
                            });
                        }
                    },
                    
                    // conditional jumps
                    OpCode::JeqImm | OpCode::JeqReg | OpCode::JneImm | OpCode::JneReg | 
                    OpCode::JltImm | OpCode::JltReg | OpCode::JleImm | OpCode::JleReg | 
                    OpCode::JgeImm | OpCode::JgeReg | OpCode::JgtImm | OpCode::JgtReg => {
                        // Jump offset is relative to next instruction (current + 8)
                        let target_addr = (instr.address + 8).wrapping_add(instr.imm as u64);
                        println!("Found jump to {:x} (offset: {:x})", target_addr, instr.imm);
                        if let Some(target) = functions.iter().find(|f| {
                            println!("Comparing {:x} == {:x}", f.address, target_addr);
                            f.address == target_addr
                        }) {
                            println!("Adding...");
                            control_flow.push(ControlFlow::Jump {
                                from_addr: instr.address,
                                to_addr: target_addr,
                                from_func: function.address,
                                to_func: target.address,
                                conditional: true,
                            });
                        }
                    },
                    
                    // unconditional jumps
                    OpCode::Ja | OpCode::Exit => {
                        // jump offset is relative to next instruction (current + 8)
                        let target_addr = (instr.address + 8).wrapping_add(instr.imm as u64);
                        println!("Found jump to {:x} (offset: {:x})", target_addr, instr.imm);
                        if let Some(target) = functions.iter().find(|f| {
                            println!("Comparing {:x} == {:x}", f.address, target_addr);
                            f.address == target_addr
                        }) {
                            println!("Adding...");
                            control_flow.push(ControlFlow::Jump {
                                from_addr: instr.address,
                                to_addr: target_addr,
                                from_func: function.address,
                                to_func: target.address,
                                conditional: false,
                            });
                        }
                    },
                    
                    _ => {}
                }
            }
        }

        debug!("Found {} control flow edges", control_flow.len());
        control_flow
    }

    fn find_memory_refs(&self, memory_map: &MemoryMap) -> Vec<MemoryReference> {
        info!("Finding memory references");
        let mut references = Vec::new();

        for instr in memory_map.get_instructions() {
            match instr.opcode {
                // load ixs
                OpCode::Ldxw | OpCode::Ldxh | OpCode::Ldxb | OpCode::Ldxdw => {
                    references.push(MemoryReference {
                        address: instr.address,
                        target: instr.dst_reg as u64 + instr.imm as u64,
                        size: match instr.opcode {
                            OpCode::Ldxw => 4,
                            OpCode::Ldxh => 2,
                            OpCode::Ldxb => 1,
                            OpCode::Ldxdw => 8,
                            _ => 0,
                        },
                        is_write: false,
                    });
                },
                
                // store ixs 
                OpCode::Stxw | OpCode::Stxh | OpCode::Stxb | OpCode::Stxdw => {
                    references.push(MemoryReference {
                        address: instr.address,
                        target: instr.dst_reg as u64 + instr.imm as u64,
                        size: match instr.opcode {
                            OpCode::Stxw => 4,
                            OpCode::Stxh => 2,
                            OpCode::Stxb => 1,
                            OpCode::Stxdw => 8,
                            _ => 0,
                        },
                        is_write: true,
                    });
                },
                
                _ => {}
            }
        }

        debug!("Found {} memory references", references.len());
        references
    }

    fn can_handle(&self, _memory_map: &MemoryMap) -> bool {
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
    fn test_find_functions() {
        let analyzer = BaseAnalyzer::new();
        let memory_map = create_test_memory_map();

        println!("Memory map: | Ixs Length: {:?}, Sections: {:?}, Syscalls: {:?} |", memory_map.instructions.len(), memory_map.sections.len(), memory_map.syscall_signatures.len());

        let functions = analyzer.find_functions(&memory_map);
        assert!(functions.len() >= 2);
        
        let func1 = match functions.iter().find(|f| f.address == 0x0000e8) {
            Some(func) => func,
            None => panic!("Failed to find function with address 0x0000e8"),
        };
        println!("{:?}", func1);
        assert_eq!(func1.size, 40);
        assert!(func1.instructions.len() >= 3);
    }

    #[test]
    fn test_map_control_flow() {
        let analyzer = BaseAnalyzer::new();
        let memory_map = create_test_memory_map();
        let functions = analyzer.find_functions(&memory_map);
        
        let control_flow = analyzer.map_control_flow(&memory_map, &functions);
        assert!(control_flow.len() > 0);

        println!("{:?}", control_flow[0]);
        
        match &control_flow[0] {
            ControlFlow::Jump { from_addr, to_addr, from_func, to_func, conditional } => {
                assert_eq!(*from_addr, 0x110);
                assert_eq!(*to_addr, 0x118);
                assert_eq!(*from_func, 0xE8);
                assert_eq!(*to_func, 0x118);
                assert_eq!(*conditional, false);
            }
            _ => panic!("Expected Jump control flow"),
        }
    }

    #[test]
    fn test_find_memory_refs() {
        let analyzer = BaseAnalyzer::new();
        let memory_map = create_test_memory_map();
        
        let refs = analyzer.find_memory_refs(&memory_map);
        assert!(refs.len() >= 1);
        
        let mem_ref = &refs[0];
        println!("{:?}", mem_ref);
        //assert_eq!(mem_ref.address, 0x10);
        assert_eq!(mem_ref.size, 1); // LDXW = 4 bytes
        //assert!(!mem_ref.is_write);
    }
}
