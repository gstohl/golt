//! Rust source file parser for extracting component/system definitions

#![allow(dead_code)]

use anyhow::{Context, Result};
use std::path::Path;
use syn::{Attribute, Field, Fields, Item, Type};

/// Parsed component information
#[derive(Debug, Clone)]
pub struct ParsedComponent {
    pub name: String,
    pub fields: Vec<ParsedField>,
    pub seed: Option<String>,
}

/// Parsed field information
#[derive(Debug, Clone)]
pub struct ParsedField {
    pub name: String,
    pub rust_type: String,
    pub ts_type: String,
    pub size: usize,
    pub is_discriminator: bool,
    pub is_bump: bool,
}

/// Parsed instruction information
#[derive(Debug, Clone)]
pub struct ParsedInstruction {
    pub name: String,
    pub tag: u8,
    pub params: Vec<ParsedParam>,
    pub accounts: Vec<ParsedAccount>,
}

#[derive(Debug, Clone)]
pub struct ParsedParam {
    pub name: String,
    pub rust_type: String,
    pub ts_type: String,
}

#[derive(Debug, Clone)]
pub struct ParsedAccount {
    pub name: String,
    pub is_signer: bool,
    pub is_writable: bool,
    pub description: String,
}

/// Parse a component's state.rs file
pub fn parse_component_state(path: &Path) -> Result<ParsedComponent> {
    let content = std::fs::read_to_string(path)
        .context(format!("Failed to read {}", path.display()))?;

    let file = syn::parse_file(&content)
        .context(format!("Failed to parse {}", path.display()))?;

    // Find the main struct (usually the one with repr(C))
    for item in &file.items {
        if let Item::Struct(s) = item {
            let has_repr_c = s.attrs.iter().any(|a| is_repr_c(a));
            if has_repr_c {
                let seed = extract_seed_from_attrs(&s.attrs);
                let fields = parse_struct_fields(&s.fields)?;
                return Ok(ParsedComponent {
                    name: s.ident.to_string(),
                    fields,
                    seed,
                });
            }
        }
    }

    anyhow::bail!("No #[repr(C)] struct found in {}", path.display())
}

/// Parse a component's instruction.rs file
pub fn parse_component_instructions(path: &Path) -> Result<Vec<ParsedInstruction>> {
    let content = std::fs::read_to_string(path)
        .context(format!("Failed to read {}", path.display()))?;

    let file = syn::parse_file(&content)
        .context(format!("Failed to parse {}", path.display()))?;

    let mut instructions = Vec::new();

    // Find the instruction enum
    for item in &file.items {
        if let Item::Enum(e) = item {
            if e.ident.to_string().ends_with("Instruction") {
                for (tag, variant) in e.variants.iter().enumerate() {
                    let params = parse_variant_fields(&variant.fields);
                    let accounts = extract_accounts_from_docs(&variant.attrs);

                    instructions.push(ParsedInstruction {
                        name: variant.ident.to_string(),
                        tag: tag as u8,
                        params,
                        accounts,
                    });
                }
            }
        }
    }

    Ok(instructions)
}

fn is_repr_c(attr: &Attribute) -> bool {
    if attr.path().is_ident("repr") {
        if let Ok(meta) = attr.parse_args::<syn::Ident>() {
            return meta == "C";
        }
    }
    false
}

fn extract_seed_from_attrs(attrs: &[Attribute]) -> Option<String> {
    for attr in attrs {
        if attr.path().is_ident("component") {
            // Try to parse #[component(seed = "...")]
            if let Ok(meta) = attr.meta.require_list() {
                let tokens = meta.tokens.to_string();
                if let Some(seed_start) = tokens.find("seed") {
                    let rest = &tokens[seed_start..];
                    if let Some(quote_start) = rest.find('"') {
                        let after_quote = &rest[quote_start + 1..];
                        if let Some(quote_end) = after_quote.find('"') {
                            return Some(after_quote[..quote_end].to_string());
                        }
                    }
                }
            }
        }
    }
    None
}

fn parse_struct_fields(fields: &Fields) -> Result<Vec<ParsedField>> {
    let mut parsed = Vec::new();

    if let Fields::Named(named) = fields {
        for field in &named.named {
            let name = field.ident.as_ref().unwrap().to_string();
            let rust_type = type_to_string(&field.ty);
            let ts_type = rust_type_to_ts(&rust_type);
            let size = estimate_type_size(&rust_type);
            let is_discriminator = name == "discriminator";
            let is_bump = name == "bump" || has_bump_attr(field);

            parsed.push(ParsedField {
                name,
                rust_type,
                ts_type,
                size,
                is_discriminator,
                is_bump,
            });
        }
    }

    Ok(parsed)
}

fn has_bump_attr(field: &Field) -> bool {
    field.attrs.iter().any(|a| a.path().is_ident("pda_bump"))
}

fn parse_variant_fields(fields: &Fields) -> Vec<ParsedParam> {
    let mut params = Vec::new();

    match fields {
        Fields::Named(named) => {
            for field in &named.named {
                let name = field.ident.as_ref().unwrap().to_string();
                let rust_type = type_to_string(&field.ty);
                let ts_type = rust_type_to_ts(&rust_type);
                params.push(ParsedParam {
                    name,
                    rust_type,
                    ts_type,
                });
            }
        }
        Fields::Unnamed(unnamed) => {
            for (i, field) in unnamed.unnamed.iter().enumerate() {
                let rust_type = type_to_string(&field.ty);
                let ts_type = rust_type_to_ts(&rust_type);
                params.push(ParsedParam {
                    name: format!("arg{}", i),
                    rust_type,
                    ts_type,
                });
            }
        }
        Fields::Unit => {}
    }

    params
}

fn extract_accounts_from_docs(attrs: &[Attribute]) -> Vec<ParsedAccount> {
    let mut accounts = Vec::new();

    for attr in attrs {
        if attr.path().is_ident("doc") {
            if let Ok(meta) = attr.meta.require_name_value() {
                if let syn::Expr::Lit(lit) = &meta.value {
                    if let syn::Lit::Str(s) = &lit.lit {
                        let doc = s.value();
                        // Parse account docs like "0. `[signer, writable]` Payer"
                        if let Some(account) = parse_account_doc(&doc) {
                            accounts.push(account);
                        }
                    }
                }
            }
        }
    }

    accounts
}

fn parse_account_doc(doc: &str) -> Option<ParsedAccount> {
    let doc = doc.trim();

    // Match pattern: "N. `[flags]` Description"
    if !doc.chars().next()?.is_ascii_digit() {
        return None;
    }

    let dot_pos = doc.find('.')?;
    let rest = doc[dot_pos + 1..].trim();

    let (flags, description) = if rest.starts_with('`') {
        let end_tick = rest[1..].find('`')?;
        let flags_str = &rest[1..end_tick + 1];
        let desc = rest[end_tick + 2..].trim();
        (flags_str, desc.to_string())
    } else {
        ("", rest.to_string())
    };

    let is_signer = flags.contains("signer");
    let is_writable = flags.contains("writable");

    // Extract name from description (first word)
    let name = description
        .split_whitespace()
        .next()
        .unwrap_or("account")
        .to_lowercase();

    Some(ParsedAccount {
        name,
        is_signer,
        is_writable,
        description,
    })
}

fn type_to_string(ty: &Type) -> String {
    quote::quote!(#ty).to_string().replace(' ', "")
}

fn rust_type_to_ts(rust_type: &str) -> String {
    match rust_type {
        "u8" | "i8" | "u16" | "i16" | "u32" | "i32" | "f32" | "f64" => "number".to_string(),
        "u64" | "i64" | "u128" | "i128" => "bigint".to_string(),
        "bool" => "boolean".to_string(),
        "Pubkey" | "[u8;32]" => "PublicKey".to_string(),
        s if s.starts_with("[u8;") => {
            let size = s
                .trim_start_matches("[u8;")
                .trim_end_matches(']')
                .parse::<usize>()
                .unwrap_or(0);
            format!("Uint8Array /* {} bytes */", size)
        }
        _ => "unknown".to_string(),
    }
}

fn estimate_type_size(rust_type: &str) -> usize {
    match rust_type {
        "u8" | "i8" | "bool" => 1,
        "u16" | "i16" => 2,
        "u32" | "i32" | "f32" => 4,
        "u64" | "i64" | "f64" => 8,
        "u128" | "i128" => 16,
        "Pubkey" | "[u8;32]" => 32,
        s if s.starts_with("[u8;") => s
            .trim_start_matches("[u8;")
            .trim_end_matches(']')
            .parse::<usize>()
            .unwrap_or(0),
        _ => 0,
    }
}
