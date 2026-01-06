//! System derive macro implementation

use darling::FromDeriveInput;
use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

#[derive(FromDeriveInput)]
#[darling(attributes(system))]
struct SystemArgs {
    id: String,
}

pub fn derive_system_impl(input: DeriveInput) -> syn::Result<TokenStream> {
    let args = SystemArgs::from_derive_input(&input)?;

    let name = &input.ident;
    let program_id = &args.id;

    let expanded = quote! {
        impl #name {
            /// Program ID as a string
            pub const PROGRAM_ID_STR: &'static str = #program_id;
        }

        // Declare the program ID
        golt_runtime::pinocchio_pubkey::declare_id!(#program_id);
    };

    Ok(expanded)
}
