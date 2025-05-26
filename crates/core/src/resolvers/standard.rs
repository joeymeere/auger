use crate::memory::MemoryMap;
use crate::models::{MemoryAccess, RustType, StructType, StructField, PrimitiveType, TypeRegistry};
use ezbpf_core::opcodes::OpCode;
use log::info;

use crate::traits::resolver::AugerResolver;

pub struct StandardTypeResolver;

impl AugerResolver for StandardTypeResolver {
    fn name(&self) -> &'static str {
        "standard_resolver"
    }
    
    fn resolve(&self, memory_map: &MemoryMap, type_registry: &mut TypeRegistry) {
        self.identify_std_string_patterns(memory_map, type_registry);
        self.identify_std_vec_patterns(memory_map, type_registry);
        self.identify_std_hash_map_patterns(memory_map, type_registry);
    }
    
    fn can_handle(&self, _access: &MemoryAccess) -> bool {
        true
    }
}

impl StandardTypeResolver {
    pub fn new() -> Self {
        Self
    }
    
    fn identify_std_string_patterns(&self, memory_map: &MemoryMap, type_registry: &mut TypeRegistry) {
        let instructions = memory_map.get_instructions();
        
        // string (ptr, len, capacity)
        for i in 0..instructions.len().saturating_sub(2) {
            let instr1 = &instructions[i];
            let instr2 = &instructions[i+1];
            let instr3 = &instructions[i+2];
            if instr1.opcode == OpCode::Ldxdw && // ldxdw for pointer
               instr2.opcode == OpCode::Ldxdw && // ldxdw for length
               instr3.opcode == OpCode::Ldxdw && // ldxdw for capacity
               instr1.offset == 0 &&
               instr2.offset == 8 &&
               instr3.offset == 16 {
                
                let fields = vec![
                    StructField {
                        name: Some("ptr".to_string()),
                        field_type: Box::new(RustType::Primitive(PrimitiveType::new("*const u8", 8))),
                        offset: 0,
                    },
                    StructField {
                        name: Some("len".to_string()),
                        field_type: Box::new(RustType::Primitive(PrimitiveType::new("usize", 8))),
                        offset: 8,
                    },
                    StructField {
                        name: Some("capacity".to_string()),
                        field_type: Box::new(RustType::Primitive(PrimitiveType::new("usize", 8))),
                        offset: 16,
                    },
                ];
                
                let struct_type = StructType {
                    name: "std::string::String".to_string(),
                    fields,
                    size: 24,
                    alignment: 8,
                };
                
                type_registry.register_struct(struct_type);
            }
        }
    }
    
    fn identify_std_vec_patterns(&self, memory_map: &MemoryMap, type_registry: &mut TypeRegistry) {
        let instructions = memory_map.get_instructions();
        
        // vec (ptr, len, capacity)
        for i in 0..instructions.len().saturating_sub(2) {
            let instr1 = &instructions[i];
            let instr2 = &instructions[i+1];
            let instr3 = &instructions[i+2];
            if instr1.opcode == OpCode::Ldxdw && // ldxdw for pointer
               instr2.opcode == OpCode::Ldxdw && // ldxdw for length
               instr3.opcode == OpCode::Ldxdw && // ldxdw for capacity
               instr1.offset == 0 &&
               instr2.offset == 8 &&
               instr3.offset == 16 {
                let fields = vec![
                    StructField {
                        name: Some("ptr".to_string()),
                        field_type: Box::new(RustType::Primitive(PrimitiveType::new("*const T", 8))),
                        offset: 0,
                    },
                    StructField {
                        name: Some("len".to_string()),
                        field_type: Box::new(RustType::Primitive(PrimitiveType::new("usize", 8))),
                        offset: 8,
                    },
                    StructField {
                        name: Some("capacity".to_string()),
                        field_type: Box::new(RustType::Primitive(PrimitiveType::new("usize", 8))),
                        offset: 16,
                    },
                ];
                
                let struct_type = StructType {
                    name: "std::vec::Vec<T>".to_string(),
                    fields,
                    size: 24,
                    alignment: 8,
                };
                
                type_registry.register_struct(struct_type);
            }
        }
    }
    
    fn identify_std_hash_map_patterns(&self, memory_map: &MemoryMap, type_registry: &mut TypeRegistry) {
        let instructions = memory_map.get_instructions();
        
        // hashmap
        for i in 0..instructions.len().saturating_sub(3) {
            let instr1 = &instructions[i];
            let instr2 = &instructions[i+1];
            let instr3 = &instructions[i+2];
            let instr4 = &instructions[i+3];
            if instr1.opcode == OpCode::Ldxdw && // ldxdw for hash_builder
               instr2.opcode == OpCode::Ldxdw && // ldxdw  for bucket_mask
               instr3.opcode == OpCode::Ldxdw && // ldxdw  for ctrl
               instr4.opcode == OpCode::Ldxdw && //ldxdw  for growth_left
               instr1.offset == 0 &&
               instr2.offset == 8 &&
               instr3.offset == 16 &&
               instr4.offset == 24 {
                let fields = vec![
                    StructField {
                        name: Some("hash_builder".to_string()),
                        field_type: Box::new(RustType::Unknown),
                        offset: 0,
                    },
                    StructField {
                        name: Some("table.bucket_mask".to_string()),
                        field_type: Box::new(RustType::Primitive(PrimitiveType::new("usize", 8))),
                        offset: 8,
                    },
                    StructField {
                        name: Some("table.ctrl".to_string()),
                        field_type: Box::new(RustType::Primitive(PrimitiveType::new("u8", 1))),
                        offset: 16,
                    },
                    StructField {
                        name: Some("table.growth_left".to_string()),
                        field_type: Box::new(RustType::Primitive(PrimitiveType::new("usize", 8))),
                        offset: 24,
                    },
                ];
                
                let struct_type = StructType {
                    name: "std::collections::HashMap<K, V>".to_string(),
                    fields,
                    size: 32,
                    alignment: 8,
                };
                
                type_registry.register_struct(struct_type);
            }
        }
    }
}
