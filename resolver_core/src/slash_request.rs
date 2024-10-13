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

    // Reserved space
    // reserved: [u8; 263],
}

impl Discriminator for SlashRequest {
    const DISCRIMINATOR: u8 = 1;
}

impl Default for SlashRequest {
    fn default() -> Self {
        Self {
            operator: Pubkey::default(),
            amount: PodU64::from(0),
            capture_slot: PodU64::from(0),
            veto_deadline_slot: PodU64::from(0),
            completed: 0,
            bump: 0,
            // reserved: [0; 263],
        }
    }
}

impl SlashRequest {
    pub fn new(
        operator: Pubkey,
        amount: u64,
        capture_slot: u64,
        veto_deadline_slot: u64,
        bump: u8,
    ) -> Self {
        Self {
            operator,
            amount: PodU64::from(amount),
            capture_slot: PodU64::from(capture_slot),
            veto_deadline_slot: PodU64::from(veto_deadline_slot),
            completed: 0,
            bump,
            // reserved: [0; 263],
        }
    }
}
