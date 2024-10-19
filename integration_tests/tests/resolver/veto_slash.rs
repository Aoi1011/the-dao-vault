#[cfg(test)]
mod tests {
    use resolver_core::slash_proposal::SlashProposal;

    use crate::{
        fixtures::fixture::{ConfiguredVault, TestBuilder},
        resolver::MAX_SLASH_AMOUNT,
    };

    #[tokio::test]
    async fn test_veto_slash_ok() {
        let mut fixture = TestBuilder::new().await;
        let mut resolver_program_client = fixture.resolver_program_client();

        let deposit_fee_bps = 0;
        let withdrawal_fee_bps = 0;
        let reward_fee_bps = 0;
        let num_operators = 1;
        let slasher_amounts = vec![MAX_SLASH_AMOUNT];

        let ConfiguredVault {
            vault_program_client: _,
            restaking_program_client: _,
            vault_config_admin: _,
            vault_root: _,
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

        let slasher_root = &slashers_amounts[0].0;

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

        let resolver_root = resolver_program_client
            .do_initialize_resolver(&ncn_root)
            .await
            .unwrap();

        //         let slasher_root = resolver_program_client
        //             .do_initialize_slasher(&ncn_root)
        //             .await
        //             .unwrap();

        resolver_program_client
            .do_propose_slash(
                &ncn_root.ncn_pubkey,
                &operator_roots[0].operator_pubkey,
                &slasher_root,
                100,
            )
            .await
            .unwrap();

        let slash_proposal_pubkey = SlashProposal::find_program_address(
            &resolver_program::id(),
            &ncn_root.ncn_pubkey,
            &operator_roots[0].operator_pubkey,
            &slasher_root.slasher_pubkey,
        )
        .0;
        let slash_proposal: SlashProposal = resolver_program_client
            .get_account(&slash_proposal_pubkey)
            .await
            .unwrap();

        assert_eq!(slash_proposal.operator, operator_roots[0].operator_pubkey);
        assert_eq!(slash_proposal.slasher, slasher_root.slasher_pubkey);
        assert_eq!(slash_proposal.amount(), 100);
        assert!(!slash_proposal.completed());

        resolver_program_client
            .do_veto_slash(
                &ncn_root.ncn_pubkey,
                &operator_roots[0].operator_pubkey,
                &slasher_root,
                &resolver_root.resolver_pubkey,
                &resolver_root.resolver_admin,
            )
            .await
            .unwrap();

        let slash_proposal: SlashProposal = resolver_program_client
            .get_account(&slash_proposal_pubkey)
            .await
            .unwrap();

        assert_eq!(slash_proposal.operator, operator_roots[0].operator_pubkey);
        assert_eq!(slash_proposal.slasher, slasher_root.slasher_pubkey);
        assert_eq!(slash_proposal.amount(), 100);
        assert!(slash_proposal.completed());
    }
}
