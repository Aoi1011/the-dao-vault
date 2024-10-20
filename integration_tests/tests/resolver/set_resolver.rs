#[cfg(test)]
mod tests {
    use resolver_core::{
        ncn_slash_proposal_ticket::NcnSlashProposalTicket, slash_proposal::SlashProposal,
    };
    use solana_program::pubkey::Pubkey;
    use solana_sdk::{signature::Keypair, signer::Signer};

    use crate::{
        fixtures::fixture::{ConfiguredVault, TestBuilder},
        resolver::MAX_SLASH_AMOUNT,
    };

    #[tokio::test]
    async fn test_set_resolver_ok() {
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

        resolver_program_client
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

        let slasher_root = &slashers_amounts[0].0;
        let slash_proposal = SlashProposal::find_program_address(
            &resolver_program::id(),
            &ncn_root.ncn_pubkey,
            &operator_roots[0].operator_pubkey,
            &slasher_root.slasher_pubkey,
        )
        .0;
        let ncn_slash_proposal_ticket_pubkey = NcnSlashProposalTicket::find_program_address(
            &resolver_program::id(),
            &ncn_root.ncn_pubkey,
            &slash_proposal,
        )
        .0;
        let ncn_slash_proposal_ticket: NcnSlashProposalTicket = resolver_program_client
            .get_account(&ncn_slash_proposal_ticket_pubkey)
            .await
            .unwrap();

        assert_eq!(ncn_slash_proposal_ticket.resolver, Pubkey::default());

        let new_resolver = Keypair::new();
        resolver_program_client
            .do_set_resolver(
                &ncn_root,
                &operator_roots[0].operator_pubkey,
                &slasher_root,
                &new_resolver.pubkey(),
            )
            .await
            .unwrap();
        let ncn_slash_proposal_ticket: NcnSlashProposalTicket = resolver_program_client
            .get_account(&ncn_slash_proposal_ticket_pubkey)
            .await
            .unwrap();

        assert_eq!(ncn_slash_proposal_ticket.resolver, new_resolver.pubkey());
    }
}
