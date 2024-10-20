use borsh::{BorshDeserialize, BorshSerialize};
use shank::ShankInstruction;

#[derive(Debug, BorshSerialize, BorshDeserialize, ShankInstruction)]
pub enum ResolverInstruction {
    InitializeConfig,
    InitializeNcnResolverProgramConfig {
        veto_duration: u64,
    },

    #[account(0, name = "config")]
    #[account(1, name = "ncn")]
    #[account(2, writable, name = "slasher")]
    #[account(3, writable, signer, name = "admin")]
    #[account(4, signer, name = "base")]
    #[account(5, name = "system_program")]
    InitializeSlasher,

    #[account(0, name = "config")]
    #[account(1, writable, name = "ncn_resolver_program_config")]
    #[account(2, name = "ncn")]
    #[account(3, writable, name = "resolver")]
    #[account(4, writable, signer, name = "admin")]
    #[account(5, signer, name = "base")]
    #[account(6, name = "system_program")]
    InitializeResolver,


    ProposeSlash {
        slash_amount: u64,
    },
    SetResolver,
    VetoSlash,
    ExecuteSlash,

    #[account(0, name = "slasher")]
    #[account(1, signer, name = "delegate_admin")]
    #[account(2, name = "token_mint")]
    #[account(3, writable, name = "token_account")]
    #[account(4, name = "delegate")]
    #[account(5, name = "token_program")]
    SlasherDelegateTokenAccount,

    /// Sets the admin for a slasher
    #[account(0, writable, name = "slasher")]
    #[account(1, signer, name = "old_admin")]
    #[account(2, signer, name = "new_admin")]
    SlasherSetAdmin,

    /// Sets the secondary admin for a slasher
    #[account(0, writable, name = "slasher")]
    #[account(1, signer, name = "admin")]
    #[account(2, name = "new_admin")]
    SlasherSetSecondaryAdmin(SlasherAdminRole),
}

#[derive(Debug, BorshSerialize, BorshDeserialize, PartialEq, Eq)]
pub enum SlasherAdminRole {
    DelegateAdmin,
}
