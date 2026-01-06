//! Code templates for generating components and systems

/// Generate Cargo.toml for a component
pub fn component_cargo_toml(name: &str) -> String {
    format!(
        r#"[package]
name = "{name}"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
pinocchio.workspace = true
pinocchio-pubkey.workspace = true
pinocchio-system.workspace = true
ephemeral-rollups-pinocchio.workspace = true
ecs-core = {{ path = "../../core" }}

[lib]
crate-type = ["cdylib", "lib"]

[features]
no-entrypoint = []
"#,
        name = name
    )
}

/// Generate lib.rs for a component
pub fn component_lib_rs(snake_name: &str, pascal_name: &str) -> String {
    format!(
        r#"//! {pascal_name} Component

#[cfg(not(feature = "no-entrypoint"))]
mod entrypoint;

pub mod error;
pub mod instruction;
pub mod processor;
pub mod state;

pub use error::*;
pub use instruction::*;
pub use state::*;

// TODO: Replace with actual program ID after running `golt generate keypair {snake_name}`
pinocchio_pubkey::declare_id!("11111111111111111111111111111111");
"#,
        pascal_name = pascal_name,
        snake_name = snake_name
    )
}

/// Generate state.rs for a component
pub fn component_state_rs(snake_name: &str, pascal_name: &str, seed: &str) -> String {
    let upper_name = snake_name.to_uppercase();

    format!(
        r#"//! {pascal_name} component state

use ecs_core::discriminators;
use pinocchio::pubkey::{{find_program_address, Pubkey}};

/// {pascal_name} component size
/// Discriminator (8) + entity (32) + ... + bump (1)
pub const {upper_name}_SIZE: usize = 8 + 32 + 1; // TODO: Update size

/// {pascal_name} component
///
/// PDA: ["{seed}", entity]
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct {pascal_name} {{
    /// Discriminator
    pub discriminator: [u8; 8],
    /// Entity this component belongs to
    pub entity: [u8; 32],
    // TODO: Add your fields here

    /// PDA bump
    pub bump: u8,
}}

impl {pascal_name} {{
    pub const SIZE: usize = {upper_name}_SIZE;

    pub fn new(entity: [u8; 32], bump: u8) -> Self {{
        Self {{
            discriminator: discriminators::{upper_name},
            entity,
            // TODO: Initialize your fields
            bump,
        }}
    }}

    pub fn unpack(data: &[u8]) -> Option<Self> {{
        if data.len() < Self::SIZE {{
            return None;
        }}
        if data[0..8] != discriminators::{upper_name} {{
            return None;
        }}
        // Safe because we checked the size
        Some(unsafe {{ *(data.as_ptr() as *const Self) }})
    }}

    pub fn pack(&self, data: &mut [u8]) {{
        let src = unsafe {{
            core::slice::from_raw_parts(self as *const Self as *const u8, Self::SIZE)
        }};
        data[..Self::SIZE].copy_from_slice(src);
    }}
}}

/// Derive {pascal_name} PDA
pub fn derive_{snake_name}_pda(entity: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {{
    find_program_address(&[ecs_core::seeds::{upper_name}, entity.as_ref()], program_id)
}}
"#,
        pascal_name = pascal_name,
        snake_name = snake_name,
        upper_name = upper_name,
        seed = seed
    )
}

/// Generate instruction.rs for a component
pub fn component_instruction_rs(pascal_name: &str) -> String {
    format!(
        r#"//! {pascal_name} instructions

use pinocchio::program_error::ProgramError;

/// Instruction discriminator for delegate (used by templates)
pub const DELEGATE_DISCRIMINATOR: u8 = 253;
/// Instruction discriminator for undelegate callback
pub const UNDELEGATE_CALLBACK_DISCRIMINATOR: u8 = 0xc4;

#[derive(Clone, Copy, Debug)]
pub enum {pascal_name}Instruction {{
    /// Initialize a new {pascal_name} component
    ///
    /// Accounts:
    /// 0. `[signer, writable]` Payer
    /// 1. `[]` Entity
    /// 2. `[writable]` {pascal_name} PDA
    /// 3. `[]` System Program
    Init,

    // TODO: Add more instructions here

    /// Delegate component to Ephemeral Rollup
    ///
    /// Accounts:
    /// 0. `[signer, writable]` Payer
    /// 1. `[writable]` {pascal_name} PDA
    /// 2. `[writable]` Buffer PDA
    /// 3. `[writable]` Delegation Record PDA
    /// 4. `[writable]` Delegation Metadata PDA
    /// 5. `[]` Owner Program (this program)
    /// 6. `[]` System Program
    /// 7. `[]` Delegation Program
    Delegate {{
        commit_frequency_ms: u32,
        validator: [u8; 32],
    }},

    /// Undelegate callback (called by delegation program)
    ///
    /// Accounts:
    /// 0. `[writable]` {pascal_name} PDA
    /// 1. `[]` Buffer PDA
    Undelegate,
}}

impl {pascal_name}Instruction {{
    pub fn unpack(data: &[u8]) -> Result<Self, ProgramError> {{
        let (&tag, rest) = data
            .split_first()
            .ok_or(ProgramError::InvalidInstructionData)?;

        match tag {{
            0 => Ok(Self::Init),
            // TODO: Add more cases

            // Delegate instruction
            DELEGATE_DISCRIMINATOR => {{
                if rest.len() < 36 {{
                    return Err(ProgramError::InvalidInstructionData);
                }}
                let commit_frequency_ms = u32::from_le_bytes(rest[0..4].try_into().unwrap());
                let validator: [u8; 32] = rest[4..36].try_into().unwrap();
                Ok(Self::Delegate {{
                    commit_frequency_ms,
                    validator,
                }})
            }}

            // Undelegate instruction (explicit call)
            254 => Ok(Self::Undelegate),

            // Undelegate callback from delegation program
            UNDELEGATE_CALLBACK_DISCRIMINATOR if data.len() >= 8 => Ok(Self::Undelegate),

            _ => Err(ProgramError::InvalidInstructionData),
        }}
    }}

    pub fn pack(&self) -> Vec<u8> {{
        match self {{
            Self::Init => vec![0],
            // TODO: Add more cases
            Self::Delegate {{
                commit_frequency_ms,
                validator,
            }} => {{
                let mut data = vec![0u8; 37];
                data[0] = DELEGATE_DISCRIMINATOR;
                data[1..5].copy_from_slice(&commit_frequency_ms.to_le_bytes());
                data[5..37].copy_from_slice(validator);
                data
            }}
            Self::Undelegate => vec![254],
        }}
    }}
}}
"#,
        pascal_name = pascal_name
    )
}

/// Generate processor.rs for a component
pub fn component_processor_rs(snake_name: &str, pascal_name: &str) -> String {
    let upper_name = snake_name.to_uppercase();

    format!(
        r#"//! {pascal_name} processor

use ecs_core::{{require_keys_eq, require_signer, require_writable, EcsError}};
use ephemeral_rollups_pinocchio::{{instruction::delegate_account, types::DelegateConfig}};
use pinocchio::{{
    account_info::AccountInfo,
    instruction::{{Seed, Signer}},
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvars::{{rent::Rent, Sysvar}},
    ProgramResult,
}};
use pinocchio_system::instructions::CreateAccount;

use crate::{{
    instruction::{pascal_name}Instruction,
    state::{{derive_{snake_name}_pda, {pascal_name}, {upper_name}_SIZE}},
}};

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {{
    let instruction = {pascal_name}Instruction::unpack(instruction_data)?;

    match instruction {{
        {pascal_name}Instruction::Init => process_init(program_id, accounts),
        // TODO: Add more cases
        {pascal_name}Instruction::Delegate {{
            commit_frequency_ms,
            validator,
        }} => process_delegate(program_id, accounts, commit_frequency_ms, validator),
        {pascal_name}Instruction::Undelegate => process_undelegate(accounts),
    }}
}}

fn process_init(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {{
    let mut iter = accounts.iter();
    let payer = iter.next().ok_or(ProgramError::NotEnoughAccountKeys)?;
    let entity = iter.next().ok_or(ProgramError::NotEnoughAccountKeys)?;
    let component_account = iter.next().ok_or(ProgramError::NotEnoughAccountKeys)?;
    let _system_program = iter.next().ok_or(ProgramError::NotEnoughAccountKeys)?;

    require_signer!(payer);
    require_writable!(component_account, EcsError::AccountNotWritable);

    // Derive and verify PDA
    let (expected_pda, bump) = derive_{snake_name}_pda(entity.key(), program_id);
    require_keys_eq!(*component_account.key(), expected_pda, EcsError::InvalidAccountData);

    let rent = Rent::get()?;
    let lamports = rent.minimum_balance({upper_name}_SIZE);

    let bump_bytes = [bump];
    let signer_seeds: &[Seed] = &[
        Seed::from(ecs_core::seeds::{upper_name}),
        Seed::from(entity.key()),
        Seed::from(&bump_bytes),
    ];
    let signer = Signer::from(signer_seeds);

    CreateAccount {{
        from: payer,
        to: component_account,
        lamports,
        space: {upper_name}_SIZE as u64,
        owner: program_id,
    }}
    .invoke_signed(&[signer])?;

    // Initialize component
    let mut data = component_account.try_borrow_mut_data()?;
    let component = {pascal_name}::new(*entity.key(), bump);
    component.pack(&mut data);

    Ok(())
}}

/// Delegate component to Ephemeral Rollup
fn process_delegate(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    commit_frequency_ms: u32,
    validator: [u8; 32],
) -> ProgramResult {{
    let mut iter = accounts.iter();
    let payer = iter.next().ok_or(ProgramError::NotEnoughAccountKeys)?;
    let component_account = iter.next().ok_or(ProgramError::NotEnoughAccountKeys)?;
    let buffer = iter.next().ok_or(ProgramError::NotEnoughAccountKeys)?;
    let delegation_record = iter.next().ok_or(ProgramError::NotEnoughAccountKeys)?;
    let delegation_metadata = iter.next().ok_or(ProgramError::NotEnoughAccountKeys)?;
    let owner_program = iter.next().ok_or(ProgramError::NotEnoughAccountKeys)?;
    let _system_program = iter.next().ok_or(ProgramError::NotEnoughAccountKeys)?;
    let _delegation_program = iter.next().ok_or(ProgramError::NotEnoughAccountKeys)?;

    require_signer!(payer);
    require_writable!(component_account, EcsError::AccountNotWritable);

    // Verify owner program matches this program
    require_keys_eq!(*owner_program.key(), *program_id, EcsError::InvalidProgramId);

    // Verify account is owned by this program
    if unsafe {{ component_account.owner() }} != program_id {{
        return Err(ProgramError::IllegalOwner);
    }}

    // Get component data to extract entity and bump
    let data = component_account.try_borrow_data()?;
    let component = {pascal_name}::unpack(&data).ok_or(EcsError::NotInitialized)?;
    let bump = component.bump;
    let entity_key = component.entity;
    drop(data);

    // Build seeds for PDA signing
    let seeds: &[&[u8]] = &[ecs_core::seeds::{upper_name}, entity_key.as_ref()];

    // Configure delegation
    let validator_pubkey = Pubkey::from(validator);
    let config = DelegateConfig {{
        commit_frequency_ms,
        validator: Some(validator_pubkey),
    }};

    // Delegate to ephemeral rollup
    delegate_account(
        &[
            payer,
            component_account,
            owner_program,
            buffer,
            delegation_record,
            delegation_metadata,
        ],
        seeds,
        bump,
        config,
    )?;

    Ok(())
}}

/// Handle undelegate callback from delegation program
fn process_undelegate(accounts: &[AccountInfo]) -> ProgramResult {{
    let mut iter = accounts.iter();
    let component_account = iter.next().ok_or(ProgramError::NotEnoughAccountKeys)?;
    let _buffer = iter.next().ok_or(ProgramError::NotEnoughAccountKeys)?;

    require_writable!(component_account, EcsError::AccountNotWritable);

    // The delegation program has already restored the account data from the buffer.
    // This callback is for any post-undelegation cleanup if needed.
    // Most components don't need to do anything here.

    Ok(())
}}
"#,
        pascal_name = pascal_name,
        snake_name = snake_name,
        upper_name = upper_name
    )
}

/// Generate entrypoint.rs
pub fn component_entrypoint_rs() -> String {
    r#"//! Program entrypoint

use pinocchio::{
    account_info::AccountInfo,
    entrypoint,
    program_error::ProgramError,
    pubkey::Pubkey,
    ProgramResult,
};

use crate::processor;

entrypoint!(process_instruction);

fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    if instruction_data.is_empty() {
        return Err(ProgramError::InvalidInstructionData);
    }
    processor::process_instruction(program_id, accounts, instruction_data)
}
"#
    .to_string()
}

/// Generate error.rs for a component
pub fn component_error_rs(pascal_name: &str) -> String {
    format!(
        r#"//! {pascal_name} errors

use pinocchio::program_error::ProgramError;

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum {pascal_name}Error {{
    // TODO: Add your custom errors here
    InvalidState = 6000,
}}

impl From<{pascal_name}Error> for ProgramError {{
    fn from(e: {pascal_name}Error) -> Self {{
        ProgramError::Custom(e as u32)
    }}
}}
"#,
        pascal_name = pascal_name
    )
}

// ============================================================================
// System templates
// ============================================================================

/// Generate Cargo.toml for a system
pub fn system_cargo_toml(name: &str) -> String {
    format!(
        r#"[package]
name = "{name}"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
pinocchio.workspace = true
pinocchio-pubkey.workspace = true
pinocchio-system.workspace = true
ecs-core = {{ path = "../../core" }}
# TODO: Add component dependencies as needed
# health = {{ path = "../components/health", features = ["no-entrypoint"] }}

[lib]
crate-type = ["cdylib", "lib"]

[features]
no-entrypoint = []
"#,
        name = name
    )
}

/// Generate lib.rs for a system
pub fn system_lib_rs(snake_name: &str, pascal_name: &str) -> String {
    format!(
        r#"//! {pascal_name} System

#[cfg(not(feature = "no-entrypoint"))]
mod entrypoint;

pub mod error;
pub mod instruction;
pub mod processor;

pub use error::*;
pub use instruction::*;

// TODO: Replace with actual program ID after running `golt generate keypair {snake_name}`
pinocchio_pubkey::declare_id!("11111111111111111111111111111111");
"#,
        pascal_name = pascal_name,
        snake_name = snake_name
    )
}

/// Generate instruction.rs for a system
pub fn system_instruction_rs(pascal_name: &str) -> String {
    format!(
        r#"//! {pascal_name} system instructions

use pinocchio::program_error::ProgramError;

#[derive(Clone, Copy, Debug)]
pub enum {pascal_name}Instruction {{
    // TODO: Define your system instructions
    // Example:
    // Execute {{ param: u32 }},
}}

impl {pascal_name}Instruction {{
    pub fn unpack(data: &[u8]) -> Result<Self, ProgramError> {{
        let (&tag, rest) = data
            .split_first()
            .ok_or(ProgramError::InvalidInstructionData)?;

        match tag {{
            // TODO: Add instruction parsing
            _ => Err(ProgramError::InvalidInstructionData),
        }}
    }}

    pub fn pack(&self) -> Vec<u8> {{
        match self {{
            // TODO: Add instruction packing
        }}
    }}
}}
"#,
        pascal_name = pascal_name
    )
}

/// Generate processor.rs for a system
pub fn system_processor_rs(pascal_name: &str) -> String {
    format!(
        r#"//! {pascal_name} system processor

use pinocchio::{{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::Pubkey,
    ProgramResult,
}};

use crate::instruction::{pascal_name}Instruction;

pub fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {{
    let instruction = {pascal_name}Instruction::unpack(instruction_data)?;

    match instruction {{
        // TODO: Handle instructions
    }}
}}
"#,
        pascal_name = pascal_name
    )
}

/// Generate error.rs for a system
pub fn system_error_rs(pascal_name: &str) -> String {
    format!(
        r#"//! {pascal_name} errors

use pinocchio::program_error::ProgramError;

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum {pascal_name}Error {{
    // TODO: Add your custom errors here
    InvalidOperation = 7000,
}}

impl From<{pascal_name}Error> for ProgramError {{
    fn from(e: {pascal_name}Error) -> Self {{
        ProgramError::Custom(e as u32)
    }}
}}
"#,
        pascal_name = pascal_name
    )
}
