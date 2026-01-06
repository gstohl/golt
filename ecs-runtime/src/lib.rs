//! Golt ECS Runtime
//!
//! Core runtime library for the Golt ECS framework.
//! Provides traits, types, and utilities for building Solana programs.

pub use pinocchio;
pub use pinocchio_pubkey;
pub use pinocchio_system;

pub mod account;
pub mod component;
pub mod error;
pub mod instruction;
pub mod pda;

pub use account::*;
pub use component::*;
pub use error::*;
pub use instruction::*;
pub use pda::*;

/// Re-export common pinocchio types
pub mod prelude {
    pub use pinocchio::{
        account_info::AccountInfo,
        entrypoint,
        instruction::{AccountMeta, Instruction, Seed, Signer},
        program::invoke_signed,
        program_error::ProgramError,
        pubkey::Pubkey,
        sysvars::{clock::Clock, rent::Rent, Sysvar},
        ProgramResult,
    };
    pub use pinocchio_system::instructions::CreateAccount;

    pub use crate::account::*;
    pub use crate::component::*;
    pub use crate::error::*;
    pub use crate::instruction::*;
    pub use crate::pda::*;
}
