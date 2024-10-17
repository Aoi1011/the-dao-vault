use borsh::{BorshDeserialize, BorshSerialize};

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub enum ResolverInstruction {
    InitializeConfig,
    InitializeNcnResolverProgramConfig { veto_duration: u64 },
    InitializeResolver,
    InitializeSlasher,
    ProposeSlash { slash_amount: u64 },
    VetoSlash,
    ExecuteSlash,
}
