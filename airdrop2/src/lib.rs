#[cfg(test)]
mod tests {
    use solana_client::rpc_client::RpcClient;
    use solana_system_interface::{program as system_program, instruction::transfer};
    use solana_sdk::{
        instruction::{Instruction, AccountMeta},
        message::Message,
        pubkey::Pubkey,
        signature::{Keypair, Signer, read_keypair_file},
        transaction::Transaction,
    };
    use std::str::FromStr;
    use std::io::{self, BufRead};
    use bs58;

    // Turbin3 Devnet RPC endpoint
    const RPC_URL: &str = "https://turbine-solanad-4cde.devnet.rpcpool.com/9a9da9cf-6db1-47dc-839a-55aca5c9c80a";

    // Step 1: Generate a new keypair
    #[test]
    fn keygen() {
        let kp = Keypair::new();
        println!("Wallet: {}", kp.pubkey());
        println!("{:?}", kp.to_bytes());
    }

    // Step 2: Base58 → wallet array
    #[test]
    fn base58_to_wallet() {
        let stdin = io::stdin();
        let base58 = stdin.lock().lines().next().unwrap().unwrap();
        let wallet = bs58::decode(base58).into_vec().unwrap();
        println!("{:?}", wallet);
    }

    // Step 3: Wallet array → Base58
    #[test]
    fn wallet_to_base58() {
        let stdin = io::stdin();
        let wallet = stdin
            .lock()
            .lines()
            .next()
            .unwrap()
            .unwrap()
            .trim_start_matches('[')
            .trim_end_matches(']')
            .split(',')
            .map(|s| s.trim().parse::<u8>().unwrap())
            .collect::<Vec<u8>>();
        let base58 = bs58::encode(wallet).into_string();
        println!("{:?}", base58);
    }

    // Step 4: Claim airdrop
    #[test]
    fn claim_airdrop() {
        let keypair = read_keypair_file("dev-wallet.json").unwrap();
        let client = RpcClient::new(RPC_URL);

        match client.request_airdrop(&keypair.pubkey(), 2_000_000_000) {
            Ok(sig) => println!("https://explorer.solana.com/tx/{}?cluster=devnet", sig),
            Err(err) => println!("Airdrop failed: {}", err),
        }
    }

    // Step 5: Transfer 0.1 SOL to Turbin3 wallet
    #[test]
    fn transfer_sol() {
        let keypair = read_keypair_file("dev-wallet.json").unwrap();
        let to_pubkey = Pubkey::from_str("3AM4N2hitJxtkZbN3newYE62Yq2p33DzM7VUsrnXmF1a").unwrap();

        let rpc_client = RpcClient::new(RPC_URL);
        let blockhash = rpc_client.get_latest_blockhash().unwrap();

        let tx = Transaction::new_signed_with_payer(
            &[transfer(&keypair.pubkey(), &to_pubkey, 100_000_000)],
            Some(&keypair.pubkey()),
            &[&keypair],
            blockhash,
        );

        let sig = rpc_client.send_and_confirm_transaction(&tx).unwrap();
        println!("https://explorer.solana.com/tx/{}/?cluster=devnet", sig);
    }

    // Step 6: Empty wallet to Turbin3 wallet
    #[test]
    fn empty_wallet() {
        let keypair = read_keypair_file("dev-wallet.json").unwrap();
        let to_pubkey = Pubkey::from_str("3AM4N2hitJxtkZbN3newYE62Yq2p33DzM7VUsrnXmF1a").unwrap();

        let rpc_client = RpcClient::new(RPC_URL);
        let blockhash = rpc_client.get_latest_blockhash().unwrap();

        let balance = rpc_client.get_balance(&keypair.pubkey()).unwrap();
        let message = Message::new_with_blockhash(
            &[transfer(&keypair.pubkey(), &to_pubkey, balance)],
            Some(&keypair.pubkey()),
            &blockhash,
        );

        let fee = rpc_client.get_fee_for_message(&message).unwrap();

        let tx = Transaction::new_signed_with_payer(
            &[transfer(&keypair.pubkey(), &to_pubkey, balance - fee)],
            Some(&keypair.pubkey()),
            &[&keypair],
            blockhash,
        );

        let sig = rpc_client.send_and_confirm_transaction(&tx).unwrap();
        println!("https://explorer.solana.com/tx/{}/?cluster=devnet", sig);
    }

    // Step 7-8: Submit proof (submit_rs)
    #[test]
    fn submit_proof() {
        let rpc_client = RpcClient::new(RPC_URL);
        let signer = read_keypair_file("dev-wallet.json").unwrap();

        let mint = Keypair::new();
        let turbin3_prereq_program = Pubkey::from_str("TRBZyQHB3m68FGeVsqTK39Wm4xejadjVhP5MAZaKWDM").unwrap();
        let collection = Pubkey::from_str("5ebsp5RChCGK7ssRZMVMufgVZhd2kFbNaotcZ5UvytN2").unwrap();
        let mpl_core_program = Pubkey::from_str("CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d").unwrap();
        let system_program = system_program::id();

        let signer_pubkey = signer.pubkey();
        let seeds = &[b"prereqs", signer_pubkey.as_ref()];
        let (prereq_pda, _bump) = Pubkey::find_program_address(seeds, &turbin3_prereq_program);

        let data = vec![77, 124, 82, 163, 21, 133, 181, 206]; // submit_rs discriminator

        let accounts = vec![
            AccountMeta::new(signer.pubkey(), true),
            AccountMeta::new(prereq_pda, false),
            AccountMeta::new(mint.pubkey(), true),
            AccountMeta::new(collection, false),
            AccountMeta::new_readonly(signer.pubkey(), false),
            AccountMeta::new_readonly(mpl_core_program, false),
            AccountMeta::new_readonly(system_program, false),
        ];

        let instruction = Instruction {
            program_id: turbin3_prereq_program,
            accounts,
            data,
        };

        let blockhash = rpc_client.get_latest_blockhash().unwrap();

        let tx = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&signer.pubkey()),
            &[&signer, &mint],
            blockhash,
        );

        let sig = rpc_client.send_and_confirm_transaction(&tx).unwrap();
        println!("https://explorer.solana.com/tx/{}/?cluster=devnet", sig);
    }
}
