//! Instruction generation for components and systems

use proc_macro2::TokenStream;
use quote::quote;
use syn::{ImplItem, ItemImpl};

pub fn generate_instructions_impl(input: ItemImpl) -> syn::Result<TokenStream> {
    let struct_name = &input.self_ty;
    let mut instruction_variants = Vec::new();
    let mut unpack_arms = Vec::new();
    let mut pack_arms = Vec::new();
    let mut process_arms = Vec::new();

    for item in &input.items {
        if let ImplItem::Fn(method) = item {
            // Look for #[instruction(tag = N)] attribute
            let instruction_attr = method.attrs.iter().find(|attr| attr.path().is_ident("instruction"));

            if let Some(attr) = instruction_attr {
                let tag: u8 = parse_instruction_tag(attr)?;
                let method_name = &method.sig.ident;
                let variant_name = heck::AsUpperCamelCase(method_name.to_string()).to_string();
                let variant_ident = syn::Ident::new(&variant_name, method_name.span());

                // Extract parameters (skip self if present)
                let params: Vec<_> = method
                    .sig
                    .inputs
                    .iter()
                    .filter_map(|arg| {
                        if let syn::FnArg::Typed(pat_type) = arg {
                            if let syn::Pat::Ident(ident) = &*pat_type.pat {
                                return Some((ident.ident.clone(), pat_type.ty.clone()));
                            }
                        }
                        None
                    })
                    .collect();

                let param_names: Vec<_> = params.iter().map(|(name, _)| name.clone()).collect();
                let param_types: Vec<_> = params.iter().map(|(_, ty)| ty.clone()).collect();

                // Generate variant
                if params.is_empty() {
                    instruction_variants.push(quote! { #variant_ident });
                } else {
                    instruction_variants.push(quote! {
                        #variant_ident { #(#param_names: #param_types),* }
                    });
                }

                // Generate unpack arm
                let unpack_code = generate_unpack_code(&params);
                if params.is_empty() {
                    unpack_arms.push(quote! {
                        #tag => Ok(Self::#variant_ident),
                    });
                } else {
                    unpack_arms.push(quote! {
                        #tag => {
                            #unpack_code
                            Ok(Self::#variant_ident { #(#param_names),* })
                        }
                    });
                }

                // Generate pack arm
                let pack_code = generate_pack_code(&params);
                let size = calculate_params_size(&params);
                if params.is_empty() {
                    pack_arms.push(quote! {
                        Self::#variant_ident => vec![#tag],
                    });
                } else {
                    pack_arms.push(quote! {
                        Self::#variant_ident { #(#param_names),* } => {
                            let mut data = vec![0u8; 1 + #size];
                            data[0] = #tag;
                            #pack_code
                            data
                        }
                    });
                }

                // Generate process arm (placeholder - actual implementation in processor)
                process_arms.push(quote! {
                    Self::#variant_ident { #(#param_names),* } => {
                        // Process #method_name
                    }
                });
            }
        }
    }

    let instruction_enum_name = syn::Ident::new(
        &format!("{}Instruction", quote!(#struct_name).to_string().replace(" ", "")),
        proc_macro2::Span::call_site(),
    );

    let expanded = quote! {
        #input

        #[derive(Clone, Debug)]
        pub enum #instruction_enum_name {
            #(#instruction_variants),*
        }

        impl #instruction_enum_name {
            pub fn unpack(data: &[u8]) -> Result<Self, golt_runtime::prelude::ProgramError> {
                let (&tag, rest) = data
                    .split_first()
                    .ok_or(golt_runtime::prelude::ProgramError::InvalidInstructionData)?;

                match tag {
                    #(#unpack_arms)*
                    _ => Err(golt_runtime::prelude::ProgramError::InvalidInstructionData),
                }
            }

            pub fn pack(&self) -> Vec<u8> {
                match self {
                    #(#pack_arms)*
                }
            }
        }
    };

    Ok(expanded)
}

pub fn generate_system_instructions_impl(input: ItemImpl) -> syn::Result<TokenStream> {
    // Similar to component instructions but for systems
    generate_instructions_impl(input)
}

fn parse_instruction_tag(attr: &syn::Attribute) -> syn::Result<u8> {
    let meta = attr.parse_args::<syn::Meta>()?;
    if let syn::Meta::NameValue(nv) = meta {
        if nv.path.is_ident("tag") {
            if let syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Int(lit_int),
                ..
            }) = nv.value
            {
                return lit_int.base10_parse();
            }
        }
    }
    Err(syn::Error::new_spanned(attr, "Expected #[instruction(tag = N)]"))
}

fn generate_unpack_code(params: &[(syn::Ident, Box<syn::Type>)]) -> TokenStream {
    let mut offset = 0usize;
    let mut code = Vec::new();

    for (name, ty) in params {
        let size = estimate_type_size(ty);
        let unpack = generate_param_unpack(name, ty, offset);
        code.push(unpack);
        offset += size;
    }

    quote! { #(#code)* }
}

fn generate_pack_code(params: &[(syn::Ident, Box<syn::Type>)]) -> TokenStream {
    let mut offset = 1usize; // Skip tag byte
    let mut code = Vec::new();

    for (name, ty) in params {
        let size = estimate_type_size(ty);
        let pack = generate_param_pack(name, ty, offset);
        code.push(pack);
        offset += size;
    }

    quote! { #(#code)* }
}

fn calculate_params_size(params: &[(syn::Ident, Box<syn::Type>)]) -> usize {
    params.iter().map(|(_, ty)| estimate_type_size(ty)).sum()
}

fn estimate_type_size(ty: &syn::Type) -> usize {
    if let syn::Type::Path(type_path) = ty {
        let ident = type_path.path.segments.last().map(|s| s.ident.to_string());
        match ident.as_deref() {
            Some("u8") | Some("i8") | Some("bool") => 1,
            Some("u16") | Some("i16") => 2,
            Some("u32") | Some("i32") | Some("f32") => 4,
            Some("u64") | Some("i64") | Some("f64") => 8,
            Some("u128") | Some("i128") => 16,
            Some("Pubkey") => 32,
            _ => 0,
        }
    } else if let syn::Type::Array(arr) = ty {
        if let syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Int(lit_int),
            ..
        }) = &arr.len
        {
            let len: usize = lit_int.base10_parse().unwrap_or(0);
            let elem_size = estimate_type_size(&arr.elem);
            len * elem_size
        } else {
            0
        }
    } else {
        0
    }
}

fn generate_param_unpack(name: &syn::Ident, ty: &syn::Type, offset: usize) -> TokenStream {
    if let syn::Type::Path(type_path) = ty {
        let ident = type_path.path.segments.last().map(|s| s.ident.to_string());
        match ident.as_deref() {
            Some("u8") => quote! { let #name = rest[#offset]; },
            Some("i8") => quote! { let #name = rest[#offset] as i8; },
            Some("bool") => quote! { let #name = rest[#offset] != 0; },
            Some("u16") => quote! {
                let #name = u16::from_le_bytes(rest[#offset..#offset + 2].try_into()
                    .map_err(|_| golt_runtime::prelude::ProgramError::InvalidInstructionData)?);
            },
            Some("i16") => quote! {
                let #name = i16::from_le_bytes(rest[#offset..#offset + 2].try_into()
                    .map_err(|_| golt_runtime::prelude::ProgramError::InvalidInstructionData)?);
            },
            Some("u32") => quote! {
                let #name = u32::from_le_bytes(rest[#offset..#offset + 4].try_into()
                    .map_err(|_| golt_runtime::prelude::ProgramError::InvalidInstructionData)?);
            },
            Some("i32") => quote! {
                let #name = i32::from_le_bytes(rest[#offset..#offset + 4].try_into()
                    .map_err(|_| golt_runtime::prelude::ProgramError::InvalidInstructionData)?);
            },
            Some("u64") => quote! {
                let #name = u64::from_le_bytes(rest[#offset..#offset + 8].try_into()
                    .map_err(|_| golt_runtime::prelude::ProgramError::InvalidInstructionData)?);
            },
            Some("i64") => quote! {
                let #name = i64::from_le_bytes(rest[#offset..#offset + 8].try_into()
                    .map_err(|_| golt_runtime::prelude::ProgramError::InvalidInstructionData)?);
            },
            _ => quote! { let #name = Default::default(); },
        }
    } else if let syn::Type::Array(arr) = ty {
        let size = estimate_type_size(ty);
        quote! {
            let #name: #ty = rest[#offset..#offset + #size].try_into()
                .map_err(|_| golt_runtime::prelude::ProgramError::InvalidInstructionData)?;
        }
    } else {
        quote! { let #name = Default::default(); }
    }
}

fn generate_param_pack(name: &syn::Ident, ty: &syn::Type, offset: usize) -> TokenStream {
    if let syn::Type::Path(type_path) = ty {
        let ident = type_path.path.segments.last().map(|s| s.ident.to_string());
        match ident.as_deref() {
            Some("u8") | Some("i8") => quote! { data[#offset] = #name as u8; },
            Some("bool") => quote! { data[#offset] = if #name { 1 } else { 0 }; },
            Some("u16") | Some("i16") => quote! {
                data[#offset..#offset + 2].copy_from_slice(&#name.to_le_bytes());
            },
            Some("u32") | Some("i32") | Some("f32") => quote! {
                data[#offset..#offset + 4].copy_from_slice(&#name.to_le_bytes());
            },
            Some("u64") | Some("i64") | Some("f64") => quote! {
                data[#offset..#offset + 8].copy_from_slice(&#name.to_le_bytes());
            },
            _ => quote! {},
        }
    } else if let syn::Type::Array(_) = ty {
        let size = estimate_type_size(ty);
        quote! {
            data[#offset..#offset + #size].copy_from_slice(&#name);
        }
    } else {
        quote! {}
    }
}
