#[cfg(test)]
mod tests {
    use borsh::BorshDeserialize;
    use resolver_core::config::Config;
    use solana_program_test::{processor, ProgramTest};
    use solana_sdk::{signature::Keypair, signer::Signer, transaction::Transaction};

    use crate::fixtures::fixture::TestBuilder;

    #[tokio::test]
    async fn test_initialize_config_ok() {
        let mut fixture = TestBuilder::new().await;

        let mut resolver_program_client = fixture.resolver_program_client();

        let config_admin = Keypair::new();
        let config = Config::find_program_address(&resolver_program::id()).0;

        fixture
            .transfer(&config_admin.pubkey(), 10.0)
            .await
            .unwrap();

        resolver_program_client
            .initialize_config(&config, &config_admin)
            .await
            .unwrap();

        let config = resolver_program_client.get_config(&config).await.unwrap();
        assert_eq!(config.admin, config_admin.pubkey());
        // assert_eq!(config.jito_vault_program, jito_vault_program::id());
        // assert_eq!(config.ncn_count(), 0);
        // assert_eq!(config.operator_count(), 0);
        // assert_eq!(config.epoch_length(), DEFAULT_SLOTS_PER_EPOCH);
    }
}
