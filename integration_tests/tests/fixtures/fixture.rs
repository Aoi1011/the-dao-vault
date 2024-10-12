use solana_program::{native_token::sol_to_lamports, pubkey::Pubkey, system_instruction::transfer};
use solana_program_test::{processor, BanksClientError, ProgramTest, ProgramTestContext};
use solana_sdk::{commitment_config::CommitmentLevel, signer::Signer, transaction::Transaction};

use super::{
    resolver_client::ResolverProgramClient, restaking_client::RestakingProgramClient,
    vault_client::VaultProgramClient,
};

pub struct TestBuilder {
    context: ProgramTestContext,
}

impl std::fmt::Debug for TestBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TestBuilder",)
    }
}

// pub struct ConfiguredVault {
//     pub vault_program_client: VaultProgramClient,
//     pub restaking_program_client: RestakingProgramClient,
//     pub resolver_program_client: ResolverProgramClient,
//     pub vault_config_admin: Keypair,
//     pub vault_root: VaultRoot,
//     pub restaking_config_admin: Keypair,
//     pub ncn_root: NcnRoot,
//     pub operator_roots: Vec<OperatorRoot>,
//     pub slashers_amounts: Vec<(Keypair, u64)>,
// }

impl TestBuilder {
    pub async fn new() -> Self {
        // $ cargo-build-sbf && SBF_OUT_DIR=$(pwd)/target/sbf-solana-solana/release cargo nextest run
        let mut program_test = ProgramTest::default();
        program_test.add_program(
            "resolver_program",
            resolver_program::id(),
            processor!(resolver_program::process_instruction),
        );
        program_test.prefer_bpf(true);
        program_test.add_program("jito_restaking_program", jito_restaking_program::id(), None);
        program_test.add_program("jito_vault_program", jito_vault_program::id(), None);

        let context = program_test.start_with_context().await;
        Self { context }
    }

    pub fn vault_program_client(&self) -> VaultProgramClient {
        VaultProgramClient::new(
            self.context.banks_client.clone(),
            self.context.payer.insecure_clone(),
        )
    }

    pub fn restaking_program_client(&self) -> RestakingProgramClient {
        RestakingProgramClient::new(
            self.context.banks_client.clone(),
            self.context.payer.insecure_clone(),
        )
    }

    pub fn resolver_program_client(&self) -> ResolverProgramClient {
        ResolverProgramClient::new(
            self.context.banks_client.clone(),
            self.context.payer.insecure_clone(),
        )
    }

    pub async fn transfer(&mut self, to: &Pubkey, sol: f64) -> Result<(), BanksClientError> {
        let blockhash = self.context.banks_client.get_latest_blockhash().await?;
        self.context
            .banks_client
            .process_transaction_with_preflight_and_commitment(
                Transaction::new_signed_with_payer(
                    &[transfer(
                        &self.context.payer.pubkey(),
                        to,
                        sol_to_lamports(sol),
                    )],
                    Some(&self.context.payer.pubkey()),
                    &[&self.context.payer],
                    blockhash,
                ),
                CommitmentLevel::Processed,
            )
            .await
    }
}
