//! Entity Registry error types

use pinocchio::program_error::ProgramError;

/// Entity Registry errors
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u32)]
pub enum RegistryError {
    /// Invalid instruction discriminator
    InvalidInstruction = 0,
    /// Entity already exists
    EntityAlreadyExists = 1,
    /// Entity not found
    EntityNotFound = 2,
    /// Invalid entity discriminator
    InvalidEntityDiscriminator = 3,
    /// Unauthorized - not the owner
    Unauthorized = 4,
    /// Entity is not active
    EntityNotActive = 5,
    /// Invalid PDA
    InvalidPda = 6,
    /// Account not writable
    AccountNotWritable = 7,
    /// Missing required signature
    MissingSignature = 8,
}

impl From<RegistryError> for ProgramError {
    fn from(e: RegistryError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
