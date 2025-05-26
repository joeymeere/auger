use crate::memory::MemoryMap;
use crate::models::{MemoryAccess, DataReference, PrimitiveType, RustType, StringType, StructField, StructType, TypeRegistry};
use ezbpf_core::opcodes::OpCode;
use log::debug;

use crate::traits::resolver::AugerResolver;

pub struct StructResolver;

impl AugerResolver for StructResolver {
    fn name(&self) -> &'static str {
        "struct_resolver"
    }
    
    fn resolve(&self, memory_map: &MemoryMap, type_registry: &mut TypeRegistry) {
        let potential_structs = memory_map.get_access_patterns(0x0, 0x100_0000);
        for acc in potential_structs {
            debug!("Analyzing potential struct at 0x{:x} with size {}", acc.address, acc.size);
            
            let accesses = memory_map.get_access_patterns(acc.address, acc.size as u64);
            
            let mut field_accesses: Vec<_> = accesses.iter()
                .map(|access| (access.address - acc.address, access))
                .collect();
            field_accesses.sort_by_key(|(offset, _)| *offset);
            
            // follow mem access patterns
            let mut fields = Vec::new();
            let mut current_offset = 0;
            
            for (offset, access) in field_accesses {
                if offset < current_offset {
                    continue;
                }
                
                let field_type = match access.size {
                    1 => RustType::Primitive(PrimitiveType::new("u8", 1)),
                    2 => RustType::Primitive(PrimitiveType::new("u16", 2)),
                    4 => {
                        // u32 or char?
                        if self.is_likely_char(access) {
                            RustType::Primitive(PrimitiveType::new("char", 4))
                        } else {
                            RustType::Primitive(PrimitiveType::new("u32", 4))
                        }
                    },
                    8 => {
                        // u64 or ptr?
                        if self.is_likely_pointer(access) {
                            if self.is_likely_string_ptr(access) {
                                RustType::String(StringType::new(false))
                            } else {
                                // generic ptr
                                RustType::Primitive(PrimitiveType::new("*const u8", 8))
                            }
                        } else {
                            RustType::Primitive(PrimitiveType::new("u64", 8))
                        }
                    },
                    _ => continue, 
                };
                
                fields.push(StructField {
                    name: Some(format!("field_{}", fields.len())),
                    field_type: Box::new(field_type),
                    offset: offset as usize,
                });
                
                current_offset = offset + access.size as u64;
            }
            
            if !fields.is_empty() {
                let struct_type = StructType {
                    name: format!("Struct_{:x}", acc.address),
                    fields,
                    size: acc.size as usize,
                    alignment: 8, // assume 8-byte alignment, prob wrong
                };
                
                debug!("Recovered struct type: {}", struct_type.name);
                type_registry.register_struct(struct_type);
            }
        }
    }
    
    fn can_handle(&self, _access: &MemoryAccess) -> bool {
        true
    }
}

impl StructResolver {
    pub fn new() -> Self {
        Self
    }
    
    fn is_likely_char(&self, access: &MemoryAccess) -> bool {
        if let Some(instr) = &access.instruction.instruction {
            matches!(instr.op,
                OpCode::JeqImm | OpCode::JneImm) &&
            (0x20..=0x7E).contains(&(access.instruction.imm)) 
        } else {
            false
        }
    }
    
    fn is_likely_pointer(&self, access: &MemoryAccess) -> bool {
        if let Some(instr) = &access.instruction.instruction {
            matches!(instr.op,
                OpCode::Ldxw | OpCode::Ldxh | OpCode::Ldxb | OpCode::Ldxdw |
                OpCode::Stxw | OpCode::Stxh | OpCode::Stxb | OpCode::Stxdw)
        } else {
            false
        }
    }
    
    fn is_likely_string_ptr(&self, access: &MemoryAccess) -> bool {
        if let Some(DataReference::String(_)) = &access.instruction.references {
            true
        } else {
            false
        }
    }
}
