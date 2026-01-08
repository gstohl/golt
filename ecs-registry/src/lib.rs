//! Golt Entity Registry Program
//!
//! A Solana program for managing entities in the Golt ECS framework.
//!
//! ## Overview
//!
//! The Entity Registry provides a centralized way to create and manage entities.
//! Each entity is stored in a PDA derived from its unique ID.
//!
//! ## Instructions
//!
//! - `Create`: Create a new entity with a unique ID
//! - `Transfer`: Transfer entity ownership to another account
//! - `Deactivate`: Mark an entity as inactive
//!
//! ## PDA Derivation
//!
//! Entity PDAs are derived as: `["entity", entity_id (u64 le bytes)]`

#[cfg(not(feature = "no-entrypoint"))]
mod entrypoint;

pub mod error;
pub mod instruction;
pub mod processor;
pub mod state;

pub use error::RegistryError;
pub use state::{Entity, ENTITY_DISCRIMINATOR, ENTITY_SEED};

// Re-export for convenience
pub use pinocchio::pubkey::Pubkey;

/// Derive entity PDA from entity ID
pub fn derive_entity_pda(entity_id: u64, program_id: &Pubkey) -> (Pubkey, u8) {
    let entity_id_bytes = entity_id.to_le_bytes();
    pinocchio::pubkey::find_program_address(&[ENTITY_SEED, &entity_id_bytes], program_id)
}
