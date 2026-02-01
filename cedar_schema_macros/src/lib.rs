use proc_macro::TokenStream;
use quote::quote;
use std::collections::BTreeSet;
use std::path::PathBuf;
use syn::{LitStr, parse_macro_input};

#[proc_macro]
pub fn cedar_schema_consts(input: TokenStream) -> TokenStream {
    let path_literal = parse_macro_input!(input as LitStr);
    match build_consts(&path_literal) {
        Ok(tokens) => tokens,
        Err(err) => err,
    }
}

fn build_consts(path_literal: &LitStr) -> Result<TokenStream, TokenStream> {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").map_err(|err| {
        compile_error(&format!(
            "cedar_schema_consts failed to read CARGO_MANIFEST_DIR: {err}"
        ))
    })?;
    let mut path = PathBuf::from(manifest_dir);
    path.push(path_literal.value());
    let src = std::fs::read_to_string(&path).map_err(|err| {
        compile_error(&format!(
            "cedar_schema_consts failed to read schema file {}: {err}",
            path.display()
        ))
    })?;

    let (entities, actions) = parse_schema(&src);
    if entities.is_empty() && actions.is_empty() {
        return Err(compile_error(
            "cedar_schema_consts found no entities/actions",
        ));
    }

    let entity_consts = entities.iter().map(|name| {
        let const_name = format!("ENTITY_TYPE_{}", to_upper_snake(name));
        let ident = syn::Ident::new(&const_name, path_literal.span());
        quote! { const #ident: &str = #name; }
    });

    let action_consts = actions.iter().map(|name| {
        let const_name = format!("ACTION_{}", to_upper_snake(name));
        let ident = syn::Ident::new(&const_name, path_literal.span());
        quote! { const #ident: &str = #name; }
    });

    Ok(quote! {
        #(#entity_consts)*
        #(#action_consts)*
    }
    .into())
}

fn parse_schema(src: &str) -> (BTreeSet<String>, BTreeSet<String>) {
    let mut entities = BTreeSet::new();
    let mut actions = BTreeSet::new();

    for line in src.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("//") {
            continue;
        }
        if let Some(name) = line.strip_prefix("entity ") {
            if let Some(name) = take_ident(name) {
                entities.insert(name.to_string());
            }
            continue;
        }
        if let Some(name) = line.strip_prefix("action ")
            && let Some(name) = take_ident(name)
        {
            actions.insert(name.to_string());
        }
    }

    (entities, actions)
}

fn take_ident(input: &str) -> Option<&str> {
    let mut end = 0;
    for (idx, ch) in input.char_indices() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            end = idx + ch.len_utf8();
        } else {
            break;
        }
    }
    if end == 0 { None } else { Some(&input[..end]) }
}

fn to_upper_snake(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut prev_is_lower_or_digit = false;

    for ch in input.chars() {
        if ch.is_ascii_uppercase() {
            if prev_is_lower_or_digit {
                out.push('_');
            }
            out.push(ch);
            prev_is_lower_or_digit = false;
        } else if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_uppercase());
            prev_is_lower_or_digit = true;
        } else {
            prev_is_lower_or_digit = false;
        }
    }

    out
}

fn compile_error(message: &str) -> TokenStream {
    quote! { compile_error!(#message); }.into()
}
