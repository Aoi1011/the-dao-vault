use jito_bytemuck::AccountDeserialize;
use jito_restaking_core::{
    ncn_operator_state::NcnOperatorState, ncn_vault_slasher_ticket::NcnVaultSlasherTicket,
    ncn_vault_ticket::NcnVaultTicket, operator_vault_ticket::OperatorVaultTicket,
};
use jito_vault_core::{
    vault::Vault, vault_ncn_slasher_operator_ticket::VaultNcnSlasherOperatorTicket,
    vault_ncn_slasher_ticket::VaultNcnSlasherTicket, vault_ncn_ticket::VaultNcnTicket,
    vault_operator_delegation::VaultOperatorDelegation,
};
use resolver_core::{
    config::Config, ncn_resolver_program_config::NcnResolverProgramConfig,
    ncn_slash_proposal_ticket::NcnSlashProposalTicket, resolver::Resolver,
    slash_proposal::SlashProposal, slasher::Slasher,
};
use resolver_sdk::{error::ResolverError, instruction::SlasherAdminRole};
use solana_program::{
    clock::Clock, instruction::InstructionError, native_token::sol_to_lamports, pubkey::Pubkey,
    system_instruction::transfer,
};
use solana_program_test::BanksClient;
use solana_sdk::{
    commitment_config::CommitmentLevel,
    signature::Keypair,
    signer::Signer,
    transaction::{Transaction, TransactionError},
};
use spl_associated_token_account::get_associated_token_address;

use super::{restaking_client::NcnRoot, vault_client::VaultRoot, TestError, TestResult};

#[derive(Debug)]
pub struct ResolverRoot {
    pub resolver_pubkey: Pubkey,
    pub resolver_admin: Keypair,
}

#[derive(Debug)]
pub struct SlasherRoot {
    pub slasher_pubkey: Pubkey,
    pub slasher_admin: Keypair,
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
        let account = self
            .banks_client
            .get_account(*account)
            .await?
            .ok_or(TestError::AccountNotFound)?;
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
        delete_slash_proposal_duration: u64,
    ) -> TestResult<()> {
        let ncn_resolver_program_config =
            NcnResolverProgramConfig::find_program_address(&resolver_program::id(), ncn).0;

        self.initialize_ncn_resolver_program_config(
            config,
            ncn,
            &ncn_resolver_program_config,
            admin,
            veto_duration,
            delete_slash_proposal_duration,
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
        delete_slash_proposal_duration: u64,
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
                delete_slash_proposal_duration,
            )],
            Some(&admin.pubkey()),
            &[admin],
            blockhash,
        ))
        .await
    }

    pub async fn do_initialize_resolver(&mut self, ncn_root: &NcnRoot) -> TestResult<ResolverRoot> {
        // create resolver + add operator vault
        let resolver_base = Keypair::new();
        let resolver_pubkey =
            Resolver::find_program_address(&resolver_program::id(), &resolver_base.pubkey()).0;

        let resolver_admin = Keypair::new();
        self._airdrop(&resolver_admin.pubkey(), 1.0).await?;

        self.initialize_resolver(
            &ncn_root.ncn_pubkey,
            &resolver_pubkey,
            &resolver_admin,
            &resolver_base,
        )
        .await?;

        Ok(ResolverRoot {
            resolver_pubkey,
            resolver_admin,
        })
    }

    async fn initialize_resolver(
        &mut self,
        ncn: &Pubkey,
        resolver: &Pubkey,
        admin: &Keypair,
        base: &Keypair,
    ) -> TestResult<()> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;

        self.process_transaction(&Transaction::new_signed_with_payer(
            &[resolver_sdk::sdk::initialize_resolver(
                &resolver_program::id(),
                &Config::find_program_address(&resolver_program::id()).0,
                &NcnResolverProgramConfig::find_program_address(&resolver_program::id(), ncn).0,
                ncn,
                resolver,
                &admin.pubkey(),
                &base.pubkey(),
            )],
            Some(&admin.pubkey()),
            &[admin, base],
            blockhash,
        ))
        .await
    }

    pub async fn do_initialize_slasher(&mut self, ncn_root: &NcnRoot) -> TestResult<SlasherRoot> {
        // create resolver + add operator vault
        let slasher_base = Keypair::new();
        let slasher_pubkey =
            Slasher::find_program_address(&resolver_program::id(), &slasher_base.pubkey()).0;

        let slasher_admin = Keypair::new();
        self._airdrop(&slasher_admin.pubkey(), 1.0).await?;

        self.initialize_slasher(
            &ncn_root.ncn_pubkey,
            &slasher_pubkey,
            &slasher_admin,
            &slasher_base,
        )
        .await?;

        Ok(SlasherRoot {
            slasher_pubkey,
            slasher_admin,
        })
    }

    async fn initialize_slasher(
        &mut self,
        ncn: &Pubkey,
        slasher: &Pubkey,
        admin: &Keypair,
        base: &Keypair,
    ) -> TestResult<()> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;

        self.process_transaction(&Transaction::new_signed_with_payer(
            &[resolver_sdk::sdk::initialize_slasher(
                &resolver_program::id(),
                &Config::find_program_address(&resolver_program::id()).0,
                ncn,
                slasher,
                &admin.pubkey(),
                &base.pubkey(),
            )],
            Some(&admin.pubkey()),
            &[admin, base],
            blockhash,
        ))
        .await
    }

    pub async fn do_propose_slash(
        &mut self,
        ncn: &Pubkey,
        operator: &Pubkey,
        slasher_root: &SlasherRoot,
        slash_amount: u64,
    ) -> TestResult<()> {
        // create resolver + add operator vault
        let slash_proposal = SlashProposal::find_program_address(
            &resolver_program::id(),
            &ncn,
            &operator,
            &slasher_root.slasher_pubkey,
        )
        .0;
        let ncn_slash_proposal_ticket = NcnSlashProposalTicket::find_program_address(
            &resolver_program::id(),
            ncn,
            &slash_proposal,
        )
        .0;

        self.propose_slash(
            ncn,
            operator,
            &slasher_root.slasher_pubkey,
            &slash_proposal,
            &ncn_slash_proposal_ticket,
            &slasher_root.slasher_admin,
            slash_amount,
        )
        .await
    }

    async fn propose_slash(
        &mut self,
        ncn: &Pubkey,
        operator: &Pubkey,
        slasher: &Pubkey,
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
                &NcnResolverProgramConfig::find_program_address(&resolver_program::id(), ncn).0,
                ncn,
                operator,
                slasher,
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

    pub async fn do_set_resolver(
        &mut self,
        ncn_root: &NcnRoot,
        operator: &Pubkey,
        slasher_root: &SlasherRoot,
        new_resolver_admin: &Pubkey,
    ) -> TestResult<()> {
        // create resolver + add operator vault
        let slash_proposal = SlashProposal::find_program_address(
            &resolver_program::id(),
            &ncn_root.ncn_pubkey,
            &operator,
            &slasher_root.slasher_pubkey,
        )
        .0;
        let ncn_slash_proposal_ticket = NcnSlashProposalTicket::find_program_address(
            &resolver_program::id(),
            &ncn_root.ncn_pubkey,
            &slash_proposal,
        )
        .0;

        self.set_resolver(
            &ncn_root.ncn_pubkey,
            operator,
            &slasher_root.slasher_pubkey,
            &slash_proposal,
            &ncn_slash_proposal_ticket,
            &ncn_root.ncn_admin,
            new_resolver_admin,
        )
        .await
    }

    async fn set_resolver(
        &mut self,
        ncn: &Pubkey,
        operator: &Pubkey,
        slasher: &Pubkey,
        slash_proposal: &Pubkey,
        ncn_slash_proposal_ticket: &Pubkey,
        ncn_resolver_admin: &Keypair,
        new_resolver_admin: &Pubkey,
    ) -> TestResult<()> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;

        self.process_transaction(&Transaction::new_signed_with_payer(
            &[resolver_sdk::sdk::set_resolver(
                &resolver_program::id(),
                &Config::find_program_address(&resolver_program::id()).0,
                &NcnResolverProgramConfig::find_program_address(&resolver_program::id(), ncn).0,
                ncn,
                operator,
                slasher,
                slash_proposal,
                ncn_slash_proposal_ticket,
                &ncn_resolver_admin.pubkey(),
                new_resolver_admin,
            )],
            Some(&ncn_resolver_admin.pubkey()),
            &[ncn_resolver_admin],
            blockhash,
        ))
        .await
    }

    pub async fn do_veto_slash(
        &mut self,
        ncn: &Pubkey,
        operator: &Pubkey,
        slasher_root: &SlasherRoot,
        resolver_root: &ResolverRoot,
    ) -> TestResult<()> {
        let slash_proposal = SlashProposal::find_program_address(
            &resolver_program::id(),
            &ncn,
            &operator,
            &slasher_root.slasher_pubkey,
        )
        .0;
        let ncn_slash_proposal_ticket = NcnSlashProposalTicket::find_program_address(
            &resolver_program::id(),
            ncn,
            &slash_proposal,
        )
        .0;

        self.veto_slash(
            ncn,
            operator,
            &slasher_root.slasher_pubkey,
            &resolver_root.resolver_pubkey,
            &slash_proposal,
            &ncn_slash_proposal_ticket,
            &resolver_root.resolver_admin,
        )
        .await
    }

    async fn veto_slash(
        &mut self,
        ncn: &Pubkey,
        operator: &Pubkey,
        slasher: &Pubkey,
        resolver: &Pubkey,
        slash_proposal: &Pubkey,
        ncn_slash_proposal_ticket: &Pubkey,
        resolver_admin: &Keypair,
    ) -> TestResult<()> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;

        self.process_transaction(&Transaction::new_signed_with_payer(
            &[resolver_sdk::sdk::veto_slash(
                &resolver_program::id(),
                &Config::find_program_address(&resolver_program::id()).0,
                &NcnResolverProgramConfig::find_program_address(&resolver_program::id(), ncn).0,
                ncn,
                operator,
                slasher,
                resolver,
                slash_proposal,
                ncn_slash_proposal_ticket,
                &resolver_admin.pubkey(),
            )],
            Some(&resolver_admin.pubkey()),
            &[resolver_admin],
            blockhash,
        ))
        .await
    }

    pub async fn do_execute_slash(
        &mut self,
        ncn_pubkey: &Pubkey,
        operator_pubkey: &Pubkey,
        slasher_root: &SlasherRoot,
        vault_root: &VaultRoot,
        resolver: &Pubkey,
    ) -> TestResult<()> {
        let ncn_operator_state_pubkey = NcnOperatorState::find_program_address(
            &jito_restaking_program::id(),
            ncn_pubkey,
            operator_pubkey,
        )
        .0;
        let ncn_vault_ticket_pubkey = NcnVaultTicket::find_program_address(
            &jito_restaking_program::id(),
            ncn_pubkey,
            &vault_root.vault_pubkey,
        )
        .0;
        let operator_vault_ticket_pubkey = OperatorVaultTicket::find_program_address(
            &jito_restaking_program::id(),
            operator_pubkey,
            &vault_root.vault_pubkey,
        )
        .0;
        let vault_ncn_ticket_pubkey = VaultNcnTicket::find_program_address(
            &jito_vault_program::id(),
            &vault_root.vault_pubkey,
            ncn_pubkey,
        )
        .0;
        let vault_operator_delegation = VaultOperatorDelegation::find_program_address(
            &jito_vault_program::id(),
            &vault_root.vault_pubkey,
            operator_pubkey,
        )
        .0;
        let ncn_slasher_ticket_pubkey = NcnVaultSlasherTicket::find_program_address(
            &jito_restaking_program::id(),
            ncn_pubkey,
            &vault_root.vault_pubkey,
            &slasher_root.slasher_pubkey,
        )
        .0;
        let vault_slasher_ticket_pubkey = VaultNcnSlasherTicket::find_program_address(
            &jito_vault_program::id(),
            &vault_root.vault_pubkey,
            ncn_pubkey,
            &slasher_root.slasher_pubkey,
        )
        .0;
        let config: jito_vault_core::config::Config = self
            .get_account(
                &jito_vault_core::config::Config::find_program_address(&jito_vault_program::id()).0,
            )
            .await
            .unwrap();
        let clock: Clock = self.banks_client.get_sysvar().await?;

        let vault_ncn_slasher_operator_ticket =
            VaultNcnSlasherOperatorTicket::find_program_address(
                &jito_vault_program::id(),
                &vault_root.vault_pubkey,
                ncn_pubkey,
                &slasher_root.slasher_pubkey,
                operator_pubkey,
                clock.slot / config.epoch_length(),
            )
            .0;

        let vault: Vault = self.get_account(&vault_root.vault_pubkey).await.unwrap();
        let vault_token_account =
            get_associated_token_address(&vault_root.vault_pubkey, &vault.supported_mint);
        let slasher_token_account =
            get_associated_token_address(&slasher_root.slasher_pubkey, &vault.supported_mint);

        let slash_proposal = SlashProposal::find_program_address(
            &resolver_program::id(),
            &ncn_pubkey,
            &operator_pubkey,
            &slasher_root.slasher_pubkey,
        )
        .0;
        let ncn_slash_proposal_ticket = NcnSlashProposalTicket::find_program_address(
            &resolver_program::id(),
            ncn_pubkey,
            &slash_proposal,
        )
        .0;

        self.execute_slash(
            ncn_pubkey,
            operator_pubkey,
            &slasher_root,
            &vault_root.vault_pubkey,
            &ncn_operator_state_pubkey,
            &ncn_vault_ticket_pubkey,
            &operator_vault_ticket_pubkey,
            &vault_ncn_ticket_pubkey,
            &vault_operator_delegation,
            &ncn_slasher_ticket_pubkey,
            &vault_slasher_ticket_pubkey,
            &vault_ncn_slasher_operator_ticket,
            &vault_token_account,
            &slasher_token_account,
            resolver,
            &slash_proposal,
            &ncn_slash_proposal_ticket,
        )
        .await
    }

    async fn execute_slash(
        &mut self,
        ncn: &Pubkey,
        operator: &Pubkey,
        slasher_root: &SlasherRoot,
        vault: &Pubkey,
        ncn_operator_state: &Pubkey,
        ncn_vault_ticket: &Pubkey,
        operator_vault_ticket: &Pubkey,
        vault_ncn_ticket: &Pubkey,
        vault_operator_delegation: &Pubkey,
        ncn_vault_slasher_ticket: &Pubkey,
        vault_ncn_slasher_ticket: &Pubkey,
        vault_ncn_slasher_operator_ticket: &Pubkey,
        vault_token_account: &Pubkey,
        slasher_token_account: &Pubkey,
        resolver: &Pubkey,
        slash_proposal: &Pubkey,
        ncn_slash_proposal_ticket: &Pubkey,
    ) -> TestResult<()> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;

        self.process_transaction(&Transaction::new_signed_with_payer(
            &[resolver_sdk::sdk::execute_slash(
                &resolver_program::id(),
                &Config::find_program_address(&resolver_program::id()).0,
                &NcnResolverProgramConfig::find_program_address(&resolver_program::id(), ncn).0,
                &jito_vault_core::config::Config::find_program_address(&jito_vault_program::id()).0,
                ncn,
                operator,
                &slasher_root.slasher_pubkey,
                vault,
                &slasher_root.slasher_admin.pubkey(),
                ncn_operator_state,
                ncn_vault_ticket,
                operator_vault_ticket,
                vault_ncn_ticket,
                vault_operator_delegation,
                ncn_vault_slasher_ticket,
                vault_ncn_slasher_ticket,
                vault_ncn_slasher_operator_ticket,
                vault_token_account,
                slasher_token_account,
                resolver,
                slash_proposal,
                ncn_slash_proposal_ticket,
            )],
            Some(&slasher_root.slasher_admin.pubkey()),
            &[&slasher_root.slasher_admin],
            blockhash,
        ))
        .await
    }

    pub async fn slasher_delegate_token_account(
        &mut self,
        slasher_pubkey: &Pubkey,
        delegate_admin: &Keypair,
        token_mint: &Pubkey,
        token_account: &Pubkey,
        delegate: &Pubkey,
        token_program_id: &Pubkey,
    ) -> TestResult<()> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;
        self.process_transaction(&Transaction::new_signed_with_payer(
            &[resolver_sdk::sdk::slasher_delegate_token_account(
                &resolver_program::id(),
                slasher_pubkey,
                &delegate_admin.pubkey(),
                token_mint,
                token_account,
                delegate,
                token_program_id,
            )],
            Some(&self.payer.pubkey()),
            &[&self.payer, delegate_admin],
            blockhash,
        ))
        .await
    }

    pub async fn slasher_set_admin(
        &mut self,
        slasher_pubkey: &Pubkey,
        old_admin: &Keypair,
        new_admin: &Keypair,
    ) -> TestResult<()> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;
        self.process_transaction(&Transaction::new_signed_with_payer(
            &[resolver_sdk::sdk::slasher_set_admin(
                &resolver_program::id(),
                slasher_pubkey,
                &old_admin.pubkey(),
                &new_admin.pubkey(),
            )],
            Some(&old_admin.pubkey()),
            &[old_admin, new_admin],
            blockhash,
        ))
        .await
    }
    pub async fn slasher_set_secondary_admin(
        &mut self,
        slasher_pubkey: &Pubkey,
        admin: &Keypair,
        new_admin: &Keypair,
        slasher_admin_role: SlasherAdminRole,
    ) -> TestResult<()> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;
        self.process_transaction(&Transaction::new_signed_with_payer(
            &[resolver_sdk::sdk::slasher_set_secondary_admin(
                &resolver_program::id(),
                slasher_pubkey,
                &admin.pubkey(),
                &new_admin.pubkey(),
                slasher_admin_role,
            )],
            Some(&admin.pubkey()),
            &[admin],
            blockhash,
        ))
        .await
    }

    pub async fn delete_slash_proposal(
        &mut self,
        ncn: &Pubkey,
        operator: &Pubkey,
        slasher: &Pubkey,
        slash_proposal: &Pubkey,
        ncn_slash_proposal_ticket: &Pubkey,
    ) -> TestResult<()> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;
        self.process_transaction(&Transaction::new_signed_with_payer(
            &[resolver_sdk::sdk::delete_slash_proposal(
                &resolver_program::id(),
                &Config::find_program_address(&resolver_program::id()).0,
                ncn,
                operator,
                slasher,
                slash_proposal,
                ncn_slash_proposal_ticket,
                &self.payer.pubkey(),
            )],
            Some(&self.payer.pubkey()),
            &[&self.payer],
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

#[inline(always)]
#[track_caller]
pub fn assert_resolver_error<T>(test_error: Result<T, TestError>, resolver_error: ResolverError) {
    assert!(test_error.is_err());
    assert_eq!(
        test_error.err().unwrap().to_transaction_error().unwrap(),
        TransactionError::InstructionError(0, InstructionError::Custom(resolver_error as u32))
    );
}
