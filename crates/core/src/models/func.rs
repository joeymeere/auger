use super::RustType;

#[derive(Debug, Clone)]
pub struct FunctionType {
    // Return type (None would indicate void/unit return type)
    pub return_type: Option<Box<RustType>>,
    // Parameters list
    pub parameters: Vec<FunctionParameter>,
    // Is this an unsafe function?
    pub is_unsafe: bool,
    // Is this a method (has self parameter)?
    pub is_method: bool,
    // Function attributes derived from binary analysis
    pub attributes: FunctionAttributes,
    // Calling convention (important for FFI functions)
    pub calling_convention: CallingConvention,
    // If a method, what is the parent type?
    pub parent_type: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct FunctionParameter {
    pub name: Option<String>,     // Parameter name if available
    pub param_type: Box<RustType>, // Parameter type
    pub by_reference: bool,       // Is this passed by reference?
}

#[derive(Debug, Clone)]
pub struct FunctionAttributes {
    pub is_inline: bool,          // Was this function inlined?
    pub is_cold: bool,            // Marked as cold (rarely executed)?
    pub is_external: bool,        // External function?
    pub is_exported: bool,        // Exported from this binary?
    pub no_return: bool,          // Function never returns (e.g., panics)?
}

#[derive(Debug, Clone, PartialEq)]
pub enum CallingConvention {
    Rust,                     // Standard Rust calling convention
    C,                        // C calling convention
    System,                   // System calling convention
    FastCall,                 // FastCall convention
    Other(String),            // Other specified convention
}

impl FunctionType {
    fn signature(&self) -> String {
        let mut sig = String::new();
        if self.is_unsafe {
            sig.push_str("unsafe ");
        }
        sig.push_str("fn(");
        let params: Vec<String> = self.parameters.iter()
            .map(|param| {
                let name_part = if let Some(name) = &param.name {
                    format!("{}: ", name)
                } else {
                    String::new()
                };
                
                let ref_part = if param.by_reference {
                    "&"
                } else {
                    ""
                };
                
                format!("{}{}", name_part, ref_part)
            })
            .collect();
        
        sig.push_str(&params.join(", "));
        sig.push(')');
        if let Some(ret_type) = &self.return_type {
            sig.push_str(&format!(" -> {:?}", ret_type));
        }
        
        sig
    }
}