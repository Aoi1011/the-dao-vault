#[cfg(test)]
mod tests {
    use resolver_core::slasher::Slasher;
    use resolver_sdk::instruction::SlasherAdminRole;
    use solana_sdk::{signature::Keypair, signer::Signer};

    use crate::{
        fixtures::fixture::{ConfiguredVault, TestBuilder},
        resolver::MAX_SLASH_AMOUNT,
    };

    #[tokio::test]
    async fn test_slasher_set_secondary_admin_ok() {
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
            ncn_root: _,
            operator_roots: _,
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
        let new_admin = Keypair::new();
        resolver_program_client
            .slasher_set_secondary_admin(
                &slasher_root.slasher_pubkey,
                &slasher_root.slasher_admin,
                &new_admin,
                SlasherAdminRole::DelegateAdmin,
            )
            .await
            .unwrap();

        let slasher: Slasher = resolver_program_client
            .get_account(&slasher_root.slasher_pubkey)
            .await
            .unwrap();

        assert_eq!(slasher.delegate_admin, new_admin.pubkey());
    }
}
