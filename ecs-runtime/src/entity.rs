//! Entity Registry for the ECS framework

use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvars::{clock::Clock, rent::Rent, Sysvar},
};
use pinocchio_system::instructions::CreateAccount;

use crate::GoltError;

/// Entity discriminator (first 8 bytes of SHA256("entity"))
pub const ENTITY_DISCRIMINATOR: [u8; 8] = [0x65, 0x6e, 0x74, 0x69, 0x74, 0x79, 0x00, 0x00];

/// PDA seed prefix for entities
pub const ENTITY_SEED: &[u8] = b"entity";

/// Size of Entity struct in bytes:
/// - 8 bytes discriminator
/// - 32 bytes owner pubkey
/// - 8 bytes created_at slot
/// - 1 byte active flag
/// - 1 byte bump
pub const ENTITY_SIZE: usize = 8 + 32 + 8 + 1 + 1;

/// Entity struct representing a unique entity in the ECS
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Entity {
    /// Owner of this entity
    pub owner: Pubkey,
    /// Slot when this entity was created
    pub created_at: u64,
    /// Whether this entity is active
    pub active: bool,
    /// PDA bump seed
    pub bump: u8,
}

impl Entity {
    /// Unpack an Entity from raw account data
    pub fn unpack(data: &[u8]) -> Option<Self> {
        if data.len() < ENTITY_SIZE {
            return None;
        }

        // Verify discriminator
        if data[0..8] != ENTITY_DISCRIMINATOR {
            return None;
        }

        let owner = Pubkey::try_from(&data[8..40]).ok()?;
        let created_at = u64::from_le_bytes(data[40..48].try_into().ok()?);
        let active = data[48] != 0;
        let bump = data[49];

        Some(Self {
            owner,
            created_at,
            active,
            bump,
        })
    }

    /// Pack an Entity into raw account data
    pub fn pack(&self, data: &mut [u8]) {
        data[0..8].copy_from_slice(&ENTITY_DISCRIMINATOR);
        data[8..40].copy_from_slice(self.owner.as_ref());
        data[40..48].copy_from_slice(&self.created_at.to_le_bytes());
        data[48] = if self.active { 1 } else { 0 };
        data[49] = self.bump;
    }

    /// Check if this entity is active
    pub fn is_active(&self) -> bool {
        self.active
    }
}

/// Trait for managing entities
pub trait EntityRegistry {
    /// Get the program ID for this registry
    fn program_id() -> &'static Pubkey;

    /// Derive the PDA for an entity given its ID
    fn derive_entity_pda(entity_id: u64, program_id: &Pubkey) -> (Pubkey, u8) {
        let entity_id_bytes = entity_id.to_le_bytes();
        pinocchio::pubkey::find_program_address(
            &[ENTITY_SEED, &entity_id_bytes],
            program_id,
        )
    }

    /// Verify an entity PDA
    fn verify_entity_pda(
        account_key: &Pubkey,
        entity_id: u64,
        program_id: &Pubkey,
    ) -> Result<u8, GoltError> {
        let (expected, bump) = Self::derive_entity_pda(entity_id, program_id);
        if account_key != &expected {
            return Err(GoltError::InvalidPda);
        }
        Ok(bump)
    }
}

/// Derive the PDA for an entity given its ID
pub fn derive_entity_pda(entity_id: u64, program_id: &Pubkey) -> (Pubkey, u8) {
    let entity_id_bytes = entity_id.to_le_bytes();
    pinocchio::pubkey::find_program_address(
        &[ENTITY_SEED, &entity_id_bytes],
        program_id,
    )
}

/// Create a new entity account
pub fn create_entity<'a>(
    payer: &AccountInfo,
    entity_account: &AccountInfo,
    owner: &Pubkey,
    entity_id: u64,
    program_id: &Pubkey,
) -> Result<Entity, ProgramError> {
    // Derive PDA and verify
    let entity_id_bytes = entity_id.to_le_bytes();
    let (expected_pda, bump) = derive_entity_pda(entity_id, program_id);

    if entity_account.key() != &expected_pda {
        return Err(GoltError::InvalidPda.into());
    }

    // Check if account is already initialized
    let data = entity_account.try_borrow_data()?;
    if !data.is_empty() && data.len() >= 8 && data[0..8] == ENTITY_DISCRIMINATOR {
        return Err(GoltError::AlreadyInitialized.into());
    }
    drop(data);

    // Get rent
    let rent = Rent::get()?;
    let lamports = rent.minimum_balance(ENTITY_SIZE);

    // Build signer seeds
    let bump_bytes = [bump];
    let seeds: [Seed; 3] = [
        Seed::from(ENTITY_SEED),
        Seed::from(&entity_id_bytes[..]),
        Seed::from(&bump_bytes[..]),
    ];
    let signer = Signer::from(&seeds[..]);

    // Create the account
    CreateAccount {
        from: payer,
        to: entity_account,
        lamports,
        space: ENTITY_SIZE as u64,
        owner: program_id,
    }
    .invoke_signed(&[signer])?;

    // Get current slot for created_at
    let clock = Clock::get()?;

    // Create entity
    let entity = Entity {
        owner: *owner,
        created_at: clock.slot,
        active: true,
        bump,
    };

    // Write entity data
    let mut data = entity_account.try_borrow_mut_data()?;
    entity.pack(&mut data);

    Ok(entity)
}

/// Deactivate an entity
pub fn deactivate_entity(entity_account: &AccountInfo) -> Result<(), ProgramError> {
    let mut data = entity_account.try_borrow_mut_data()?;

    // Verify it's an entity
    if data.len() < ENTITY_SIZE || data[0..8] != ENTITY_DISCRIMINATOR {
        return Err(GoltError::InvalidDiscriminator.into());
    }

    // Check if already inactive
    if data[48] == 0 {
        return Err(GoltError::EntityNotActive.into());
    }

    // Set active flag to false
    data[48] = 0;

    Ok(())
}

/// Check if an entity is active from account data
pub fn is_entity_active(entity_account: &AccountInfo) -> Result<bool, ProgramError> {
    let data = entity_account.try_borrow_data()?;

    // Verify it's an entity
    if data.len() < ENTITY_SIZE || data[0..8] != ENTITY_DISCRIMINATOR {
        return Err(GoltError::InvalidDiscriminator.into());
    }

    Ok(data[48] != 0)
}

/// Load an entity from an account
pub fn load_entity(entity_account: &AccountInfo) -> Result<Entity, ProgramError> {
    let data = entity_account.try_borrow_data()?;
    Entity::unpack(&data).ok_or(GoltError::InvalidAccountData.into())
}
