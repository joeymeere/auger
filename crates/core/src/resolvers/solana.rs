use crate::memory::MemoryMap;
use crate::models::{MemoryAccess, RustType, StructType, StructField, PrimitiveType, ArrayType, TypeRegistry};
use log::{debug, info};

use crate::traits::resolver::AugerResolver;

pub struct SolanaTypeResolver;

impl AugerResolver for SolanaTypeResolver {
    fn name(&self) -> &'static str {
        "solana_resolver"
    }
    
    fn resolve(&self, memory_map: &MemoryMap, type_registry: &mut TypeRegistry) {
        info!("Identifying standard Solana types");
        
        let patterns = [
            // Pubkey pattern (32 bytes)
            (32, "Pubkey"),
            // Account Info (pubkey + lamports + data)
            (8 + 8 + 32, "AccountInfo"),
            // Instruction data (variable length buffer)
            (4 + 8, "InstructionData"),
        ];
        
        for acc in memory_map.get_access_patterns(0x0, 0x100_0000) {
            for &(pattern_size, type_name) in &patterns {
                if acc.size == pattern_size {
                    debug!("Found potential {} at 0x{:x}", type_name, 0x0);
                    match type_name {
                        "Pubkey" => {
                            let pubkey_type = StructType {
                                name: "solana_program::pubkey::Pubkey".to_string(),
                                fields: vec![StructField {
                                    name: Some("bytes".to_string()),
                                    field_type: Box::new(RustType::Array(ArrayType::new(
                                        RustType::Primitive(PrimitiveType::new("u8", 1)),
                                        32
                                    ))),
                                    offset: 0,
                                }],
                                size: 32,
                                alignment: 1,
                            };
                            type_registry.register_struct(pubkey_type);
                        },
                        "AccountInfo" => {
                            let account_info_type = StructType {
                                name: "solana_program::account_info::AccountInfo".to_string(),
                                fields: vec![
                                    StructField {
                                        name: Some("key".to_string()),
                                        field_type: Box::new(RustType::Struct(StructType {
                                            name: "solana_program::pubkey::Pubkey".to_string(),
                                            fields: vec![],
                                            size: 32,
                                            alignment: 1,
                                        })),
                                        offset: 0,
                                    },
                                    StructField {
                                        name: Some("lamports".to_string()),
                                        field_type: Box::new(RustType::Primitive(PrimitiveType::new("u64", 8))),
                                        offset: 32,
                                    },
                                    StructField {
                                        name: Some("data_len".to_string()),
                                        field_type: Box::new(RustType::Primitive(PrimitiveType::new("u64", 8))),
                                        offset: 40,
                                    },
                                ],
                                size: 48,
                                alignment: 8,
                            };
                            type_registry.register_struct(account_info_type);
                        },
                        _ => (), 
                    }
                }
            }
        }
    }
    
    fn can_handle(&self, _access: &MemoryAccess) -> bool {
        true
    }
}

impl SolanaTypeResolver {
    pub fn new() -> Self {
        Self
    }
}
