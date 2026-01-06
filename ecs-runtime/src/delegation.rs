//! Ephemeral Rollups Delegation Module
//!
//! Provides helpers for delegating accounts to MagicBlock's Ephemeral Rollups.
//!
//! # Usage
//!
//! Components can be delegated to ephemeral rollups for fast, gasless gameplay.
//! When delegated, the account is temporarily owned by the delegation program,
//! allowing the ER validator to process transactions.
//!
//! ## Delegation Flow
//! 1. Call `delegate_account` with the component PDA and config
//! 2. Account ownership transfers to delegation program
//! 3. ER validator can now process transactions on the account
//! 4. Call `undelegate` (or schedule commit+undelegate) to return to L1
//!
//! # Example
//! ```ignore
//! use golt_runtime::delegation::{delegate_account, DelegateConfig};
//!
//! // In your processor
//! fn process_delegate(program_id: &Pubkey, accounts: &[AccountInfo], ...) -> ProgramResult {
//!     let config = DelegateConfig {
//!         commit_frequency_ms: 10000,
//!         validator: Some(validator_pubkey),
//!     };
//!
//!     delegate_account(
//!         &[payer, component_pda, owner_program, buffer, delegation_record, delegation_metadata],
//!         &[SEED, entity.as_ref()],
//!         bump,
//!         config,
//!     )
//! }
//! ```

use pinocchio::{account_info::AccountInfo, pubkey::Pubkey};

// Re-export from ephemeral-rollups-pinocchio
pub use ephemeral_rollups_pinocchio::{
    id as DELEGATION_PROGRAM_ID,
    instruction::{delegate_account, undelegate},
    pda::{
        delegation_metadata_pda_from_delegated_account,
        delegation_record_pda_from_delegated_account,
        delegate_buffer_pda_from_delegated_account_and_owner_program,
    },
    types::DelegateConfig,
};

/// Check if an account is currently delegated to ephemeral rollups
///
/// An account is delegated when its owner is the delegation program.
#[inline]
pub fn is_delegated(account: &AccountInfo) -> bool {
    unsafe { *account.owner() == ephemeral_rollups_pinocchio::id() }
}

/// Delegation program ID
pub const DELEGATION_PROGRAM: Pubkey = ephemeral_rollups_pinocchio::id();

/// Standard delegation seeds (from ephemeral-rollups-pinocchio)
pub mod seeds {
    pub use ephemeral_rollups_pinocchio::seeds::*;
}

/// Helper to derive delegation-related PDAs
pub mod pda {
    use pinocchio::pubkey::Pubkey;

    /// Derive the buffer PDA for a delegated account
    pub fn derive_buffer_pda(account: &Pubkey, owner_program: &Pubkey) -> Pubkey {
        super::delegate_buffer_pda_from_delegated_account_and_owner_program(account, owner_program)
    }

    /// Derive the delegation record PDA
    pub fn derive_delegation_record_pda(account: &Pubkey) -> Pubkey {
        super::delegation_record_pda_from_delegated_account(account)
    }

    /// Derive the delegation metadata PDA
    pub fn derive_delegation_metadata_pda(account: &Pubkey) -> Pubkey {
        super::delegation_metadata_pda_from_delegated_account(account)
    }
}

/// Instruction discriminators for delegation callback handling
pub mod discriminators {
    /// Discriminator for undelegate callback (0xc4 prefix)
    pub const UNDELEGATE_CALLBACK: u8 = 0xc4;
}

/// Check if instruction data is an undelegate callback
///
/// The delegation program calls back with a specific discriminator.
/// Use this to detect and handle undelegate callbacks in your processor.
#[inline]
pub fn is_undelegate_callback(instruction_data: &[u8]) -> bool {
    !instruction_data.is_empty() && instruction_data[0] == discriminators::UNDELEGATE_CALLBACK
}
