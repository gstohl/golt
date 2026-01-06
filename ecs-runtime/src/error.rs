//! Common ECS errors

use pinocchio::program_error::ProgramError;
use thiserror::Error;

/// Common errors shared across all ECS programs
#[derive(Error, Debug, Clone, Copy, PartialEq)]
#[repr(u32)]
pub enum GoltError {
    #[error("Account not initialized")]
    NotInitialized = 1000,

    #[error("Account already initialized")]
    AlreadyInitialized = 1001,

    #[error("Invalid authority")]
    InvalidAuthority = 1002,

    #[error("Invalid account data")]
    InvalidAccountData = 1003,

    #[error("Account not writable")]
    AccountNotWritable = 1004,

    #[error("Account not signer")]
    AccountNotSigner = 1005,

    #[error("Invalid program ID")]
    InvalidProgramId = 1006,

    #[error("Invalid PDA")]
    InvalidPda = 1007,

    #[error("Invalid discriminator")]
    InvalidDiscriminator = 1008,

    #[error("Arithmetic overflow")]
    Overflow = 1009,

    #[error("Arithmetic underflow")]
    Underflow = 1010,

    #[error("Invalid instruction data")]
    InvalidInstructionData = 1011,

    #[error("Entity not active")]
    EntityNotActive = 1012,

    #[error("Component not found")]
    ComponentNotFound = 1013,
}

impl From<GoltError> for ProgramError {
    fn from(e: GoltError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

/// Require a condition to be true
#[macro_export]
macro_rules! require {
    ($cond:expr, $err:expr) => {
        if !$cond {
            return Err($err.into());
        }
    };
}

/// Require two keys to be equal
#[macro_export]
macro_rules! require_keys_eq {
    ($a:expr, $b:expr, $err:expr) => {
        if $a != $b {
            return Err($err.into());
        }
    };
}

/// Require an account to be a signer
#[macro_export]
macro_rules! require_signer {
    ($account:expr) => {
        if !$account.is_signer() {
            return Err($crate::GoltError::AccountNotSigner.into());
        }
    };
    ($account:expr, $err:expr) => {
        if !$account.is_signer() {
            return Err($err.into());
        }
    };
}

/// Require an account to be writable
#[macro_export]
macro_rules! require_writable {
    ($account:expr) => {
        if !$account.is_writable() {
            return Err($crate::GoltError::AccountNotWritable.into());
        }
    };
    ($account:expr, $err:expr) => {
        if !$account.is_writable() {
            return Err($err.into());
        }
    };
}

/// Require an account to be owned by a specific program
#[macro_export]
macro_rules! require_owner {
    ($account:expr, $owner:expr) => {
        if unsafe { $account.owner() } != $owner {
            return Err($crate::GoltError::InvalidProgramId.into());
        }
    };
    ($account:expr, $owner:expr, $err:expr) => {
        if unsafe { $account.owner() } != $owner {
            return Err($err.into());
        }
    };
}
