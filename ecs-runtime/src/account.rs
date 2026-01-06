//! Account wrapper types for type-safe account handling

use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvars::{rent::Rent, Sysvar},
};
use pinocchio_system::instructions::CreateAccount;

use crate::{Component, GoltError};

/// Wrapper for accounts that provides validation and typed access
pub struct AccountContext<'a> {
    accounts: &'a [AccountInfo],
    index: usize,
}

impl<'a> AccountContext<'a> {
    pub fn new(accounts: &'a [AccountInfo]) -> Self {
        Self { accounts, index: 0 }
    }

    /// Get the next account, advancing the internal index
    pub fn next(&mut self) -> Result<&'a AccountInfo, ProgramError> {
        if self.index >= self.accounts.len() {
            return Err(ProgramError::NotEnoughAccountKeys);
        }
        let account = &self.accounts[self.index];
        self.index += 1;
        Ok(account)
    }

    /// Get the next account as a signer
    pub fn next_signer(&mut self) -> Result<&'a AccountInfo, ProgramError> {
        let account = self.next()?;
        if !account.is_signer() {
            return Err(GoltError::AccountNotSigner.into());
        }
        Ok(account)
    }

    /// Get the next account as writable
    pub fn next_writable(&mut self) -> Result<&'a AccountInfo, ProgramError> {
        let account = self.next()?;
        if !account.is_writable() {
            return Err(GoltError::AccountNotWritable.into());
        }
        Ok(account)
    }

    /// Get the next account as signer and writable
    pub fn next_signer_writable(&mut self) -> Result<&'a AccountInfo, ProgramError> {
        let account = self.next()?;
        if !account.is_signer() {
            return Err(GoltError::AccountNotSigner.into());
        }
        if !account.is_writable() {
            return Err(GoltError::AccountNotWritable.into());
        }
        Ok(account)
    }

    /// Get remaining accounts
    pub fn remaining(&self) -> &'a [AccountInfo] {
        &self.accounts[self.index..]
    }
}

/// Initialize a new PDA account for a component
pub fn init_component_account<'a, C: Component>(
    payer: &AccountInfo,
    account: &AccountInfo,
    program_id: &Pubkey,
    signer_seeds: &[&[u8]],
) -> Result<(), ProgramError> {
    let rent = Rent::get()?;
    let lamports = rent.minimum_balance(C::SIZE);

    // Build signer
    let seeds: Vec<Seed> = signer_seeds
        .iter()
        .map(|s| Seed::from(*s))
        .collect();
    let signer = Signer::from(&seeds[..]);

    CreateAccount {
        from: payer,
        to: account,
        lamports,
        space: C::SIZE as u64,
        owner: program_id,
    }
    .invoke_signed(&[signer])?;

    // Write discriminator
    let mut data = account.try_borrow_mut_data()?;
    data[0..8].copy_from_slice(&C::DISCRIMINATOR);

    Ok(())
}

/// Load a component from an account
pub fn load_component<C: Component>(account: &AccountInfo) -> Result<C, ProgramError> {
    let data = account.try_borrow_data()?;
    C::unpack(&data).ok_or(GoltError::InvalidAccountData.into())
}

/// Load a component mutably from an account
pub fn load_component_mut<'a, C: Component>(
    account: &'a AccountInfo,
) -> Result<ComponentMut<'a, C>, ProgramError> {
    let data = account.try_borrow_data()?;
    let component = C::unpack(&data).ok_or(GoltError::InvalidAccountData)?;
    drop(data);
    Ok(ComponentMut {
        account,
        component,
    })
}

/// Mutable component wrapper that saves on drop
pub struct ComponentMut<'a, C: Component> {
    account: &'a AccountInfo,
    pub component: C,
}

impl<'a, C: Component> ComponentMut<'a, C> {
    /// Save the component back to the account
    pub fn save(self) -> Result<(), ProgramError> {
        let mut data = self.account.try_borrow_mut_data()?;
        self.component.pack(&mut data);
        Ok(())
    }

    /// Get a reference to the inner component
    pub fn get(&self) -> &C {
        &self.component
    }

    /// Get a mutable reference to the inner component
    pub fn get_mut(&mut self) -> &mut C {
        &mut self.component
    }
}

impl<'a, C: Component> std::ops::Deref for ComponentMut<'a, C> {
    type Target = C;
    fn deref(&self) -> &Self::Target {
        &self.component
    }
}

impl<'a, C: Component> std::ops::DerefMut for ComponentMut<'a, C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.component
    }
}
