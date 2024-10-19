#[cfg(test)]
mod tests {
    use resolver_core::ncn_resolver_program_config::NcnResolverProgramConfig;

    use crate::{
        fixtures::fixture::{ConfiguredVault, TestBuilder},
        resolver::{MAX_SLASH_AMOUNT, VETO_DURATION},
    };

    #[tokio::test]
    async fn test_initialize_ncn_resolver_program_config_ok() {
        let mut fixture = TestBuilder::new().await;
        let mut resolver_program_client = fixture.resolver_program_client();

        let deposit_fee_bps = 0;
        let withdrawal_fee_bps = 0;
        let reward_fee_bps = 0;
        let num_operators = 1;
        let slasher_amounts = vec![MAX_SLASH_AMOUNT];

        let ConfiguredVault {
            vault_program_client: _,
            vault_config_admin: _,
            vault_root: _,
            ncn_root,
            operator_roots: _,
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

        // resolver_program_client
        //     .do_initialize_config()
        //     .await
        //     .unwrap();

        // resolver_program_client
        //     .do_initialize_ncn_resolver_program_config(
        //         &Config::find_program_address(&resolver_program::id()).0,
        //         &ncn_root.ncn_pubkey,
        //         &ncn_root.ncn_admin,
        //         VETO_DURATION,
        //     )
        //     .await
        //     .unwrap();

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

        assert_eq!(ncn_resolver_program_config.veto_duration(), VETO_DURATION);
        assert_eq!(ncn_resolver_program_config.resolver_count(), 0);
    }
}
