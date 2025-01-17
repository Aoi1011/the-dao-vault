use borsh::{BorshDeserialize, BorshSerialize};
use shank::ShankInstruction;

#[derive(Debug, BorshSerialize, BorshDeserialize, ShankInstruction)]
pub enum ResolverInstruction {
    InitializeConfig,
    InitializeNcnResolverProgramConfig {
        veto_duration: u64,
        delete_slash_proposal_duration: u64,
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

    #[account(0, name = "config")]
    #[account(1, name = "ncn_resolver_program_config")]
    #[account(2, name = "ncn")]
    #[account(3, name = "operator")]
    #[account(4, name = "slasher")]
    #[account(5, writable, name = "slash_proposal")]
    #[account(6, writable, name = "ncn_slash_proposal_ticket")]
    #[account(7, writable, signer, name = "slasher_admin")]
    #[account(8, name = "system_program")]
    ProposeSlash {
        slash_amount: u64,
    },

    #[account(0, name = "config")]
    #[account(1, name = "ncn")]
    #[account(2, name = "operator")]
    #[account(3, name = "slasher")]
    #[account(4, name = "slash_proposal")]
    #[account(5, writable, name = "ncn_slash_proposal_ticket")]
    #[account(6, signer, name = "ncn_slasher_admin")]
    #[account(7, name = "new_resolver_info")]
    SetResolver,

    #[account(0, name = "config")]
    #[account(1, name = "ncn")]
    #[account(2, name = "operator")]
    #[account(3, name = "slasher")]
    #[account(4, name = "resolver")]
    #[account(5, writable, name = "slash_proposal")]
    #[account(6, writable, name = "ncn_slash_proposal_ticket")]
    #[account(7, signer, name = "resolver_admin")]
    #[account(8, name = "system_program")]
    VetoSlash,

    #[account(0, name = "config")]
    #[account(1, name = "vault_config")]
    #[account(2, name = "ncn")]
    #[account(3, name = "operator")]
    #[account(4, name = "slasher")]
    #[account(5, writable, name = "vault")]
    #[account(6, signer, name = "slasher_admin")]
    #[account(7, name = "ncn_operator_state")]
    #[account(8, name = "ncn_vault_ticket")]
    #[account(9, name = "vault_ncn_ticket")]
    #[account(10, writable, name = "vault_operator_delegation")]
    #[account(11, name = "ncn_vault_slasher_ticket")]
    #[account(12, name = "vault_ncn_slasker_ticket")]
    #[account(13, writable, name = "vault_ncn_slasher_operator_ticket")]
    #[account(14, writable, name = "vault_token_account")]
    #[account(15, writable, name = "slasher_token_account")]
    #[account(16, name = "resolver")]
    #[account(17, writable, name = "slash_proposal")]
    #[account(18, writable, name = "ncn_slash_proposal_ticket")]
    #[account(19, name = "token_program")]
    #[account(20, name = "jito_vault_program")]
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

    DeleteSlashProposal,
}

#[derive(Debug, BorshSerialize, BorshDeserialize, PartialEq, Eq)]
pub enum SlasherAdminRole {
    DelegateAdmin,
}
