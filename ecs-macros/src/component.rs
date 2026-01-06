//! Component derive macro implementation

use darling::FromDeriveInput;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields};

use crate::utils::{generate_pack_field, generate_unpack_field, string_to_discriminator, type_size};

#[derive(FromDeriveInput)]
#[darling(attributes(component))]
struct ComponentArgs {
    seed: String,
    #[darling(default)]
    discriminator: Option<String>,
}

pub fn derive_component_impl(input: DeriveInput) -> syn::Result<TokenStream> {
    let args = ComponentArgs::from_derive_input(&input)?;

    let name = &input.ident;
    let seed = &args.seed;
    let discriminator_str = args.discriminator.as_deref().unwrap_or(&args.seed);
    let discriminator = string_to_discriminator(discriminator_str);

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => return Err(syn::Error::new_spanned(name, "Only named fields are supported")),
        },
        _ => return Err(syn::Error::new_spanned(name, "Only structs are supported")),
    };

    // Calculate size and generate pack/unpack code
    let mut offset = 8usize; // Start after discriminator
    let mut pack_fields = Vec::new();
    let mut unpack_fields = Vec::new();
    let mut field_names = Vec::new();

    for field in fields.iter() {
        let field_name = field.ident.as_ref().unwrap();
        field_names.push(field_name.clone());

        // Check for skip attribute
        let has_skip = field.attrs.iter().any(|attr| attr.path().is_ident("skip"));
        if has_skip {
            continue;
        }

        // Check for pda_bump attribute (always last, size 1)
        let is_bump = field.attrs.iter().any(|attr| attr.path().is_ident("pda_bump"));

        let size = type_size(&field.ty).unwrap_or(0);

        pack_fields.push(generate_pack_field(field, offset));
        unpack_fields.push(generate_unpack_field(field, offset));

        offset += size;

        if is_bump && size != 1 {
            return Err(syn::Error::new_spanned(
                field,
                "pda_bump field must be u8",
            ));
        }
    }

    let total_size = offset;
    let disc_bytes = discriminator;

    let expanded = quote! {
        impl golt_runtime::Component for #name {
            const DISCRIMINATOR: [u8; 8] = [
                #(#disc_bytes),*
            ];
            const SEED: &'static [u8] = #seed.as_bytes();
            const SIZE: usize = #total_size;

            fn unpack(data: &[u8]) -> Option<Self> {
                if data.len() < Self::SIZE {
                    return None;
                }
                if data[0..8] != Self::DISCRIMINATOR {
                    return None;
                }

                #(#unpack_fields)*

                Some(Self {
                    #(#field_names),*
                })
            }

            fn pack(&self, data: &mut [u8]) {
                data[0..8].copy_from_slice(&Self::DISCRIMINATOR);
                #(#pack_fields)*
            }
        }

        impl #name {
            /// Derive the PDA for this component given the entity
            pub fn derive_pda_with_entity(
                entity: &[u8; 32],
                program_id: &golt_runtime::prelude::Pubkey,
            ) -> (golt_runtime::prelude::Pubkey, u8) {
                golt_runtime::pda::derive_pda(
                    &[Self::SEED, entity.as_ref()],
                    program_id,
                )
            }
        }
    };

    Ok(expanded)
}
