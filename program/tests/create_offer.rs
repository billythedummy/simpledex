#![cfg(feature = "test-bpf")]

use helpers::{create_token_account, create_two_mints, mint_tokens, program_test};
use simpledex::instructions::create_offer;
use solana_program::{hash::Hash, pubkey::Pubkey};
use solana_program_test::{tokio, BanksClient};
use solana_sdk::{signature::Keypair, signer::Signer, transaction::Transaction};

mod helpers;

struct CreateOfferEnv {
    client: BanksClient,
    recent_blockhash: Hash,
    payer: Keypair,
    owner: Keypair,
    token_a: Pubkey,
    token_a_account: Pubkey,
    token_b: Pubkey,
    token_b_account: Pubkey,
}

async fn setup(mint_tokens_a: u64, mint_tokens_b: u64) -> CreateOfferEnv {
    let (mut client, payer, recent_blockhash) = program_test().start().await;
    let owner = Keypair::new();
    let token_a_account = Keypair::new();
    let token_b_account = Keypair::new();
    let (token_a, token_b) =
        create_two_mints(&mut client, &payer, &recent_blockhash, &payer.pubkey()).await;
    create_token_account(
        &mut client,
        &payer,
        &recent_blockhash,
        &token_a_account,
        &token_a,
        &payer.pubkey(),
    )
    .await
    .unwrap();
    create_token_account(
        &mut client,
        &payer,
        &recent_blockhash,
        &token_b_account,
        &token_b,
        &payer.pubkey(),
    )
    .await
    .unwrap();
    mint_tokens(
        &mut client,
        &payer,
        &recent_blockhash,
        &token_a,
        &token_a_account.pubkey(),
        &payer,
        mint_tokens_a,
    )
    .await
    .unwrap();
    mint_tokens(
        &mut client,
        &payer,
        &recent_blockhash,
        &token_b,
        &token_b_account.pubkey(),
        &payer,
        mint_tokens_b,
    )
    .await
    .unwrap();
    CreateOfferEnv {
        client,
        recent_blockhash,
        payer,
        owner,
        token_a,
        token_a_account: token_a_account.pubkey(),
        token_b,
        token_b_account: token_b_account.pubkey(),
    }
}

#[tokio::test]
async fn success() {
    let mut env = setup(50, 0).await;
    let ix = create_offer(
        &env.payer.pubkey(),
        &env.owner.pubkey(),
        &env.token_a_account,
        &env.token_a_account,
        &env.token_b_account,
        &env.payer.pubkey(),
        &env.token_a,
        &env.token_b,
        0,
        45,
        2,
    )
    .unwrap();
    let mut transaction = Transaction::new_with_payer(&[ix], Some(&env.payer.pubkey()));
    transaction.sign(&[&env.payer, &env.owner], env.recent_blockhash);
    env.client.process_transaction(transaction).await.unwrap();
}
