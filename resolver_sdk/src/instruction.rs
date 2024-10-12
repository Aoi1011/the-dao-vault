use borsh::{BorshDeserialize, BorshSerialize};

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub enum VaultInstruction {
    InitializeConfig,
    InitializeResolver,
}
