//! Entity state definition

use pinocchio::pubkey::Pubkey;

/// Entity discriminator: "entity\0\0"
pub const ENTITY_DISCRIMINATOR: [u8; 8] = [0x65, 0x6e, 0x74, 0x69, 0x74, 0x79, 0x00, 0x00];

/// Entity seed for PDA derivation
pub const ENTITY_SEED: &[u8] = b"entity";

/// Entity state stored in PDA
/// PDA: ["entity", entity_id (u64 le bytes)]
#[repr(C)]
pub struct Entity {
    /// Discriminator for account type verification
    pub discriminator: [u8; 8],
    /// Unique entity ID
    pub id: u64,
    /// Owner of this entity (can transfer ownership)
    pub owner: Pubkey,
    /// Whether the entity is active
    pub active: bool,
    /// PDA bump seed
    pub bump: u8,
    /// Reserved for future use
    pub _reserved: [u8; 6],
}

impl Entity {
    /// Size of Entity account in bytes
    /// 8 (discriminator) + 8 (id) + 32 (owner) + 1 (active) + 1 (bump) + 6 (reserved) = 56
    pub const SIZE: usize = 56;

    /// Unpack entity from account data
    pub fn unpack(data: &[u8]) -> Option<Self> {
        if data.len() < Self::SIZE {
            return None;
        }

        let discriminator: [u8; 8] = data[0..8].try_into().ok()?;
        if discriminator != ENTITY_DISCRIMINATOR {
            return None;
        }

        let id = u64::from_le_bytes(data[8..16].try_into().ok()?);
        let owner: [u8; 32] = data[16..48].try_into().ok()?;
        let active = data[48] != 0;
        let bump = data[49];
        let _reserved: [u8; 6] = data[50..56].try_into().ok()?;

        Some(Self {
            discriminator,
            id,
            owner: Pubkey::from(owner),
            active,
            bump,
            _reserved,
        })
    }

    /// Pack entity into account data
    pub fn pack(&self, data: &mut [u8]) {
        data[0..8].copy_from_slice(&self.discriminator);
        data[8..16].copy_from_slice(&self.id.to_le_bytes());
        data[16..48].copy_from_slice(self.owner.as_ref());
        data[48] = if self.active { 1 } else { 0 };
        data[49] = self.bump;
        data[50..56].copy_from_slice(&self._reserved);
    }

    /// Create a new entity
    pub fn new(id: u64, owner: Pubkey, bump: u8) -> Self {
        Self {
            discriminator: ENTITY_DISCRIMINATOR,
            id,
            owner,
            active: true,
            bump,
            _reserved: [0u8; 6],
        }
    }
}
