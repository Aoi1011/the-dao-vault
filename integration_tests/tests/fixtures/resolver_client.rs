use jito_bytemuck::AccountDeserialize;
use resolver_core::config::Config;
use solana_program::{native_token::sol_to_lamports, pubkey::Pubkey, system_instruction::transfer};
use solana_program_test::BanksClient;
use solana_sdk::{
    commitment_config::CommitmentLevel, signature::Keypair, signer::Signer,
    transaction::Transaction,
};

use super::TestResult;

#[derive(Debug)]
pub struct ResolverRoot {
    pub resolver_pubkey: Pubkey,
    pub resolver_admin: Keypair,
}

pub struct ResolverProgramClient {
    banks_client: BanksClient,
    payer: Keypair,
}

impl ResolverProgramClient {
    pub const fn new(banks_client: BanksClient, payer: Keypair) -> Self {
        Self {
            banks_client,
            payer,
        }
    }

    pub async fn get_config(&mut self, account: &Pubkey) -> TestResult<Config> {
        let account = self.banks_client.get_account(*account).await?.unwrap();
        Ok(Config::try_from_slice_unchecked(&mut account.data.as_slice())?.clone())
    }

    pub async fn do_initialize_config(&mut self) -> TestResult<Keypair> {
        let resolver_config_pubkey = Config::find_program_address(&resolver_program::id()).0;
        let resolver_config_admin = Keypair::new();

        self._airdrop(&resolver_config_admin.pubkey(), 1.0).await?;
        self.initialize_config(&resolver_config_pubkey, &resolver_config_admin)
            .await?;

        Ok(resolver_config_admin)
    }

    pub async fn initialize_config(
        &mut self,
        config: &Pubkey,
        config_admin: &Keypair,
    ) -> TestResult<()> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;
        self.process_transaction(&Transaction::new_signed_with_payer(
            &[resolver_sdk::sdk::initialize_config(
                &resolver_program::id(),
                config,
                &config_admin.pubkey(),
                &jito_restaking_program::id(),
                &jito_vault_program::id(),
            )],
            Some(&config_admin.pubkey()),
            &[config_admin],
            blockhash,
        ))
        .await
    }

    pub async fn _airdrop(&mut self, to: &Pubkey, sol: f64) -> TestResult<()> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;
        self.banks_client
            .process_transaction_with_preflight_and_commitment(
                Transaction::new_signed_with_payer(
                    &[transfer(&self.payer.pubkey(), to, sol_to_lamports(sol))],
                    Some(&self.payer.pubkey()),
                    &[&self.payer],
                    blockhash,
                ),
                CommitmentLevel::Processed,
            )
            .await?;
        Ok(())
    }

    pub async fn process_transaction(&mut self, tx: &Transaction) -> TestResult<()> {
        self.banks_client
            .process_transaction_with_preflight_and_commitment(
                tx.clone(),
                CommitmentLevel::Processed,
            )
            .await?;
        Ok(())
    }
}
