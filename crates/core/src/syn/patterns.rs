use proc_macro2::TokenStream;
use quote::quote;

use syn::{
    braced, 
    bracketed, 
    parenthesized, 
    parse::{Parse, ParseStream}, 
    parse_macro_input, 
    punctuated::Punctuated, 
    Expr, 
    Ident, 
    LitStr, 
    Token
};

enum BytePattern {
    Concrete(u8),
    Wildcard,
}

impl Parse for BytePattern {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(syn::LitInt) {
            let lit: syn::LitInt = input.parse()?;
            let value = lit.base10_parse::<u16>()?;
            if value > 255 {
                return Err(syn::Error::new(lit.span(), "Byte value cannot exceed 255"));
            }
            Ok(BytePattern::Concrete(value as u8))
        } else if input.peek(Token![?]) {
            let _: Token![?] = input.parse()?;
            let _: Token![?] = input.parse()?;
            Ok(BytePattern::Wildcard)
        } else {
            Err(input.error("Expected a byte value (0-255) or wildcard (??)"))
        }
    }
}

// Parse the signature pattern (array of bytes)
struct SignaturePattern {
    bytes: Vec<BytePattern>,
}

impl Parse for SignaturePattern {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        bracketed!(content in input);
        
        let mut bytes = Vec::new();
        while !content.is_empty() {
            bytes.push(content.parse()?);
            if !content.is_empty() {
                let _: Token![,] = content.parse()?;
            }
        }
        
        Ok(SignaturePattern { bytes })
    }
}

// Parse a capture definition
struct CaptureDefinition {
    name: Ident,
    colon_token: Token![:],
    capture_type: Ident,
    args: Punctuated<Expr, Token![,]>,
}

impl Parse for CaptureDefinition {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name: Ident = input.parse()?;
        let colon_token: Token![:] = input.parse()?;
        let capture_type: Ident = input.parse()?;
        
        let content;
        parenthesized!(content in input);
        let args = Punctuated::parse_terminated(&content)?;
        
        Ok(CaptureDefinition {
            name,
            colon_token,
            capture_type,
            args,
        })
    }
}

// Parse the captures block
struct CapturesBlock {
    captures: Vec<CaptureDefinition>,
}

impl Parse for CapturesBlock {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        braced!(content in input);
        
        let mut captures = Vec::new();
        while !content.is_empty() {
            captures.push(content.parse()?);
            if !content.is_empty() {
                let _: Token![,] = content.parse()?;
            }
        }
        
        Ok(CapturesBlock { captures })
    }
}

// Parse the entire pattern definition
struct PatternDefinition {
    name: LitStr,
    signature: SignaturePattern,
    captures: Option<CapturesBlock>,
}

impl Parse for PatternDefinition {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Parse name: "pattern_name",
        input.parse::<Ident>()?; // Parse "name"
        input.parse::<Token![:]>()?;
        let name: LitStr = input.parse()?;
        input.parse::<Token![,]>()?;
        
        // Parse signature: [...],
        input.parse::<Ident>()?; // Parse "signature"
        input.parse::<Token![:]>()?;
        let signature: SignaturePattern = input.parse()?;
        input.parse::<Token![,]>()?;
        
        // Optionally parse capture: {...}
        let captures = if input.peek(Ident) && input.peek2(Token![:]) && input.peek3(syn::token::Brace) {
            input.parse::<Ident>()?; // Parse "capture"
            input.parse::<Token![:]>()?;
            let captures: CapturesBlock = input.parse()?;
            Some(captures)
        } else {
            None
        };
        
        Ok(PatternDefinition {
            name,
            signature,
            captures,
        })
    }
}

pub fn define_pattern(input: TokenStream) -> TokenStream {
    let pattern_def = parse_macro_input!(input as PatternDefinition);
    
    // Convert the signature to a bytecode pattern
    let pattern_bytes = pattern_def.signature.bytes.iter().map(|byte| {
        match byte {
            BytePattern::Concrete(value) => quote! { 
                PatternByte::Exact(#value) 
            },
            BytePattern::Wildcard => quote! { 
                PatternByte::Any 
            },
        }
    });
    
    // Generate capture definitions
    let captures = if let Some(captures_block) = pattern_def.captures {
        let capture_defs = captures_block.captures.iter().map(|capture| {
            let name = &capture.name;
            let capture_type = &capture.capture_type;
            let args = &capture.args;
            
            quote! {
                pattern_builder.add_capture(
                    #name, 
                    CaptureType::#capture_type(#(#args),*)
                );
            }
        });
        
        quote! {
            #(#capture_defs)*
        }
    } else {
        quote! {}
    };
    
    let pattern_name = &pattern_def.name;
    
    let expanded = quote! {
        {
            let mut pattern_builder = PatternBuilder::new(#pattern_name);
            pattern_builder.set_bytes(&[
                #(#pattern_bytes),*
            ]);
            #captures
            pattern_builder.build()
        }
    };
    
    TokenStream::from(expanded)
}