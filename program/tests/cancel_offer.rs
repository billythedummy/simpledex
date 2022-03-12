#![cfg(feature = "test-bpf")]

mod helpers;

use helpers::{
    create_and_get_offer, create_token_account, create_two_mints, mint_tokens, program_test,
    transfer,
};
use simpledex::instructions::cancel_offer;
use solana_program::{hash::Hash, pubkey::Pubkey};
use solana_program_test::{tokio, BanksClient};
use solana_sdk::{signature::Keypair, signer::Signer, transaction::Transaction};
use spl_associated_token_account::get_associated_token_address;

struct CancelOfferEnv {
    client: BanksClient,
    recent_blockhash: Hash,
    payer: Keypair,
    owner: Keypair,
    token_a: Pubkey,
    token_a_account: Pubkey,
    token_b: Pubkey,
    token_b_account: Pubkey,
}

async fn setup(mint_tokens_a: u64, mint_tokens_b: u64) -> CancelOfferEnv {
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
        &owner.pubkey(),
    )
    .await
    .unwrap();
    create_token_account(
        &mut client,
        &payer,
        &recent_blockhash,
        &token_b_account,
        &token_b,
        &owner.pubkey(),
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
    // give some lamports to owner to allow him to send the cancel tx
    // else test will just hang silently
    transfer(
        &mut client,
        &payer,
        &recent_blockhash,
        &owner.pubkey(),
        1_000_000_000,
    )
    .await;
    CancelOfferEnv {
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
    let seed = 0;
    let offering = 45;
    let accept_at_least = 2;
    let mut env = setup(offering + 5, 0).await;

    let (offer_addr, offer) = create_and_get_offer(
        &mut env.client,
        &env.recent_blockhash,
        &env.payer,
        &env.owner,
        &env.token_a_account,
        &env.token_b_account,
        &env.token_a,
        &env.token_b,
        seed,
        offering,
        accept_at_least,
    )
    .await;

    let payer_lamports_before_cancel = env.client.get_balance(env.payer.pubkey()).await.unwrap();

    let cancel_ix = cancel_offer(&offer).unwrap();
    let mut cancel_tx = Transaction::new_with_payer(&[cancel_ix], Some(&env.owner.pubkey()));
    cancel_tx.sign(&[&env.owner], env.recent_blockhash);
    env.client.process_transaction(cancel_tx).await.unwrap();
    // check offer and holding no longer exists
    assert!(env.client.get_account(offer_addr).await.unwrap().is_none());
    let holding_addr = get_associated_token_address(&offer_addr, &offer.offer_mint);
    assert!(env
        .client
        .get_account(holding_addr)
        .await
        .unwrap()
        .is_none());

    let payer_lamports_after_cancel = env.client.get_balance(env.payer.pubkey()).await.unwrap();
    assert!(payer_lamports_after_cancel > payer_lamports_before_cancel);
}

// TODO: more tests
