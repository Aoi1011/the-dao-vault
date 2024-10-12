#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use borsh::BorshDeserialize;
    use litesvm::LiteSVM;
    use solana_sdk::{
        native_token::LAMPORTS_PER_SOL, signature::Keypair, signer::Signer,
        transaction::Transaction,
    };
    use test_log;
    use vault_sdk::inline_mpl_token_metadata;

    fn read_vault_program() -> Vec<u8> {
        let mut so_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        so_path.push("../target/deploy/vault_program.so");
        println!("SO path: {:?}", so_path.to_str());
        std::fs::read(so_path).unwrap()
    }

    fn read_mpl_token_metadata_program() -> Vec<u8> {
        let mut so_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        so_path.push("tests/fixtures/mpl_token_metadata.so");
        println!("SO path: {:?}", so_path.to_str());
        std::fs::read(so_path).unwrap()
    }

    #[test_log::test]
    fn test_create_token_metadata_ok_lite_svm() {
        let mut svm = LiteSVM::new();
        svm.add_program(vault_program::id(), &read_vault_program());
        svm.add_program(
            inline_mpl_token_metadata::id(),
            &read_mpl_token_metadata_program(),
        );

        let payer_kp = Keypair::new();
        let payer_pk = payer_kp.pubkey();

        svm.airdrop(&payer_pk, 10 * LAMPORTS_PER_SOL).unwrap();

        let mint_account = Keypair::new();

        // Create token metadata
        let name = "restaking JTO";
        let symbol = "rJTO";
        let uri = "https://www.jito.network/restaking/";

        let metadata_pubkey =
            inline_mpl_token_metadata::pda::find_metadata_account(&mint_account.pubkey()).0;

        let ix = vault_sdk::sdk::create_token_metadata(
            &vault_program::id(),
            &mint_account.pubkey(),
            &payer_pk,
            &metadata_pubkey,
            &payer_pk,
            &spl_token::id(),
            name.to_string(),
            symbol.to_string(),
            uri.to_string(),
        );

        let blockhash = svm.latest_blockhash();
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&payer_pk),
            &[&mint_account, &payer_kp],
            blockhash,
        );

        let tx_result = svm.send_transaction(tx);
        assert!(tx_result.is_ok());

        let token_metadata_account = svm.get_account(&metadata_pubkey).unwrap();
        let metadata = crate::helpers::token::Metadata::deserialize(
            &mut token_metadata_account.data.as_slice(),
        )
        .unwrap();

        assert!(metadata.name.starts_with(name));
        assert!(metadata.symbol.starts_with(symbol));
        assert!(metadata.uri.starts_with(uri));
    }
}
