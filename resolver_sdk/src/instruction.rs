use borsh::{BorshDeserialize, BorshSerialize};
use shank::ShankInstruction;

#[derive(Debug, BorshSerialize, BorshDeserialize, ShankInstruction)]
pub enum ResolverInstruction {
    InitializeConfig,
    InitializeNcnResolverProgramConfig {
        veto_duration: u64,
    },
    InitializeResolver,
    InitializeSlasher,
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
}
