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
///
/// Implement this trait for components that need to be delegated to MagicBlock's
/// Ephemeral Rollups for fast, gasless gameplay.
///
/// # Example
/// ```ignore
/// impl Delegatable for MyComponent {
///     fn get_entity(&self) -> &Pubkey {
///         &self.entity
///     }
///
///     fn get_bump(&self) -> u8 {
///         self.bump
///     }
/// }
/// ```
pub trait Delegatable: Component {
    /// Get the entity this component belongs to
    fn get_entity(&self) -> &Pubkey;

    /// Get the PDA bump for this component
    fn get_bump(&self) -> u8;

    /// Build the PDA seeds for signing delegation transactions
    fn delegation_seeds(&self) -> Vec<&[u8]> {
        vec![Self::SEED, self.get_entity().as_ref()]
    }
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
