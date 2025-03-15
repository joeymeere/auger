use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum SymbolType {
    /// regular function
    Function,
    /// method on a struct/enum
    Method,
    /// static method on a struct/enum (like a constructor)
    StaticMethod,
    /// trait implementation function
    TraitImpl,
    /// generic helper function/specialization
    GenericHelper,
    /// operator overload
    Operator,
    /// field or property accessor
    Accessor,
    /// type definition or constructor
    TypeDef,
    /// unknown/unidentified symbol type
    Unknown
}

impl fmt::Display for SymbolType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone)]
pub struct DemangledSymbol {
    pub path: Vec<String>,
    pub name: String,
    pub implementing_trait: Option<TraitImplementation>,
    pub hash: Option<String>,
    pub symbol_type: SymbolType,
    pub original: String,
}

#[derive(Debug, Clone)]
pub struct TraitImplementation {
    pub for_type: Vec<String>,
    pub trait_path: Vec<String>,
}

impl fmt::Display for DemangledSymbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let path_str = self.path.join("::");
        
        match self.symbol_type {
            SymbolType::TraitImpl => {
                if let Some(trait_impl) = &self.implementing_trait {
                    write!(
                        f,
                        "<{} as {}>::{}",
                        trait_impl.for_type.join("::"),
                        trait_impl.trait_path.join("::"),
                        self.name
                    )?;
                } else {
                    write!(f, "{}::{}", path_str, self.name)?;
                }
            },
            SymbolType::Method => {
                if self.path.is_empty() {
                    write!(f, "{}", self.name)?;
                } else {
                    let type_name = &self.path[self.path.len() - 1];
                    let prefix = if self.path.len() > 1 {
                        self.path[..self.path.len() - 1].join("::")
                    } else {
                        String::new()
                    };
                    
                    if !prefix.is_empty() {
                        write!(f, "{}::{}::{}", prefix, type_name, self.name)?;
                    } else {
                        write!(f, "{}::{}", type_name, self.name)?;
                    }
                }
            },
            SymbolType::StaticMethod => {
                if self.path.is_empty() {
                    write!(f, "{}", self.name)?;
                } else {
                    let type_name = &self.path[self.path.len() - 1];
                    let prefix = if self.path.len() > 1 {
                        self.path[..self.path.len() - 1].join("::")
                    } else {
                        String::new()
                    };
                    
                    if !prefix.is_empty() {
                        write!(f, "{}::{}::{} [static]", prefix, type_name, self.name)?;
                    } else {
                        write!(f, "{}::{} [static]", type_name, self.name)?;
                    }
                }
            },
            SymbolType::GenericHelper => {
                if let Some(trait_impl) = &self.implementing_trait {
                    let prefix = if !self.path.is_empty() {
                        format!("{}::", self.path.join("::"))
                    } else {
                        String::new()
                    };
                    
                    write!(
                        f,
                        "{}{}::{}",
                        prefix,
                        trait_impl.for_type[0],
                        self.name
                    )?;
                } else {
                    write!(f, "{}::{}", path_str, self.name)?;
                }
            },
            _ => {
                if !self.path.is_empty() {
                    write!(f, "{}::{}", path_str, self.name)?;
                } else {
                    write!(f, "{}", self.name)?;
                }
            }
        }
        
        if let Some(hash) = &self.hash {
            write!(f, " [{}]", hash)?;
        }
        
        Ok(())
    }
}

pub fn demangle(mangled: &str) -> Result<DemangledSymbol, &'static str> {
    // check prefix
    if !mangled.starts_with("_ZN") {
        return Err("Not a valid mangled name: missing _ZN prefix");
    }
    
    let chars: Vec<char> = mangled[3..].chars().collect();
    if !chars.is_empty() && chars[0].is_digit(10) {
        if chars.len() > 1 && chars[1].is_digit(10) {
            return parse_trait_implementation(mangled);
        } else if mangled[3..].contains("$LT$") || mangled[3..].contains("impl") {
            return parse_trait_implementation(mangled);
        } else {
            return parse_regular_function(mangled);
        }
    }
    
    parse_regular_function(mangled)
}

fn parse_regular_function(mangled: &str) -> Result<DemangledSymbol, &'static str> {
    let mut parts = Vec::new();
    let mut i = 3; // Skip "_ZN"
    
    if mangled[i..].contains("$LT$impl") {
        match parse_trait_implementation(mangled) {
            Ok(symbol) => return Ok(symbol),
            Err(_) => {} 
        }
    }
    
    while i < mangled.len() {
        let length_end = mangled[i..].find(|c: char| !c.is_digit(10))
            .map(|pos| i + pos)
            .unwrap_or(mangled.len());
        
        if i == length_end {
            break;
        }
        
        let length: usize = mangled[i..length_end].parse()
            .map_err(|_| "Invalid length in mangled name")?;
        
        i = length_end;
        
        if i + length > mangled.len() {
            return Err("Component length exceeds remaining string");
        }
        
        let component = &mangled[i..i + length];
        
        let cleaned_component = clean_component(component);
        parts.push(cleaned_component);
        
        i += length;
    }
    
    if parts.is_empty() {
        return Err("No components found in mangled name");
    }
    
    let name = parts.pop().unwrap();
    let hash = if i < mangled.len() {
        extract_hash(&mangled[i..])
    } else {
        None
    };
    
    let symbol_type = determine_symbol_type(&parts, &name);
    
    Ok(DemangledSymbol {
        path: parts,
        name,
        implementing_trait: None,
        hash,
        symbol_type,
        original: mangled.to_string(),
    })
}

fn clean_component(component: &str) -> String {
    if !component.contains('$') {
        return component.to_string();
    }

    let mut result = component.to_string();

    let replacements = [
        ("$LT$", "<"),
        ("$GT$", ">"),
        ("$u20$", " "),
        ("$u21$", "!"),
    ];

    for (from, to) in &replacements {
        result = result.replace(from, to);
    }

    result
}

fn determine_symbol_type(path: &[String], name: &str) -> SymbolType {
    if path.is_empty() {
        return SymbolType::Function;
    }
    
    let last_component = &path[path.len() - 1];

    if last_component.contains("$LT$") || last_component.contains("<impl") || 
       last_component.contains("as") || name.contains("$LT$") {
        return SymbolType::TraitImpl;
    }
    
    if let Some(ch) = last_component.chars().next() {
        if ch.is_uppercase() {
            if name == "new" || name.starts_with("new_") || name.starts_with("create_") {
                return SymbolType::StaticMethod;
            }
            
            if name.starts_with("get_") || name.starts_with("set_") || 
               name.starts_with("is_") || name.starts_with("has_") {
                return SymbolType::Accessor;
            }
            
            return SymbolType::Method;
        }
    }
    
    if name.contains("add") || name.contains("sub") || name.contains("mul") || 
       name.contains("div") || name.contains("eq") || name.contains("cmp") ||
       name.contains("index") || name.contains("deref") {
        return SymbolType::Operator;
    }
    
    if name.contains("do_") || path.iter().any(|p| p.contains("helper")) ||
       path.iter().any(|p| p.contains("util")) {
        return SymbolType::GenericHelper;
    }
    
    if name.contains("type") || name == "drop" || name == "clone" || name == "default" {
        return SymbolType::TypeDef;
    }
    
    SymbolType::Function
}

fn parse_trait_implementation(mangled: &str) -> Result<DemangledSymbol, &'static str> {
    let mut i = 3; // Skip itanium prefix _ZN

    let length_end = mangled[i..].find(|c: char| !c.is_digit(10))
        .map(|pos| i + pos)
        .unwrap_or(mangled.len());
    
    let _full_length: usize = mangled[i..length_end].parse()
        .map_err(|_| "Invalid length in trait implementation")?;
    
    i = length_end;
    
    let is_prefixed = mangled[i..].starts_with("_$LT$");
    
    if is_prefixed {
        i += 5; // Skip less than _$LT$
    } else if mangled[i..].starts_with("$LT$") {
        i += 4; // Skip less than $LT$
    } else {
        return Err("Expected trait implementation marker $LT$ not found");
    }
    
    
    if let Some(as_pos) = mangled[i..].find("$u20$as$u20$") {
        let for_type_string = mangled[i..i + as_pos].to_string();
        let for_type = parse_type_path(&for_type_string);
        
        i += as_pos + 12;
        
        let trait_path_end = mangled[i..].find("$GT$")
            .ok_or("Missing trait implementation end marker $GT$")?;
        
        let trait_path_string = mangled[i..i + trait_path_end].to_string();
        let trait_path = parse_type_path(&trait_path_string);
        
        i += trait_path_end + 4; // kip $GT$
        
        let method_length_end = mangled[i..].find(|c: char| !c.is_digit(10))
            .map(|pos| i + pos)
            .ok_or("Missing method name length")?;
        
        let method_length: usize = mangled[i..method_length_end].parse()
            .map_err(|_| "Invalid method name length")?;
        
        i = method_length_end;
        
        let method_name = &mangled[i..i + method_length];
        i += method_length;
        
        let hash = extract_hash(&mangled[i..]);
        
        return Ok(DemangledSymbol {
            path: Vec::new(),
            name: clean_component(method_name),
            implementing_trait: Some(TraitImplementation {
                for_type,
                trait_path,
            }),
            hash,
            symbol_type: SymbolType::TraitImpl,
            original: mangled.to_string(),
        });
    }
    
    let gt_pos = mangled[i..].find("$GT$");
    
    if let Some(gt_pos) = gt_pos {
        let generic_part = mangled[i..i + gt_pos].to_string();
        i += gt_pos + 4; // Skip $GT$
        
        let mut parts = Vec::new();
        
        while i < mangled.len() {
            let length_end = mangled[i..].find(|c: char| !c.is_digit(10))
                .map(|pos| i + pos)
                .unwrap_or(mangled.len());
            
            if i == length_end {
                break;
            }
            
            let length: usize = mangled[i..length_end].parse()
                .map_err(|_| "Invalid length in mangled name")?;
            
            i = length_end;
            
            if i + length > mangled.len() {
                return Err("Component length exceeds remaining string");
            }
            
            let component = &mangled[i..i + length];
            parts.push(clean_component(component));
            
            i += length;
        }
        
        if parts.is_empty() {
            return Err("No components found after generic part");
        }
        
        let name = parts.pop().unwrap();
        
        let hash = if i < mangled.len() {
            extract_hash(&mangled[i..])
        } else {
            None
        };
        
        let generic_type = parse_specialized_generic(&generic_part);
        
        return Ok(DemangledSymbol {
            path: parts,
            name,
            implementing_trait: Some(TraitImplementation {
                for_type: vec![generic_type],
                trait_path: vec!["Specialized".to_string()],
            }),
            hash,
            symbol_type: SymbolType::GenericHelper,
            original: mangled.to_string(),
        });
    }
    
    Err("Unrecognized trait implementation pattern")
}

fn parse_type_path(type_str: &str) -> Vec<String> {
    let cleaned = type_str.replace("..", "::");
    
    let mut path = cleaned
        .split("::")
        .map(|s| clean_component(s))
        .collect::<Vec<_>>();
    
    path.retain(|s| !s.is_empty());
    
    path
}

fn parse_specialized_generic(generic_str: &str) -> String {
    let cleaned = clean_component(generic_str);
    
    let mut formatted = cleaned.replace("<", "< ");
    formatted = formatted.replace(">", " >");
    formatted = formatted.replace(",", ", ");
    
    formatted
}

/// typically "17h" followed by 16 hex digits and ending with "E" and a null terminator
/// \d+h[0-9a-f]+E
fn extract_hash(hash_part: &str) -> Option<String> {
    if hash_part.is_empty() || hash_part.len() < 4 {
        return None;
    }
    
    let mut i = 0;
    
    while i < hash_part.len() && !hash_part[i..].chars().next().unwrap().is_digit(10) {
        i += 1;
    }
    
    if i >= hash_part.len() {
        return None;
    }
    
    let length_start = i;
    while i < hash_part.len() && hash_part[i..].chars().next().unwrap().is_digit(10) {
        i += 1;
    }
    
    if i >= hash_part.len() || !hash_part[i..].starts_with('h') {
        return None;
    }
    
    let length_str = &hash_part[length_start..i];
    i += 1;
    
    if i >= hash_part.len() {
        return None;
    }
    
    let hash_start = i;
    while i < hash_part.len() && 
          hash_part[i..].chars().next().unwrap().is_digit(16) {
        i += 1;
    }
    
    if i >= hash_part.len() || !hash_part[i..].starts_with('E') {
        return None;
    }
    
    let hash_value = &hash_part[hash_start..i];
    
    if let Ok(expected_len) = length_str.parse::<usize>() {
        if hash_value.len() != expected_len {
            if hash_value.len() >= 8 && hash_value.chars().all(|c| c.is_digit(16)) {
                return Some(format!("h{}", hash_value));
            }
            return None;
        }
    }
    
    Some(format!("h{}", hash_value))
}

pub fn extract_mangled_names(blob: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut start_idx = 0;
    
    while let Some(pos) = blob[start_idx..].find("_ZN") {
        let name_start = start_idx + pos;
        
        if name_start > start_idx && start_idx > 0 {
            names.push(blob[start_idx..name_start].to_string());
        }
        
        start_idx = name_start;
        
        let next_pattern = blob[start_idx + 3..].find("_ZN").map(|p| start_idx + 3 + p);
        
        if let Some(end_pos) = blob[start_idx..].find('E') {
            let potential_end = start_idx + end_pos + 1;
            
            if next_pattern.is_none() || potential_end <= next_pattern.unwrap() {
                names.push(blob[start_idx..potential_end].to_string());
                start_idx = potential_end;
            }
        } else if let Some(next_start) = next_pattern {
            names.push(blob[start_idx..next_start].to_string());
            start_idx = next_start;
        } else {
            names.push(blob[start_idx..].to_string());
            break;
        }
    }
    
    if start_idx < blob.len() {
        names.push(blob[start_idx..].to_string());
    }
    
    names.into_iter()
        .filter(|name| name.starts_with("_ZN") && name.ends_with('E'))
        .collect()
}