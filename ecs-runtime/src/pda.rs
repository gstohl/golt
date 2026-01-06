//! PDA derivation utilities

use pinocchio::pubkey::{find_program_address, Pubkey};

/// Derive a PDA with the given seeds
pub fn derive_pda(seeds: &[&[u8]], program_id: &Pubkey) -> (Pubkey, u8) {
    find_program_address(seeds, program_id)
}

/// Verify a PDA matches the expected address
pub fn verify_pda(
    account_key: &Pubkey,
    seeds: &[&[u8]],
    program_id: &Pubkey,
) -> Result<u8, crate::GoltError> {
    let (expected, bump) = derive_pda(seeds, program_id);
    if account_key != &expected {
        return Err(crate::GoltError::InvalidPda);
    }
    Ok(bump)
}

/// Build signer seeds with bump
pub fn build_signer_seeds<'a>(seeds: &'a [&'a [u8]], bump: &'a [u8; 1]) -> Vec<&'a [u8]> {
    let mut all_seeds: Vec<&[u8]> = seeds.to_vec();
    all_seeds.push(bump);
    all_seeds
}
