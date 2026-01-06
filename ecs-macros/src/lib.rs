//! Golt ECS Proc Macros
//!
//! Provides `#[component]` and `#[system]` macros for generating
//! boilerplate code for Solana ECS programs.

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput, ItemImpl};

mod component;
mod instruction;
mod system;
mod utils;

/// Derive macro for ECS components
///
/// # Example
///
/// ```rust
/// use golt_macros::Component;
///
/// #[derive(Component)]
/// #[component(seed = "health", discriminator = "health\0\0")]
/// pub struct Health {
///     pub entity: [u8; 32],
///     pub current: u32,
///     pub max: u32,
///     #[pda_bump]
///     pub bump: u8,
/// }
/// ```
#[proc_macro_derive(Component, attributes(component, pda_bump, skip))]
pub fn derive_component(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    component::derive_component_impl(input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

/// Attribute macro for component instruction implementations
///
/// # Example
///
/// ```rust
/// #[component_instructions]
/// impl Health {
///     #[instruction(tag = 0)]
///     pub fn init(entity: [u8; 32], max: u32) -> Self {
///         Self { entity, current: max, max, bump: 0 }
///     }
///
///     #[instruction(tag = 1)]
///     pub fn damage(&mut self, amount: u32) {
///         self.current = self.current.saturating_sub(amount);
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn component_instructions(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemImpl);
    instruction::generate_instructions_impl(input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

/// Derive macro for ECS systems
///
/// # Example
///
/// ```rust
/// use golt_macros::System;
///
/// #[derive(System)]
/// #[system(id = "CombatProgram111111111111111111111111111111")]
/// pub struct Combat;
/// ```
#[proc_macro_derive(System, attributes(system))]
pub fn derive_system(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    system::derive_system_impl(input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

/// Attribute macro for system instruction implementations
#[proc_macro_attribute]
pub fn system_instructions(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemImpl);
    instruction::generate_system_instructions_impl(input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}
