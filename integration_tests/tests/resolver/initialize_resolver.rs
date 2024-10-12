#[cfg(test)]
mod tests {
    use resolver_core::resolver::Resolver;
    use solana_sdk::{signature::Keypair, signer::Signer};

    use crate::fixtures::fixture::TestBuilder;

    #[tokio::test]
    async fn test_initialize_resolver_ok() {
        let fixture = TestBuilder::new().await;
        let mut restaking_program_client = fixture.restaking_program_client();
        let mut resolver_program_client = fixture.resolver_program_client();

        restaking_program_client
            .do_initialize_config()
            .await
            .unwrap();

        resolver_program_client
            .do_initialize_config()
            .await
            .unwrap();

        let mut vault_program_client = fixture.vault_program_client();
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
            .do_initialize_ncn_vault_slasher_ticket(
                &ncn_root,
                &vault_root.vault_pubkey,
                &slasher.pubkey(),
                100,
            )
            .await
            .unwrap();

        let ticket = restaking_program_client
            .get_ncn_vault_slasher_ticket(
                &ncn_root.ncn_pubkey,
                &vault_root.vault_pubkey,
                &slasher.pubkey(),
            )
            .await
            .unwrap();

        let resolver_root = resolver_program_client
            .do_initialize_resolver(&slasher, &ncn_root.ncn_pubkey, &vault_root.vault_pubkey)
            .await
            .unwrap();

        let resolver: Resolver = resolver_program_client
            .get_account(&resolver_root.resolver_pubkey)
            .await
            .unwrap();

        assert_eq!(resolver.admin, slasher.pubkey());
        assert_eq!(resolver.index(), ticket.index());
    }
}
