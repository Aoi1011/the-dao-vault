#[cfg(test)]
mod tests {
    use jito_vault_core::{
        vault_ncn_slasher_operator_ticket::VaultNcnSlasherOperatorTicket,
        vault_ncn_slasher_ticket::VaultNcnSlasherTicket,
    };
    use resolver_core::slash_proposal::SlashProposal;
    use solana_sdk::{signature::Keypair, signer::Signer};

    use crate::{
        fixtures::fixture::{ConfiguredVault, TestBuilder},
        resolver::{DELEGATION_AMOUNT, MAX_SLASH_AMOUNT, MINT_AMOUNT},
    };

    #[tokio::test]
    async fn test_execute_slash_ok() {
        let mut fixture = TestBuilder::new().await;
        let mut resolver_program_client = fixture.resolver_program_client();

        let deposit_fee_bps = 0;
        let withdrawal_fee_bps = 0;
        let reward_fee_bps = 0;
        let num_operators = 1;
        let slasher_amounts = vec![MAX_SLASH_AMOUNT];

        let ConfiguredVault {
            mut vault_program_client,
            restaking_program_client: _,
            vault_config_admin,
            vault_root,
            ncn_root,
            operator_roots,
            slashers_amounts,
            ..
        } = fixture
            .setup_vault_with_ncn_and_operators(
                deposit_fee_bps,
                withdrawal_fee_bps,
                reward_fee_bps,
                num_operators,
                &slasher_amounts,
            )
            .await
            .unwrap();

        let depositor = Keypair::new();
        vault_program_client
            .configure_depositor(&vault_root, &depositor.pubkey(), MINT_AMOUNT)
            .await
            .unwrap();
        vault_program_client
            .do_mint_to(&vault_root, &depositor, MINT_AMOUNT, MINT_AMOUNT)
            .await
            .unwrap();

        let operator_root = &operator_roots[0];
        vault_program_client
            .do_add_delegation(
                &vault_root,
                &operator_root.operator_pubkey,
                DELEGATION_AMOUNT,
            )
            .await
            .unwrap();

        let config = vault_program_client
            .get_config(
                &jito_vault_core::config::Config::find_program_address(&jito_vault_program::id()).0,
            )
            .await
            .unwrap();
        fixture
            .warp_slot_incremental(2 * config.epoch_length())
            .await
            .unwrap();
        let operator_root_pubkeys: Vec<_> =
            operator_roots.iter().map(|r| r.operator_pubkey).collect();
        vault_program_client
            .do_full_vault_update(&vault_root.vault_pubkey, &operator_root_pubkeys)
            .await
            .unwrap();

        let vault = vault_program_client
            .get_vault(&vault_root.vault_pubkey)
            .await
            .unwrap();

        // configure slasher and slash
        let slasher_root = &slashers_amounts[0].0;

        fixture
            .create_ata(&vault.supported_mint, &slasher_root.slasher_pubkey)
            .await
            .unwrap();

        let epoch = fixture.get_current_slot().await.unwrap() / config.epoch_length();
        vault_program_client
            .initialize_vault_ncn_slasher_operator_ticket(
                &jito_vault_core::config::Config::find_program_address(&jito_vault_program::id()).0,
                &vault_root.vault_pubkey,
                &ncn_root.ncn_pubkey,
                &slasher_root.slasher_pubkey,
                &operator_root.operator_pubkey,
                &VaultNcnSlasherTicket::find_program_address(
                    &jito_vault_program::id(),
                    &vault_root.vault_pubkey,
                    &ncn_root.ncn_pubkey,
                    &slasher_root.slasher_pubkey,
                )
                .0,
                &VaultNcnSlasherOperatorTicket::find_program_address(
                    &jito_vault_program::id(),
                    &vault_root.vault_pubkey,
                    &ncn_root.ncn_pubkey,
                    &slasher_root.slasher_pubkey,
                    &operator_root.operator_pubkey,
                    epoch,
                )
                .0,
                &vault_config_admin,
            )
            .await
            .unwrap();

        let resolver_root = resolver_program_client
            .do_initialize_resolver(&ncn_root)
            .await
            .unwrap();

        resolver_program_client
            .do_propose_slash(
                &ncn_root.ncn_pubkey,
                &operator_roots[0].operator_pubkey,
                &slasher_root,
                100,
            )
            .await
            .unwrap();

        fixture.warp_slot_incremental(101).await.unwrap();

        resolver_program_client
            .do_execute_slash(
                &ncn_root.ncn_pubkey,
                &operator_roots[0].operator_pubkey,
                &slasher_root,
                &vault_root,
                &resolver_root.resolver_pubkey,
            )
            .await
            .unwrap();

        let slash_proposal: SlashProposal = resolver_program_client
            .get_account(
                &SlashProposal::find_program_address(
                    &resolver_program::id(),
                    &ncn_root.ncn_pubkey,
                    &operator_roots[0].operator_pubkey,
                    &slasher_root.slasher_pubkey,
                )
                .0,
            )
            .await
            .unwrap();

        assert_eq!(slash_proposal.operator, operator_roots[0].operator_pubkey);
        assert_eq!(slash_proposal.slasher, slasher_root.slasher_pubkey);
        assert_eq!(slash_proposal.amount(), 100);
        assert!(slash_proposal.completed());
    }
}
