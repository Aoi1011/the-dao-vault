use jito_bytemuck::AccountDeserialize;
use resolver_core::{
    config::Config, ncn_resolver_program_config::NcnResolverProgramConfig,
    ncn_slash_proposal_ticket::NcnSlashProposalTicket, resolver::Resolver,
    slash_proposal::SlashProposal,
};
use solana_program::{native_token::sol_to_lamports, pubkey::Pubkey, system_instruction::transfer};
use solana_program_test::BanksClient;
use solana_sdk::{
    commitment_config::CommitmentLevel, signature::Keypair, signer::Signer,
    transaction::Transaction,
};

use super::{restaking_client::NcnRoot, TestResult};

#[derive(Debug)]
pub struct ResolverRoot {
    pub resolver_pubkey: Pubkey,
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

    pub async fn get_account<T: AccountDeserialize>(&mut self, account: &Pubkey) -> TestResult<T> {
        let account = self.banks_client.get_account(*account).await?.unwrap();
        Ok(T::try_from_slice_unchecked(&mut account.data.as_slice())?.clone())
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

    pub async fn do_initialize_ncn_resolver_program_config(
        &mut self,
        config: &Pubkey,
        ncn: &Pubkey,
        admin: &Keypair,
        veto_duration: u64,
    ) -> TestResult<()> {
        let ncn_resolver_program_config =
            NcnResolverProgramConfig::find_program_address(&resolver_program::id(), ncn).0;

        self.initialize_ncn_resolver_program_config(
            config,
            ncn,
            &ncn_resolver_program_config,
            admin,
            veto_duration,
        )
        .await?;

        Ok(())
    }

    pub async fn initialize_ncn_resolver_program_config(
        &mut self,
        config: &Pubkey,
        ncn: &Pubkey,
        ncn_resolver_program_config: &Pubkey,
        admin: &Keypair,
        veto_duration: u64,
    ) -> TestResult<()> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;
        self.process_transaction(&Transaction::new_signed_with_payer(
            &[resolver_sdk::sdk::initialize_ncn_resolver_program_config(
                &resolver_program::id(),
                config,
                ncn,
                ncn_resolver_program_config,
                &admin.pubkey(),
                veto_duration,
            )],
            Some(&admin.pubkey()),
            &[admin],
            blockhash,
        ))
        .await
    }

    pub async fn do_initialize_resolver(
        &mut self,
        ncn_root: &NcnRoot,
        resolver_admin: &Pubkey,
    ) -> TestResult<ResolverRoot> {
        // create resolver + add operator vault
        let resolver_base = Keypair::new();
        let resolver_pubkey =
            Resolver::find_program_address(&resolver_program::id(), &resolver_base.pubkey()).0;

        self.initialize_resolver(
            &ncn_root.ncn_pubkey,
            &resolver_pubkey,
            &resolver_base,
            &ncn_root.ncn_admin,
            resolver_admin,
        )
        .await?;

        Ok(ResolverRoot { resolver_pubkey })
    }

    async fn initialize_resolver(
        &mut self,
        ncn: &Pubkey,
        resolver: &Pubkey,
        base: &Keypair,
        ncn_slasher_admin: &Keypair,
        resolver_admin: &Pubkey,
    ) -> TestResult<()> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;

        self.process_transaction(&Transaction::new_signed_with_payer(
            &[resolver_sdk::sdk::initialize_resolver(
                &resolver_program::id(),
                &Config::find_program_address(&resolver_program::id()).0,
                &NcnResolverProgramConfig::find_program_address(&resolver_program::id(), ncn).0,
                ncn,
                resolver,
                &base.pubkey(),
                &ncn_slasher_admin.pubkey(),
                &self.payer.pubkey(),
                resolver_admin,
            )],
            Some(&self.payer.pubkey()),
            &[base, ncn_slasher_admin, &self.payer],
            blockhash,
        ))
        .await
    }

    pub async fn do_propose_slash(
        &mut self,
        ncn: &Pubkey,
        operator: &Pubkey,
        resolver: &Pubkey,
        slasher_admin: &Keypair,
        slash_amount: u64,
    ) -> TestResult<()> {
        // create resolver + add operator vault
        let slash_proposal = SlashProposal::find_program_address(
            &resolver_program::id(),
            &ncn,
            &operator,
            &resolver,
        )
        .0;
        let ncn_slash_proposal_ticket =
            NcnSlashProposalTicket::find_program_address(&resolver_program::id(), ncn).0;

        self.propose_slash(
            ncn,
            operator,
            resolver,
            &slash_proposal,
            &ncn_slash_proposal_ticket,
            slasher_admin,
            slash_amount,
        )
        .await
    }

    async fn propose_slash(
        &mut self,
        ncn: &Pubkey,
        operator: &Pubkey,
        resolver: &Pubkey,
        slash_proposal: &Pubkey,
        ncn_slash_proposal_ticket: &Pubkey,
        slasher_admin: &Keypair,
        slash_amount: u64,
    ) -> TestResult<()> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;

        self.process_transaction(&Transaction::new_signed_with_payer(
            &[resolver_sdk::sdk::propose_slash(
                &resolver_program::id(),
                &Config::find_program_address(&resolver_program::id()).0,
                ncn,
                operator,
                resolver,
                slash_proposal,
                ncn_slash_proposal_ticket,
                &slasher_admin.pubkey(),
                slash_amount,
            )],
            Some(&slasher_admin.pubkey()),
            &[slasher_admin],
            blockhash,
        ))
        .await
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
