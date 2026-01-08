//! Entity Registry instructions

/// Instruction discriminators
pub mod discriminator {
    /// Create entity instruction
    pub const CREATE: u8 = 0;
    /// Transfer ownership instruction
    pub const TRANSFER: u8 = 1;
    /// Deactivate entity instruction
    pub const DEACTIVATE: u8 = 2;
}

/// Create entity instruction data
///
/// Accounts:
/// 0. `[signer, writable]` Payer (becomes owner)
/// 1. `[writable]` Entity PDA
/// 2. `[]` System program
#[repr(C)]
pub struct CreateEntityInstruction {
    /// Instruction discriminator (0)
    pub discriminator: u8,
    /// Entity ID
    pub entity_id: u64,
}

impl CreateEntityInstruction {
    pub const SIZE: usize = 9; // 1 + 8

    pub fn unpack(data: &[u8]) -> Option<Self> {
        if data.len() < Self::SIZE {
            return None;
        }
        if data[0] != discriminator::CREATE {
            return None;
        }

        let entity_id = u64::from_le_bytes(data[1..9].try_into().ok()?);
        Some(Self {
            discriminator: discriminator::CREATE,
            entity_id,
        })
    }
}

/// Transfer ownership instruction data
///
/// Accounts:
/// 0. `[signer]` Current owner
/// 1. `[writable]` Entity PDA
/// 2. `[]` New owner
#[repr(C)]
pub struct TransferOwnershipInstruction {
    /// Instruction discriminator (1)
    pub discriminator: u8,
}

impl TransferOwnershipInstruction {
    pub const SIZE: usize = 1;

    pub fn unpack(data: &[u8]) -> Option<Self> {
        if data.is_empty() {
            return None;
        }
        if data[0] != discriminator::TRANSFER {
            return None;
        }

        Some(Self {
            discriminator: discriminator::TRANSFER,
        })
    }
}

/// Deactivate entity instruction data
///
/// Accounts:
/// 0. `[signer]` Owner
/// 1. `[writable]` Entity PDA
#[repr(C)]
pub struct DeactivateEntityInstruction {
    /// Instruction discriminator (2)
    pub discriminator: u8,
}

impl DeactivateEntityInstruction {
    pub const SIZE: usize = 1;

    pub fn unpack(data: &[u8]) -> Option<Self> {
        if data.is_empty() {
            return None;
        }
        if data[0] != discriminator::DEACTIVATE {
            return None;
        }

        Some(Self {
            discriminator: discriminator::DEACTIVATE,
        })
    }
}
