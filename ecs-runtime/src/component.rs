//! Component trait and utilities

use pinocchio::pubkey::Pubkey;

/// Trait implemented by all ECS components
pub trait Component: Sized {
    /// Component discriminator (8 bytes)
    const DISCRIMINATOR: [u8; 8];

    /// PDA seed prefix
    const SEED: &'static [u8];

    /// Total size of the component in bytes (including discriminator)
    const SIZE: usize;

    /// Unpack component from raw account data
    fn unpack(data: &[u8]) -> Option<Self>;

    /// Pack component into raw account data
    fn pack(&self, data: &mut [u8]);

    /// Derive the PDA for this component
    fn derive_pda(seeds: &[&[u8]], program_id: &Pubkey) -> (Pubkey, u8) {
        let mut all_seeds = vec![Self::SEED];
        all_seeds.extend_from_slice(seeds);
        pinocchio::pubkey::find_program_address(&all_seeds, program_id)
    }

    /// Verify the discriminator matches
    fn verify_discriminator(data: &[u8]) -> bool {
        if data.len() < 8 {
            return false;
        }
        data[0..8] == Self::DISCRIMINATOR
    }
}

/// Trait for components that can be delegated to Ephemeral Rollups
pub trait Delegatable: Component {
    /// Check if the component is currently delegated
    fn is_delegated(&self) -> bool;
}

/// Helper to calculate struct size at compile time
#[macro_export]
macro_rules! component_size {
    // Base case: discriminator
    () => { 8 };
    // Recursive case
    ($t:ty $(, $rest:ty)*) => {
        std::mem::size_of::<$t>() + component_size!($($rest),*)
    };
}
