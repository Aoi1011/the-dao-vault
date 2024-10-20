#[cfg(test)]
mod tests {
    use solana_program::{program_option::COption, pubkey::Pubkey};
    use solana_sdk::{signature::Keypair, signer::Signer};
    use spl_associated_token_account::get_associated_token_address;
    use test_case::test_case;

    use crate::{
        fixtures::fixture::{ConfiguredVault, TestBuilder},
        resolver::{MAX_SLASH_AMOUNT, MINT_AMOUNT},
    };

    #[test_case(spl_token::id(); "token")]
    #[test_case(spl_token_2022::id(); "token-2022")]
    #[tokio::test]
    async fn test_initialize_slasher_ok(token_program_id: Pubkey) {
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

        let random_mint = Keypair::new();
        fixture
            .vault_program_client()
            .create_token_mint(&random_mint, &token_program_id)
            .await
            .unwrap();

        let slasher_root = &slashers_amounts[0].0;
        let slasher_token_account = Keypair::new();
        if token_program_id.eq(&spl_token::id()) {
            fixture
                .mint_spl_to(
                    &random_mint.pubkey(),
                    &slasher_root.slasher_pubkey,
                    MINT_AMOUNT,
                    &token_program_id,
                )
                .await
                .unwrap();
        } else {
            fixture
                .create_token_account(
                    &token_program_id,
                    &slasher_token_account,
                    &random_mint.pubkey(),
                    &slasher_root.slasher_pubkey,
                    &[],
                )
                .await
                .unwrap();
            fixture
                .mint_spl_to(
                    &random_mint.pubkey(),
                    &slasher_token_account.pubkey(),
                    MINT_AMOUNT,
                    &token_program_id,
                )
                .await
                .unwrap();
        }

        let bob = Pubkey::new_unique();
        if token_program_id.eq(&spl_token::id()) {
            // Delegate
            resolver_program_client
                .slasher_delegate_token_account(
                    &slasher_root.slasher_pubkey,
                    &slasher_root.slasher_admin,
                    &random_mint.pubkey(),
                    &get_associated_token_address(
                        &slasher_root.slasher_pubkey,
                        &random_mint.pubkey(),
                    ),
                    &bob,
                    &token_program_id,
                )
                .await
                .unwrap();
            let ata =
                get_associated_token_address(&slasher_root.slasher_pubkey, &random_mint.pubkey());
            let token_account_acc = fixture.get_token_account(&ata).await.unwrap();

            assert_eq!(token_account_acc.delegate, COption::Some(bob));
            assert_eq!(token_account_acc.delegated_amount, u64::MAX);
        } else {
            resolver_program_client
                .slasher_delegate_token_account(
                    &slasher_root.slasher_pubkey,
                    &slasher_root.slasher_admin,
                    &random_mint.pubkey(),
                    &slasher_token_account.pubkey(),
                    &bob,
                    &token_program_id,
                )
                .await
                .unwrap();

            let vault_token_acc = fixture
                .get_token_account(&slasher_token_account.pubkey())
                .await
                .unwrap();

            assert_eq!(vault_token_acc.delegate, COption::Some(bob));
            assert_eq!(vault_token_acc.delegated_amount, u64::MAX);
        }
    }
}
