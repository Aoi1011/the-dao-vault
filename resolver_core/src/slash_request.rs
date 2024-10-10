use bytemuck::{Pod, Zeroable};
use jito_bytemuck::{types::PodU64, AccountDeserialize, Discriminator};
use shank::ShankAccount;
use solana_program::pubkey::Pubkey;

/// The vault configuration account for the vault program.
/// Manages program-wide settings and state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Pod, Zeroable, AccountDeserialize, ShankAccount)]
#[repr(C)]
pub struct SlashRequest {
    /// The operator account
    pub operator: Pubkey,

    /// The slash amount
    pub amount: PodU64,

    pub capture_slot: PodU64,

    pub veto_deadline_slot: PodU64,

    pub completed: u8,

    /// The bump seed for the PDA
    pub bump: u8,

    /// Reserved space
    reserved: [u8; 263],
}

impl Discriminator for SlashRequest {
    const DISCRIMINATOR: u8 = 1;
}
