#[cfg(test)]
mod tests {
    use resolver_core::{
        ncn_slash_proposal_ticket::NcnSlashProposalTicket, slash_proposal::SlashProposal,
    };

    use crate::{
        fixtures::fixture::{ConfiguredVault, TestBuilder},
        resolver::MAX_SLASH_AMOUNT,
    };

    #[tokio::test]
    async fn test_delete_slash_proposal_ok() {
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

        let slash_proposal_pubkey = SlashProposal::find_program_address(
            &resolver_program::id(),
            &ncn_root.ncn_pubkey,
            &operator_roots[0].operator_pubkey,
            &slasher_root.slasher_pubkey,
        )
        .0;

        let ncn_slash_proposal_ticket_pubkey = NcnSlashProposalTicket::find_program_address(
            &resolver_program::id(),
            &ncn_root.ncn_pubkey,
            &slash_proposal_pubkey,
        )
        .0;

        resolver_program_client
            .do_set_resolver(
                &ncn_root,
                &operator_roots[0].operator_pubkey,
                &slasher_root,
                &resolver_root.resolver_pubkey,
            )
            .await
            .unwrap();

        resolver_program_client
            .do_veto_slash(
                &ncn_root.ncn_pubkey,
                &operator_roots[0].operator_pubkey,
                &slasher_root,
                &resolver_root,
            )
            .await
            .unwrap();

        fixture.warp_slot_incremental(101).await.unwrap();

        resolver_program_client
            .delete_slash_proposal(
                &ncn_root.ncn_pubkey,
                &operator_roots[0].operator_pubkey,
                &slasher_root.slasher_pubkey,
                &slash_proposal_pubkey,
                &ncn_slash_proposal_ticket_pubkey,
            )
            .await
            .unwrap();

        assert!(resolver_program_client
            .get_account::<SlashProposal>(&slash_proposal_pubkey)
            .await
            .is_err());
        assert!(resolver_program_client
            .get_account::<NcnSlashProposalTicket>(&ncn_slash_proposal_ticket_pubkey)
            .await
            .is_err());
    }
}
