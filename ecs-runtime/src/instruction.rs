//! Instruction parsing utilities

use pinocchio::program_error::ProgramError;

/// Trait for instruction enums
pub trait InstructionData: Sized {
    /// Unpack instruction from raw data
    fn unpack(data: &[u8]) -> Result<Self, ProgramError>;

    /// Pack instruction into raw data
    fn pack(&self) -> Vec<u8>;
}

/// Helper to read a u8 from a slice
#[inline]
pub fn read_u8(data: &[u8], offset: usize) -> Result<u8, ProgramError> {
    data.get(offset)
        .copied()
        .ok_or(ProgramError::InvalidInstructionData)
}

/// Helper to read a u16 from a slice (little-endian)
#[inline]
pub fn read_u16(data: &[u8], offset: usize) -> Result<u16, ProgramError> {
    let bytes: [u8; 2] = data
        .get(offset..offset + 2)
        .ok_or(ProgramError::InvalidInstructionData)?
        .try_into()
        .map_err(|_| ProgramError::InvalidInstructionData)?;
    Ok(u16::from_le_bytes(bytes))
}

/// Helper to read a u32 from a slice (little-endian)
#[inline]
pub fn read_u32(data: &[u8], offset: usize) -> Result<u32, ProgramError> {
    let bytes: [u8; 4] = data
        .get(offset..offset + 4)
        .ok_or(ProgramError::InvalidInstructionData)?
        .try_into()
        .map_err(|_| ProgramError::InvalidInstructionData)?;
    Ok(u32::from_le_bytes(bytes))
}

/// Helper to read a u64 from a slice (little-endian)
#[inline]
pub fn read_u64(data: &[u8], offset: usize) -> Result<u64, ProgramError> {
    let bytes: [u8; 8] = data
        .get(offset..offset + 8)
        .ok_or(ProgramError::InvalidInstructionData)?
        .try_into()
        .map_err(|_| ProgramError::InvalidInstructionData)?;
    Ok(u64::from_le_bytes(bytes))
}

/// Helper to read an i64 from a slice (little-endian)
#[inline]
pub fn read_i64(data: &[u8], offset: usize) -> Result<i64, ProgramError> {
    let bytes: [u8; 8] = data
        .get(offset..offset + 8)
        .ok_or(ProgramError::InvalidInstructionData)?
        .try_into()
        .map_err(|_| ProgramError::InvalidInstructionData)?;
    Ok(i64::from_le_bytes(bytes))
}

/// Helper to read a pubkey from a slice
#[inline]
pub fn read_pubkey(data: &[u8], offset: usize) -> Result<[u8; 32], ProgramError> {
    let bytes: [u8; 32] = data
        .get(offset..offset + 32)
        .ok_or(ProgramError::InvalidInstructionData)?
        .try_into()
        .map_err(|_| ProgramError::InvalidInstructionData)?;
    Ok(bytes)
}

/// Helper to write a u8 to a slice
#[inline]
pub fn write_u8(data: &mut [u8], offset: usize, value: u8) {
    data[offset] = value;
}

/// Helper to write a u16 to a slice (little-endian)
#[inline]
pub fn write_u16(data: &mut [u8], offset: usize, value: u16) {
    data[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}

/// Helper to write a u32 to a slice (little-endian)
#[inline]
pub fn write_u32(data: &mut [u8], offset: usize, value: u32) {
    data[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

/// Helper to write a u64 to a slice (little-endian)
#[inline]
pub fn write_u64(data: &mut [u8], offset: usize, value: u64) {
    data[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}

/// Helper to write an i64 to a slice (little-endian)
#[inline]
pub fn write_i64(data: &mut [u8], offset: usize, value: i64) {
    data[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}

/// Helper to write a pubkey to a slice
#[inline]
pub fn write_pubkey(data: &mut [u8], offset: usize, value: &[u8; 32]) {
    data[offset..offset + 32].copy_from_slice(value);
}
