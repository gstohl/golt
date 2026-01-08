//! Entity Registry instruction processor

use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    program_error::ProgramError,
    pubkey::{find_program_address, Pubkey},
    sysvars::{rent::Rent, Sysvar},
    ProgramResult,
};
use pinocchio_system::instructions::CreateAccount;

use crate::{
    error::RegistryError,
    instruction::{
        discriminator, CreateEntityInstruction, DeactivateEntityInstruction,
        TransferOwnershipInstruction,
    },
    state::{Entity, ENTITY_SEED},
};

/// Process instruction
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    if instruction_data.is_empty() {
        return Err(RegistryError::InvalidInstruction.into());
    }

    match instruction_data[0] {
        discriminator::CREATE => process_create_entity(program_id, accounts, instruction_data),
        discriminator::TRANSFER => process_transfer_ownership(accounts, instruction_data),
        discriminator::DEACTIVATE => process_deactivate_entity(accounts, instruction_data),
        _ => Err(RegistryError::InvalidInstruction.into()),
    }
}

/// Process create entity instruction
fn process_create_entity(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = CreateEntityInstruction::unpack(instruction_data)
        .ok_or(RegistryError::InvalidInstruction)?;

    if accounts.len() < 3 {
        return Err(ProgramError::NotEnoughAccountKeys);
    }

    let payer = &accounts[0];
    let entity_account = &accounts[1];
    let _system_program = &accounts[2];

    // Verify payer is signer
    if !payer.is_signer() {
        return Err(RegistryError::MissingSignature.into());
    }

    // Derive PDA
    let entity_id_bytes = instruction.entity_id.to_le_bytes();
    let seeds: &[&[u8]] = &[ENTITY_SEED, &entity_id_bytes];
    let (expected_pda, bump) = find_program_address(seeds, program_id);

    if entity_account.key() != &expected_pda {
        return Err(RegistryError::InvalidPda.into());
    }

    // Check if entity already exists
    if !entity_account.data_is_empty() {
        return Err(RegistryError::EntityAlreadyExists.into());
    }

    // Create the account
    let rent = Rent::get()?;
    let lamports = rent.minimum_balance(Entity::SIZE);

    // Build signer seeds
    let bump_bytes = [bump];
    let signer_seeds: [Seed; 3] = [
        Seed::from(ENTITY_SEED),
        Seed::from(&entity_id_bytes[..]),
        Seed::from(&bump_bytes[..]),
    ];
    let signer = Signer::from(&signer_seeds[..]);

    CreateAccount {
        from: payer,
        to: entity_account,
        lamports,
        space: Entity::SIZE as u64,
        owner: program_id,
    }
    .invoke_signed(&[signer])?;

    // Initialize entity data
    let entity = Entity::new(instruction.entity_id, *payer.key(), bump);

    let mut data = entity_account.try_borrow_mut_data()?;
    entity.pack(&mut data);

    Ok(())
}

/// Process transfer ownership instruction
fn process_transfer_ownership(accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
    let _instruction = TransferOwnershipInstruction::unpack(instruction_data)
        .ok_or(RegistryError::InvalidInstruction)?;

    if accounts.len() < 3 {
        return Err(ProgramError::NotEnoughAccountKeys);
    }

    let owner = &accounts[0];
    let entity_account = &accounts[1];
    let new_owner = &accounts[2];

    // Verify owner is signer
    if !owner.is_signer() {
        return Err(RegistryError::MissingSignature.into());
    }

    // Verify entity account is writable
    if !entity_account.is_writable() {
        return Err(RegistryError::AccountNotWritable.into());
    }

    // Load and verify entity
    let data = entity_account.try_borrow_data()?;
    let mut entity = Entity::unpack(&data).ok_or(RegistryError::InvalidEntityDiscriminator)?;
    drop(data);

    // Verify ownership
    if entity.owner != *owner.key() {
        return Err(RegistryError::Unauthorized.into());
    }

    // Verify entity is active
    if !entity.active {
        return Err(RegistryError::EntityNotActive.into());
    }

    // Transfer ownership
    entity.owner = *new_owner.key();

    let mut data = entity_account.try_borrow_mut_data()?;
    entity.pack(&mut data);

    Ok(())
}

/// Process deactivate entity instruction
fn process_deactivate_entity(accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
    let _instruction = DeactivateEntityInstruction::unpack(instruction_data)
        .ok_or(RegistryError::InvalidInstruction)?;

    if accounts.len() < 2 {
        return Err(ProgramError::NotEnoughAccountKeys);
    }

    let owner = &accounts[0];
    let entity_account = &accounts[1];

    // Verify owner is signer
    if !owner.is_signer() {
        return Err(RegistryError::MissingSignature.into());
    }

    // Verify entity account is writable
    if !entity_account.is_writable() {
        return Err(RegistryError::AccountNotWritable.into());
    }

    // Load and verify entity
    let data = entity_account.try_borrow_data()?;
    let mut entity = Entity::unpack(&data).ok_or(RegistryError::InvalidEntityDiscriminator)?;
    drop(data);

    // Verify ownership
    if entity.owner != *owner.key() {
        return Err(RegistryError::Unauthorized.into());
    }

    // Deactivate
    entity.active = false;

    let mut data = entity_account.try_borrow_mut_data()?;
    entity.pack(&mut data);

    Ok(())
}
