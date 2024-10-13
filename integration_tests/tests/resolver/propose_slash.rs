#[cfg(test)]
mod tests {
    use resolver_core::{config::Config, slash_proposal::SlashProposal};
    use solana_sdk::{signature::Keypair, signer::Signer};

    use crate::{
        fixtures::fixture::{ConfiguredVault, TestBuilder},
        resolver::{MAX_SLASH_AMOUNT, VETO_DURATION},
    };

    #[tokio::test]
    async fn test_propose_slash_ok() {
        let mut fixture = TestBuilder::new().await;
        // let mut restaking_program_client = fixture.restaking_program_client();
        let mut resolver_program_client = fixture.resolver_program_client();

        let deposit_fee_bps = 0;
        let withdrawal_fee_bps = 0;
        let reward_fee_bps = 0;
        let num_operators = 1;
        let slasher_amounts = vec![MAX_SLASH_AMOUNT];

        let ConfiguredVault {
            vault_program_client: _,
            mut restaking_program_client,
            vault_config_admin: _,
            vault_root,
            ncn_root,
            operator_roots,
            slashers_amounts: _,
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

        // restaking_program_client
        //     .do_initialize_config()
        //     .await
        //     .unwrap();

        resolver_program_client
            .do_initialize_config()
            .await
            .unwrap();

        // let mut vault_program_client = fixture.vault_program_client();
        // let (_vault_config_admin, vault_root) = vault_program_client
        //     .setup_config_and_vault(0, 0, 0)
        //     .await
        //     .unwrap();

        // let ncn_root = restaking_program_client.do_initialize_ncn().await.unwrap();
        // restaking_program_client
        //     .do_initialize_ncn_vault_ticket(&ncn_root, &vault_root.vault_pubkey)
        //     .await
        //     .unwrap();

        let slasher = Keypair::new();
        restaking_program_client
            ._airdrop(&slasher.pubkey(), 100.0)
            .await
            .unwrap();
        restaking_program_client
            .do_initialize_ncn_vault_slasher_ticket(
                &ncn_root,
                &vault_root.vault_pubkey,
                &slasher.pubkey(),
                100,
            )
            .await
            .unwrap();

        // let ticket = restaking_program_client
        //     .get_ncn_vault_slasher_ticket(
        //         &ncn_root.ncn_pubkey,
        //         &vault_root.vault_pubkey,
        //         &slasher.pubkey(),
        //     )
        //     .await
        //     .unwrap();
        resolver_program_client
            .do_initialize_ncn_resolver_program_config(
                &Config::find_program_address(&resolver_program::id()).0,
                &ncn_root.ncn_pubkey,
                &ncn_root.ncn_admin,
                VETO_DURATION,
            )
            .await
            .unwrap();

        let resolver = Keypair::new();
        let resolver_root = resolver_program_client
            .do_initialize_resolver(&ncn_root, &resolver.pubkey())
            .await
            .unwrap();

        resolver_program_client
            .do_propose_slash(
                &ncn_root.ncn_pubkey,
                &operator_roots[0].operator_pubkey,
                &resolver_root.resolver_pubkey,
                &slasher,
                100,
            )
            .await
            .unwrap();

        let slash_proposal: SlashProposal = resolver_program_client
            .get_account(
                &SlashProposal::find_program_address(
                    &resolver_program::id(),
                    &ncn_root.ncn_pubkey,
                    &operator_roots[0].operator_pubkey,
                    &resolver_root.resolver_pubkey,
                )
                .0,
            )
            .await
            .unwrap();

        assert_eq!(slash_proposal.operator, operator_roots[0].operator_pubkey);
        assert_eq!(slash_proposal.resolver, resolver_root.resolver_pubkey);
        assert_eq!(slash_proposal.amount(), 100);
        assert_eq!(slash_proposal.completed, 0);
    }
}
