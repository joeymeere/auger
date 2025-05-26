use super::{ArrayType, EnumType, FunctionType, VectorType};

#[derive(Debug, Clone)]
pub enum RustType {
    Primitive(PrimitiveType),
    String(StringType),
    Vector(VectorType),
    Array(ArrayType),
    Struct(StructType),
    Enum(EnumType),
    Option(Box<RustType>),
    Result(Box<RustType>, Box<RustType>),
    Box(Box<RustType>),
    Reference(Box<RustType>),
    Function(FunctionType),
    Unknown,
}

impl RustType {
    pub fn size(&self) -> usize {
        match self {
            RustType::Primitive(p) => p.size,
            RustType::String(_) => 24, // ptr + len + capacity
            RustType::Vector(_) => 24, // ptr + len + capacity
            RustType::Array(a) => a.total_size,
            RustType::Struct(s) => s.size,
            RustType::Enum(e) => e.size,
            RustType::Option(inner) => inner.size() + 1, // tag + inner
            RustType::Result(ok, err) => {
                // size is tag + max(ok, err)
                1 + std::cmp::max(ok.size(), err.size())
            },
            RustType::Box(_) => 8, // heap pointer
            RustType::Reference(_) => 8, // pointer
            RustType::Function(_) => 8, // function pointer
            RustType::Unknown => 0,
        }
    }
    
    pub fn alignment(&self) -> usize {
        match self {
            RustType::Primitive(p) => p.size, // primitives are aligned to their size
            RustType::String(_) => 8,
            RustType::Vector(_) => 8,
            RustType::Array(a) => a.element_type.alignment(), 
            RustType::Struct(s) => s.alignment,
            RustType::Enum(e) => e.alignment,
            RustType::Option(inner) => inner.alignment(),
            RustType::Result(ok, err) => std::cmp::max(ok.alignment(), err.alignment()),
            RustType::Box(_) => 8, 
            RustType::Reference(_) => 8,
            RustType::Function(_) => 8,
            RustType::Unknown => 1,
        }
    }
    
    pub fn description(&self) -> String {
        match self {
            RustType::Primitive(p) => p.name.to_string(),
            RustType::String(_) => "String".to_string(),
            RustType::Vector(v) => format!("Vec<{}>", v.element_type.description()),
            RustType::Array(a) => format!("[{}; {}]", a.element_type.description(), a.length),
            RustType::Struct(s) => s.name.clone(),
            RustType::Enum(e) => e.name.clone(),
            RustType::Option(inner) => format!("Option<{}>", inner.description()),
            RustType::Result(ok, err) => format!("Result<{}, {}>", ok.description(), err.description()),
            RustType::Box(inner) => format!("Box<{}>", inner.description()),
            RustType::Reference(inner) => format!("&{}", inner.description()),
            RustType::Function(_) => "fn()".to_string(), // Simplified
            RustType::Unknown => "unknown".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PrimitiveType {
    pub name: String,  // "i32", "u64", etc.
    pub size: usize,
}

impl PrimitiveType {
    pub fn new(name: &str, size: usize) -> Self {
        Self { name: name.to_string(), size }
    }
}

#[derive(Debug, Clone)]
pub struct StringType {
    // Rust `String` is ptr, len, capacity
    pub is_static: bool,
}

impl StringType {
    pub fn new(is_static: bool) -> Self {
        Self { is_static }
    }
}

#[derive(Debug, Clone)]
pub struct StructType {
    pub name: String,
    pub fields: Vec<StructField>,
    pub size: usize,
    pub alignment: usize,
}

#[derive(Debug, Clone)]
pub struct StructField {
    pub name: Option<String>,
    pub offset: usize,
    pub field_type: Box<RustType>,
}

#[derive(Debug, Clone)]
pub struct SliceType {
    pub element_type: Box<RustType>,
    // Rust slices are fat pointers: data pointer + length
    pub is_mut: bool,
}

impl SliceType {
    fn description(&self) -> String {
        let mut_part = if self.is_mut { "mut " } else { "" };
        format!("&{}", mut_part)
    }
    
    fn is_str_slice(&self) -> bool {
        if let RustType::Primitive(primitive) = &*self.element_type {
            primitive.name == "u8" || primitive.name == "char"
        } else {
            false
        }
    }
    
    fn size(&self) -> usize {
        // typically 16 bytes (8 for pointer + 8 for length)
        16
    }
}
