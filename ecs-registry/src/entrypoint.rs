//! Entity Registry program entrypoint

use pinocchio::{account_info::AccountInfo, entrypoint, pubkey::Pubkey, ProgramResult};

use crate::processor::process_instruction;

entrypoint!(process_entrypoint);

fn process_entrypoint(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    if let Err(error) = process_instruction(program_id, accounts, instruction_data) {
        return Err(error);
    }
    Ok(())
}
