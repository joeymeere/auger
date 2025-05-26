use quote::quote;
use proc_macro2::Span;
use proc_macro::TokenStream;

use syn::{
    parse_macro_input, 
    Meta, 
    MetaList, 
    NestedMeta, 
    Lit, 
    LitStr, 
    parse::Parse
};

#[proc_macro_attribute]
pub fn component_docs(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr_args = parse_macro_input!(attr as syn::AttributeArgs);
    
    let mut input_item = parse_macro_input!(item as syn::Item);
    
    let mut related_components = Vec::new();
    let mut examples = Vec::new();
    let mut safety_notes = None;
    
    for arg in attr_args {
        if let NestedMeta::Meta(Meta::NameValue(nv)) = arg {
            let path = nv.path.get_ident().map(|i| i.to_string()).unwrap_or_default();
            
            match path.as_str() {
                "related" => {
                    if let Lit::Str(lit_str) = &nv.lit {
                        let content = lit_str.value();
                        let content = content.trim_start_matches('[').trim_end_matches(']');
                        related_components = content
                            .split(',')
                            .map(|s| s.trim().trim_matches('"'))
                            .filter(|s| !s.is_empty())
                            .collect();
                    }
                },
                "examples" => {
                    if let Lit::Str(lit_str) = &nv.lit {
                        // Parse array of example files
                        let content = lit_str.value();
                        let content = content.trim_start_matches('[').trim_end_matches(']');
                        examples = content
                            .split(',')
                            .map(|s| s.trim().trim_matches('"'))
                            .filter(|s| !s.is_empty())
                            .collect();
                    }
                },
                "safety_notes" => {
                    if let Lit::Str(lit_str) = &nv.lit {
                        safety_notes = Some(lit_str.value());
                    }
                },
                _ => {}
            }
        }
    }
    
    match &mut input_item {
        syn::Item::Struct(item_struct) => {
            let mut doc = String::new();
            
            if !related_components.is_empty() {
                doc.push_str("# Related Components\n\n");
                for component in &related_components {
                    doc.push_str(&format!("- [`{0}`]\n", component));
                }
                doc.push_str("\n");
            }
            
            if !examples.is_empty() {
                doc.push_str("# Examples\n\n");
                for example in &examples {
                    doc.push_str(&format!("See [{0}]({0}) for usage examples.\n", example));
                }
                doc.push_str("\n");
            }
            
            if let Some(notes) = &safety_notes {
                doc.push_str("# Safety Notes\n\n");
                doc.push_str(notes);
                doc.push_str("\n\n");
            }
            
            let existing_doc = extract_doc_comments(&item_struct.attrs);
            if !existing_doc.is_empty() {
                doc = format!("{}\n\n{}", existing_doc, doc);
            }
            
            item_struct.attrs.retain(|attr| !is_doc_attr(attr));
            
            let doc_attr = syn::parse_quote!(#[doc = #doc]);
            item_struct.attrs.push(doc_attr);
        },
        _ => {}
    }
    
    TokenStream::from(quote! { #input_item })
}

fn is_doc_attr(attr: &syn::Attribute) -> bool {
    attr.path.is_ident("doc")
}

fn extract_doc_comments(attrs: &[syn::Attribute]) -> String {
    let mut docs = String::new();
    
    for attr in attrs {
        if is_doc_attr(attr) {
            if let Ok(Meta::NameValue(meta)) = attr.parse_meta() {
                if let Lit::Str(lit) = meta.lit {
                    if !docs.is_empty() {
                        docs.push('\n');
                    }
                    docs.push_str(&lit.value());
                }
            }
        }
    }
    
    docs
}