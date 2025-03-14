pub fn to_snake_case(s: &str) -> String {
    let mut snake_case = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i != 0 {
                snake_case.push('_');
            }
            snake_case.push(c.to_ascii_lowercase());
        } else {
            snake_case.push(c);
        }
    }
    snake_case
}

pub fn snake_to_pascal(name: &str) -> String {
    let mut pascal = String::new();
    for (i, c) in name.split('_').enumerate() {
        if i > 0 {
            pascal.push_str(&c.to_ascii_uppercase());
        } else {
            pascal.push_str(&c.to_ascii_lowercase());
        }
    }
    pascal
}

pub fn capitalize_first_letter(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}


