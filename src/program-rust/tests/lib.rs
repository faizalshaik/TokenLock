use tokenlock::{entrypoint::process_instruction, types::TokenLockAccount};

use borsh::BorshDeserialize;
use solana_program_test::*;
use solana_sdk::{
    account::Account,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::Signer,
    transaction::Transaction,
};
use std::mem;


#[tokio::test]
async fn test_helloworld() {
    let program_id = Pubkey::new_unique();
    let greeted_pubkey = Pubkey::new_unique();

    let mut program_test = ProgramTest::new(
        "tokenlock", // Run the BPF version with `cargo test-bpf`
        program_id,
        processor!(process_instruction), // Run the native version with `cargo test`
    );
    program_test.add_account(
        greeted_pubkey,
        Account {
            lamports: 5,
            //data: vec![0_u8; mem::size_of::<u32>()],
            data: vec![0_u8; 20],
            owner: program_id,
            ..Account::default()
        },
    );
    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    // Verify account has zero greetings
    let greeted_account = banks_client
        .get_account(greeted_pubkey)
        .await
        .expect("get_account")
        .expect("greeted_account not found");
    assert_eq!(
        TokenLockAccount::try_from_slice(&greeted_account.data)
            .unwrap()
            .counter,
        0
        // GreetingAccount::try_from_slice(&greeted_account.data)
        //     .unwrap()
        //     .counter,
        // 0
    );

    // Greet once
    let mut transaction = Transaction::new_with_payer(
        &[Instruction::new_with_bincode(
            program_id,
            &[0_u8], // ignored but makes the instruction unique in the slot
            vec![AccountMeta::new(greeted_pubkey, false)],
        )],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    // Verify account has one greeting
    let greeted_account = banks_client
        .get_account(greeted_pubkey)
        .await
        .expect("get_account")
        .expect("greeted_account not found");
    assert_eq!(        
        TokenLockAccount::try_from_slice(&greeted_account.data)
            .unwrap()
            .counter,
        1

        // GreetingAccount::try_from_slice(&greeted_account.data)
        //     .unwrap()
        //     .counter,
        // 1
    );

    // Greet again
    let mut transaction = Transaction::new_with_payer(
        &[Instruction::new_with_bincode(
            program_id,
            &[0_u8], // ignored but makes the instruction unique in the slot
            vec![AccountMeta::new(greeted_pubkey, false)],
        )],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    // Verify account has two greetings
    let greeted_account = banks_client
        .get_account(greeted_pubkey)
        .await
        .expect("get_account")
        .expect("greeted_account not found");
    assert_eq!(
        TokenLockAccount::try_from_slice(&greeted_account.data)
            .unwrap()
            .counter,
        2
        // GreetingAccount::try_from_slice(&greeted_account.data)
        //     .unwrap()
        //     .counter,
        // 2
    );
}
