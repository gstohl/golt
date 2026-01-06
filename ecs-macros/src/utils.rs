//! Utility functions for code generation

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Field, Type};

/// Get the size of a type in bytes
pub fn type_size(ty: &Type) -> Option<usize> {
    match ty {
        Type::Path(type_path) => {
            let ident = type_path.path.segments.last()?.ident.to_string();
            match ident.as_str() {
                "u8" | "i8" | "bool" => Some(1),
                "u16" | "i16" => Some(2),
                "u32" | "i32" | "f32" => Some(4),
                "u64" | "i64" | "f64" => Some(8),
                "u128" | "i128" => Some(16),
                "Pubkey" => Some(32),
                _ => None,
            }
        }
        Type::Array(arr) => {
            if let syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Int(lit_int),
                ..
            }) = &arr.len
            {
                let len: usize = lit_int.base10_parse().ok()?;
                let elem_size = type_size(&arr.elem)?;
                Some(len * elem_size)
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Generate pack code for a field
pub fn generate_pack_field(field: &Field, offset: usize) -> TokenStream {
    let name = field.ident.as_ref().unwrap();
    let ty = &field.ty;

    match ty {
        Type::Path(type_path) => {
            let ident = type_path.path.segments.last().unwrap().ident.to_string();
            match ident.as_str() {
                "u8" | "i8" => quote! {
                    data[#offset] = self.#name as u8;
                },
                "bool" => quote! {
                    data[#offset] = if self.#name { 1 } else { 0 };
                },
                "u16" | "i16" => quote! {
                    data[#offset..#offset + 2].copy_from_slice(&self.#name.to_le_bytes());
                },
                "u32" | "i32" | "f32" => quote! {
                    data[#offset..#offset + 4].copy_from_slice(&self.#name.to_le_bytes());
                },
                "u64" | "i64" | "f64" => quote! {
                    data[#offset..#offset + 8].copy_from_slice(&self.#name.to_le_bytes());
                },
                "u128" | "i128" => quote! {
                    data[#offset..#offset + 16].copy_from_slice(&self.#name.to_le_bytes());
                },
                "Pubkey" => quote! {
                    data[#offset..#offset + 32].copy_from_slice(&self.#name);
                },
                _ => quote! {
                    // Unknown type, try to copy as bytes
                    data[#offset..#offset + core::mem::size_of_val(&self.#name)]
                        .copy_from_slice(&self.#name);
                },
            }
        }
        Type::Array(arr) => {
            if let Some(size) = type_size(ty) {
                quote! {
                    data[#offset..#offset + #size].copy_from_slice(&self.#name);
                }
            } else {
                quote! {
                    // Array copy
                    let arr_size = core::mem::size_of_val(&self.#name);
                    data[#offset..#offset + arr_size].copy_from_slice(
                        unsafe { core::slice::from_raw_parts(
                            &self.#name as *const _ as *const u8,
                            arr_size
                        )}
                    );
                }
            }
        }
        _ => quote! {},
    }
}

/// Generate unpack code for a field
pub fn generate_unpack_field(field: &Field, offset: usize) -> TokenStream {
    let name = field.ident.as_ref().unwrap();
    let ty = &field.ty;

    match ty {
        Type::Path(type_path) => {
            let ident = type_path.path.segments.last().unwrap().ident.to_string();
            match ident.as_str() {
                "u8" => quote! {
                    let #name = data[#offset];
                },
                "i8" => quote! {
                    let #name = data[#offset] as i8;
                },
                "bool" => quote! {
                    let #name = data[#offset] != 0;
                },
                "u16" => quote! {
                    let #name = u16::from_le_bytes(data[#offset..#offset + 2].try_into().ok()?);
                },
                "i16" => quote! {
                    let #name = i16::from_le_bytes(data[#offset..#offset + 2].try_into().ok()?);
                },
                "u32" => quote! {
                    let #name = u32::from_le_bytes(data[#offset..#offset + 4].try_into().ok()?);
                },
                "i32" => quote! {
                    let #name = i32::from_le_bytes(data[#offset..#offset + 4].try_into().ok()?);
                },
                "f32" => quote! {
                    let #name = f32::from_le_bytes(data[#offset..#offset + 4].try_into().ok()?);
                },
                "u64" => quote! {
                    let #name = u64::from_le_bytes(data[#offset..#offset + 8].try_into().ok()?);
                },
                "i64" => quote! {
                    let #name = i64::from_le_bytes(data[#offset..#offset + 8].try_into().ok()?);
                },
                "f64" => quote! {
                    let #name = f64::from_le_bytes(data[#offset..#offset + 8].try_into().ok()?);
                },
                "u128" => quote! {
                    let #name = u128::from_le_bytes(data[#offset..#offset + 16].try_into().ok()?);
                },
                "i128" => quote! {
                    let #name = i128::from_le_bytes(data[#offset..#offset + 16].try_into().ok()?);
                },
                "Pubkey" => quote! {
                    let #name: [u8; 32] = data[#offset..#offset + 32].try_into().ok()?;
                },
                _ => quote! {
                    let #name = Default::default(); // Unknown type
                },
            }
        }
        Type::Array(arr) => {
            if let Some(size) = type_size(ty) {
                quote! {
                    let #name: #ty = data[#offset..#offset + #size].try_into().ok()?;
                }
            } else {
                quote! {
                    let #name: #ty = Default::default(); // Unknown array
                }
            }
        }
        _ => quote! {
            let #name = Default::default();
        },
    }
}

/// Convert a string to a discriminator (8 bytes, padded with zeros)
pub fn string_to_discriminator(s: &str) -> [u8; 8] {
    let mut disc = [0u8; 8];
    let bytes = s.as_bytes();
    let len = bytes.len().min(8);
    disc[..len].copy_from_slice(&bytes[..len]);
    disc
}
