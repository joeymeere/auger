use std::collections::HashMap;

use super::{ArrayType, EnumType, FunctionType, PrimitiveType, RustType, StringType, StructType, VectorType};

pub struct TypeRegistry {
    /// Map of type ID to type
    types: HashMap<u64, RustType>,
    /// Map of type name to type ID
    type_names: HashMap<String, u64>,
    /// Next available type ID
    next_id: u64,
}

impl TypeRegistry {
    pub fn new() -> Self {
        Self {
            types: HashMap::new(),
            type_names: HashMap::new(),
            next_id: 1, // Start at 1, 0 can be reserved for "unknown"
        }
    }
    
    pub fn register_primitive(&mut self, name: &str, size: usize) -> u64 {
        let primitive = PrimitiveType {
            name: name.to_string(),
            size,
        };
        
        self.register_type(RustType::Primitive(primitive), Some(name.to_string()))
    }
    
    pub fn register_string(&mut self, string_type: StringType) -> u64 {
        self.register_type(RustType::String(string_type), Some("String".to_string()))
    }
    
    pub fn register_vector(&mut self, vector_type: VectorType) -> u64 {
        let description = format!("Vec<{}>", vector_type.element_type.description());
        self.register_type(RustType::Vector(vector_type), Some(description))
    }
    
    pub fn register_array(&mut self, array_type: ArrayType) -> u64 {
        let description = format!("[{}; {}]", array_type.element_type.description(), array_type.length);
        self.register_type(RustType::Array(array_type), Some(description))
    }
    
    pub fn register_struct(&mut self, struct_type: StructType) -> u64 {
        let name = struct_type.name.clone();
        self.register_type(RustType::Struct(struct_type), Some(name))
    }
    
    pub fn register_enum(&mut self, enum_type: EnumType) -> u64 {
        let name = enum_type.name.clone();
        self.register_type(RustType::Enum(enum_type), Some(name))
    }
    
    pub fn register_option(&mut self, inner_type: RustType) -> u64 {
        let description = format!("Option<{}>", inner_type.description());
        self.register_type(RustType::Option(Box::new(inner_type)), Some(description))
    }
    
    pub fn register_result(&mut self, ok_type: RustType, err_type: RustType) -> u64 {
        let description = format!("Result<{}, {}>", ok_type.description(), err_type.description());
        self.register_type(
            RustType::Result(Box::new(ok_type), Box::new(err_type)),
            Some(description)
        )
    }
    
    pub fn register_box(&mut self, inner_type: RustType) -> u64 {
        let description = format!("Box<{}>", inner_type.description());
        self.register_type(RustType::Box(Box::new(inner_type)), Some(description))
    }
    
    pub fn register_reference(&mut self, inner_type: RustType) -> u64 {
        let description = format!("&{}", inner_type.description());
        self.register_type(RustType::Reference(Box::new(inner_type)), Some(description))
    }
    
    pub fn register_function(&mut self, function_type: FunctionType) -> u64 {
        self.register_type(RustType::Function(function_type), None)
    }
    
    fn register_type(&mut self, rust_type: RustType, name: Option<String>) -> u64 {
        let type_id = self.next_id;
        self.next_id += 1;
        
        self.types.insert(type_id, rust_type);
        
        if let Some(name) = name {
            self.type_names.insert(name, type_id);
        }
        
        type_id
    }
    
    pub fn get_type(&self, type_id: u64) -> Option<&RustType> {
        self.types.get(&type_id)
    }
    
    pub fn get_type_id(&self, name: &str) -> Option<u64> {
        self.type_names.get(name).copied()
    }
    
    pub fn get_type_name(&self, type_id: u64) -> String {
        if let Some(rust_type) = self.get_type(type_id) {
            rust_type.description()
        } else {
            "unknown".to_string()
        }
    }
    
    pub fn get_type_description(&self, rust_type: &RustType) -> String {
        rust_type.description()
    }
    
    pub fn get_all_structs(&self) -> Vec<&StructType> {
        self.types.values()
            .filter_map(|t| {
                if let RustType::Struct(s) = t {
                    Some(s)
                } else {
                    None
                }
            })
            .collect()
    }
    
    pub fn get_all_enums(&self) -> Vec<&EnumType> {
        self.types.values()
            .filter_map(|t| {
                if let RustType::Enum(e) = t {
                    Some(e)
                } else {
                    None
                }
            })
            .collect()
    }
    
    pub fn get_all_arrays(&self) -> Vec<&ArrayType> {
        self.types.values()
            .filter_map(|t| {
                if let RustType::Array(a) = t {
                    Some(a)
                } else {
                    None
                }
            })
            .collect()
    }
    
    pub fn get_all_vectors(&self) -> Vec<&VectorType> {
        self.types.values()
            .filter_map(|t| {
                if let RustType::Vector(v) = t {
                    Some(v)
                } else {
                    None
                }
            })
            .collect()
    }
}