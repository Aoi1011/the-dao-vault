#[cfg(test)]
mod tests {
    use resolver_core::{
        config::Config, ncn_resolver_program_config::NcnResolverProgramConfig, resolver::Resolver,
    };
    use solana_sdk::{signature::Keypair, signer::Signer};

    use crate::{fixtures::fixture::TestBuilder, resolver::VETO_DURATION};

    #[tokio::test]
    async fn test_initialize_resolver_ok() {
        let fixture = TestBuilder::new().await;
        let mut restaking_program_client = fixture.restaking_program_client();
        let mut resolver_program_client = fixture.resolver_program_client();
        let mut vault_program_client = fixture.vault_program_client();

        // let deposit_fee_bps = 0;
        // let withdrawal_fee_bps = 0;
        // let reward_fee_bps = 0;
        // let num_operators = 1;
        // let slasher_amounts = vec![MAX_SLASH_AMOUNT];

        // let ConfiguredVault {
        //     mut vault_program_client,
        //     mut restaking_program_client,
        //     vault_config_admin: _,
        //     vault_root: _,
        //     ncn_root: _,
        //     operator_roots: _,
        //     slashers_amounts: _,
        //     ..
        // } = fixture
        //     .setup_vault_with_ncn_and_operators(
        //         deposit_fee_bps,
        //         withdrawal_fee_bps,
        //         reward_fee_bps,
        //         num_operators,
        //         &slasher_amounts,
        //     )
        //     .await
        //     .unwrap();

        restaking_program_client
            .do_initialize_config()
            .await
            .unwrap();

        resolver_program_client
            .do_initialize_config()
            .await
            .unwrap();

        // let mut vault_program_client = fixture.vault_program_client();
        let (_vault_config_admin, vault_root) = vault_program_client
            .setup_config_and_vault(0, 0, 0)
            .await
            .unwrap();

        let ncn_root = restaking_program_client.do_initialize_ncn().await.unwrap();
        restaking_program_client
            .do_initialize_ncn_vault_ticket(&ncn_root, &vault_root.vault_pubkey)
            .await
            .unwrap();

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
        let resolver_pubkey = resolver.pubkey();
        let resolver_root = resolver_program_client
            .do_initialize_resolver(&ncn_root, &resolver.pubkey())
            .await
            .unwrap();

        let resolver: Resolver = resolver_program_client
            .get_account(&resolver_root.resolver_pubkey)
            .await
            .unwrap();

        assert_eq!(resolver.admin, resolver_pubkey);
        assert_eq!(resolver.index(), 0);

        let ncn_resolver_program_config: NcnResolverProgramConfig = resolver_program_client
            .get_account(
                &NcnResolverProgramConfig::find_program_address(
                    &resolver_program::id(),
                    &ncn_root.ncn_pubkey,
                )
                .0,
            )
            .await
            .unwrap();

        assert_eq!(ncn_resolver_program_config.resolver_count(), 1);
    }
}
