#[cfg(test)]
mod tests {
    use jito_restaking_core::ncn::Ncn;
    use resolver_core::slasher::Slasher;
    use solana_sdk::signer::Signer;

    use crate::fixtures::fixture::TestBuilder;

    #[tokio::test]
    async fn test_initialize_slasher_ok() {
        let fixture = TestBuilder::new().await;
        let mut restaking_program_client = fixture.restaking_program_client();
        let mut resolver_program_client = fixture.resolver_program_client();
        let mut vault_program_client = fixture.vault_program_client();

        restaking_program_client
            .do_initialize_config()
            .await
            .unwrap();

        resolver_program_client
            .do_initialize_config()
            .await
            .unwrap();

        let (_vault_config_admin, vault_root) = vault_program_client
            .setup_config_and_vault(0, 0, 0)
            .await
            .unwrap();

        let ncn_root = restaking_program_client.do_initialize_ncn().await.unwrap();
        restaking_program_client
            .do_initialize_ncn_vault_ticket(&ncn_root, &vault_root.vault_pubkey)
            .await
            .unwrap();

        let slasher_root = resolver_program_client
            .do_initialize_slasher(&ncn_root)
            .await
            .unwrap();

        let slasher: Slasher = resolver_program_client
            .get_account(&slasher_root.slasher_pubkey)
            .await
            .unwrap();

        let ncn: Ncn = resolver_program_client
            .get_account(&ncn_root.ncn_pubkey)
            .await
            .unwrap();

        assert_eq!(slasher.admin, slasher_root.slasher_admin.pubkey());
        assert_eq!(slasher.index(), ncn.slasher_count());
    }
}
