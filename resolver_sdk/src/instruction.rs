use borsh::{BorshDeserialize, BorshSerialize};

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub enum ResolverInstruction {
    InitializeConfig,
    InitializeResolver,
    InitializeSlashRequestList,
}
