use super::{StructField, RustType};
use crate::memory::MemoryMap;

#[derive(Debug, Clone)]
pub struct ArrayType {
    pub element_type: Box<RustType>,
    pub length: usize,            // Fixed length for arrays
    pub stride: usize,            // Bytes between elements (accounts for alignment)
    pub total_size: usize,        // Total size of the array in bytes
}

impl ArrayType {
    pub fn new(element_type: RustType, length: usize) -> Self {
        let element_size = element_type.size();
        let _element_align = element_type.alignment();
        
        // Calculate stride (size with alignment)
        // In Rust, array elements are tightly packed without padding if element's
        // alignment requirements are met
        let stride = element_size;
        let total_size = stride * length;
        
        Self {
            element_type: Box::new(element_type),
            length,
            stride,
            total_size,
        }
    }
    
    fn element_offset(&self, index: usize) -> Option<usize> {
        if index < self.length {
            Some(index * self.stride)
        } else {
            None
        }
    }
    
    fn could_be_str(&self, _memory_map: &MemoryMap) -> bool {
        if let RustType::Primitive(primitive) = &*self.element_type {
            if primitive.name == "u8" || primitive.name == "char" {
                return true;
            }
        }
        false
    }
    
    // Special case for detecting zero-sized arrays often used in FFI
    fn is_zero_sized(&self) -> bool {
        self.length == 0
    }
    
    // Special case for flexible array members in FFI structs
    fn is_flexible_array(&self) -> bool {
        self.length == 1 && self.is_at_end_of_struct()
    }
    
    fn is_at_end_of_struct(&self) -> bool {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub struct VectorType {
    pub element_type: Box<RustType>,
    // Vec is ptr, len, capacity
}

#[derive(Debug, Clone)]
pub struct EnumType {
    pub name: String,
    pub variants: Vec<EnumVariant>,
    // Rust enums have discriminant + possibly data
    pub size: usize,
    pub alignment: usize,
}

impl EnumType {
    fn is_option_type(&self) -> Option<Box<RustType>> {
        if self.variants.len() == 2 {
            let has_none = self.variants.iter().any(|v| v.is_option_none());
            let some_variant = self.variants.iter().find(|v| v.is_option_some());
            
            if has_none && some_variant.is_some() {
                if let Some(variant) = some_variant {
                    if let VariantFields::Tuple(types) = &variant.fields {
                        return Some(types[0].clone());
                    }
                }
            }
        }
        None
    }
    
    fn is_result_type(&self) -> Option<(Box<RustType>, Box<RustType>)> {
        if self.variants.len() == 2 {
            let ok_variant = self.variants.iter().find(|v| v.is_result_ok());
            let err_variant = self.variants.iter().find(|v| v.is_result_err());
            
            if ok_variant.is_some() && err_variant.is_some() {
                // Extract both wrapped types
                if let (Some(ok), Some(err)) = (ok_variant, err_variant) {
                    if let (VariantFields::Tuple(ok_types), VariantFields::Tuple(err_types)) = 
                        (&ok.fields, &err.fields) {
                        return Some((ok_types[0].clone(), err_types[0].clone()));
                    }
                }
            }
        }
        None
    }
    
    // C-style enums (just tags, no data)
    fn is_c_style_enum(&self) -> bool {
        self.variants.iter().all(|v| matches!(v.fields, VariantFields::Unit))
    }
    
    fn representation_strategy(&self) -> EnumRepresentation {
        if self.is_c_style_enum() {
            return EnumRepresentation::CStyle;
        }
        if self.is_option_type().is_some() && self.size <= 8 {
            return EnumRepresentation::NicheOptimized;
        }
        EnumRepresentation::Tagged
    }
}

#[derive(Debug, Clone, PartialEq)]
enum EnumRepresentation {
    CStyle,         // Just a discriminant, no data variants
    Tagged,         // Standard discriminant + largest variant size
    NicheOptimized, // Uses available niche bits (like null pointer optimization)
    Custom,         // #[repr()] attribute specified custom layout
}

#[derive(Debug, Clone)]
pub struct EnumVariant {
    pub name: String,
    pub discriminant: Option<i64>,  // Explicit discriminant value 
    pub fields: VariantFields,
    pub size: usize, 
}

#[derive(Debug, Clone)]
enum VariantFields {
    Unit,                         // Unit variant (no data)
    Tuple(Vec<Box<RustType>>),    // Tuple variant (unnamed fields)
    Struct(Vec<StructField>),     // Struct variant (named fields)
}

impl EnumVariant {
    fn total_size(&self, discriminant_size: usize) -> usize {
        discriminant_size + self.size
    }
    pub fn new_unit(name: String, discriminant: Option<i64>) -> Self {
        Self {
            name,
            discriminant,
            fields: VariantFields::Unit,
            size: 0, 
        }
    }
    
    fn is_option_none(&self) -> bool {
        self.name == "None" && matches!(self.fields, VariantFields::Unit)
    }

    fn is_option_some(&self) -> bool {
        self.name == "Some" && 
        matches!(self.fields, VariantFields::Tuple(ref types) if types.len() == 1)
    }
    
    fn is_result_ok(&self) -> bool {
        self.name == "Ok" && 
        matches!(self.fields, VariantFields::Tuple(ref types) if types.len() == 1)
    }
    
    fn is_result_err(&self) -> bool {
        self.name == "Err" && 
        matches!(self.fields, VariantFields::Tuple(ref types) if types.len() == 1)
    }
}