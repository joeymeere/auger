use quote::{quote, format_ident};

use syn::{
    parse_macro_input, 
    LitStr, 
    Expr, 
    Token, 
    parse::{Parse, ParseStream}, 
    punctuated::Punctuated
};

pub struct PluginRegistration {
    component_type: syn::Ident,
    colon_token: Token![:],
    name: LitStr,
    arrow_token: Token![=>],
    constructor: Expr,
}

impl Parse for PluginRegistration {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(PluginRegistration {
            component_type: input.parse()?,
            colon_token: input.parse()?,
            name: input.parse()?,
            arrow_token: input.parse()?,
            constructor: input.parse()?,
        })
    }
}

struct RegisterPluginsInput {
    registrations: Punctuated<PluginRegistration, Token![,]>,
}

impl Parse for RegisterPluginsInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(RegisterPluginsInput {
            registrations: Punctuated::parse_terminated(input)?,
        })
    }
}


pub fn register_plugins(input: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    let RegisterPluginsInput { registrations } = parse_macro_input!(input as RegisterPluginsInput);
    
    let registration_statements = registrations.iter().map(|reg| {
        let component_type = &reg.component_type;
        let name = &reg.name;
        let constructor = &reg.constructor;
        
        match component_type.to_string().as_str() {
            "parser" => quote! {
                registry.register_parser(#name, #constructor);
            },
            "analyzer" => quote! {
                registry.register_analyzer(#name, #constructor);
            },
            "resolver" => quote! {
                registry.register_resolver(#name, #constructor);
            },
            _ => {
                let method_name = format_ident!("register_{}", component_type);
                quote! {
                    registry.#method_name(#name, #constructor);
                }
            },
        }
    });
    
    let expanded = quote! {
        {
            #(#registration_statements)*
        }
    };
    
    TokenStream::from(expanded)
}