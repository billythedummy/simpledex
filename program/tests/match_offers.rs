#![cfg(feature = "test-bpf")]

mod helpers;

use std::assert_eq;

use helpers::{
    create_and_get_offer, create_token_account, create_two_mints, get_token_acc, mint_tokens,
    program_test, transfer,
};
use simpledex::instructions::match_offers;
use solana_program::{hash::Hash, pubkey::Pubkey};
use solana_program_test::{tokio, BanksClient};
use solana_sdk::{signature::Keypair, signer::Signer, transaction::Transaction};
use spl_associated_token_account::get_associated_token_address;

struct MatchOffersEnv {
    client: BanksClient,
    recent_blockhash: Hash,
    token_a: Pubkey,
    token_b: Pubkey,
    payer: Keypair,

    owner_a: Keypair,
    owner_a_token_a_account: Pubkey,
    owner_a_token_b_account: Pubkey,

    owner_b: Keypair,
    owner_b_token_a_account: Pubkey,
    owner_b_token_b_account: Pubkey,

    matcher: Keypair,
    matcher_token_a_account: Pubkey,
    matcher_token_b_account: Pubkey,
}

async fn setup_accounts(
    client: &mut BanksClient,
    payer: &Keypair,
    owner: &Pubkey,
    recent_blockhash: &Hash,
    token_a: &Pubkey,
    token_b: &Pubkey,
    mint_tokens_a: u64,
    mint_tokens_b: u64,
) -> (Pubkey, Pubkey) {
    let token_a_account = Keypair::new();
    let token_b_account = Keypair::new();
    create_token_account(
        client,
        payer,
        recent_blockhash,
        &token_a_account,
        token_a,
        owner,
    )
    .await
    .unwrap();
    create_token_account(
        client,
        payer,
        recent_blockhash,
        &token_b_account,
        token_b,
        owner,
    )
    .await
    .unwrap();
    mint_tokens(
        client,
        payer,
        recent_blockhash,
        token_a,
        &token_a_account.pubkey(),
        payer,
        mint_tokens_a,
    )
    .await
    .unwrap();
    mint_tokens(
        client,
        payer,
        recent_blockhash,
        token_b,
        &token_b_account.pubkey(),
        payer,
        mint_tokens_b,
    )
    .await
    .unwrap();
    // give some lamports to owner to allow him to send the cancel tx
    // else test will just hang silently
    let lamports = 1_000_000_000;
    transfer(client, payer, recent_blockhash, owner, lamports).await;
    (token_a_account.pubkey(), token_b_account.pubkey())
}

async fn setup(mint_tokens_a: u64, mint_tokens_b: u64) -> MatchOffersEnv {
    let (mut client, payer, recent_blockhash) = program_test().start().await;
    let (token_a, token_b) =
        create_two_mints(&mut client, &payer, &recent_blockhash, &payer.pubkey()).await;

    let owner_a = Keypair::new();
    let owner_b = Keypair::new();
    let matcher = Keypair::new();

    let (owner_a_token_a_account, owner_a_token_b_account) = setup_accounts(
        &mut client,
        &payer,
        &owner_a.pubkey(),
        &recent_blockhash,
        &token_a,
        &token_b,
        mint_tokens_a,
        0,
    )
    .await;

    let (owner_b_token_a_account, owner_b_token_b_account) = setup_accounts(
        &mut client,
        &payer,
        &owner_b.pubkey(),
        &recent_blockhash,
        &token_a,
        &token_b,
        0,
        mint_tokens_b,
    )
    .await;

    let (matcher_token_a_account, matcher_token_b_account) = setup_accounts(
        &mut client,
        &payer,
        &matcher.pubkey(),
        &recent_blockhash,
        &token_a,
        &token_b,
        0,
        0,
    )
    .await;

    MatchOffersEnv {
        client,
        recent_blockhash,
        token_a,
        token_b,
        payer,

        owner_a,
        owner_a_token_a_account,
        owner_a_token_b_account,

        owner_b,
        owner_b_token_a_account,
        owner_b_token_b_account,

        matcher,
        matcher_token_a_account,
        matcher_token_b_account,
    }
}

#[tokio::test]
async fn success_match_exact() {
    let mint_a_tokens = 1_000_000;
    let mint_b_tokens = 123_456;
    let offer_a_offering = 900_000;
    let offer_a_accept_at_least = 99_000;
    let offer_b_offering = offer_a_accept_at_least;
    let offer_b_accept_at_least = offer_a_offering;
    let seed_a = 0;
    let seed_b = 0;

    let mut env = setup(mint_a_tokens, mint_b_tokens).await;

    let (offering_a_addr, offering_a) = create_and_get_offer(
        &mut env.client,
        &env.recent_blockhash,
        &env.payer,
        &env.owner_a,
        &env.owner_a_token_a_account,
        &env.owner_a_token_b_account,
        &env.token_a,
        &env.token_b,
        seed_a,
        offer_a_offering,
        offer_a_accept_at_least,
    )
    .await;

    let (offering_b_addr, offering_b) = create_and_get_offer(
        &mut env.client,
        &env.recent_blockhash,
        &env.payer, // &env.payer - possible runtime bug. spl-token::close CPI fails with imbalanced instruction if both offers have the same refund_rent_to
        &env.owner_b,
        &env.owner_b_token_b_account,
        &env.owner_b_token_a_account,
        &env.token_b,
        &env.token_a,
        seed_b,
        offer_b_offering,
        offer_b_accept_at_least,
    )
    .await;

    let match_ix = match_offers(
        &offering_a,
        &offering_b,
        &env.matcher_token_a_account,
        &env.matcher_token_b_account,
    )
    .unwrap();
    let mut match_tx = Transaction::new_with_payer(&[match_ix], Some(&env.matcher.pubkey()));
    match_tx.sign(&[&env.matcher], env.recent_blockhash);
    env.client.process_transaction(match_tx).await.unwrap();

    // check both offers no longer exists
    assert!(env
        .client
        .get_account(offering_a_addr)
        .await
        .unwrap()
        .is_none());
    assert!(env
        .client
        .get_account(offering_b_addr)
        .await
        .unwrap()
        .is_none());
    let holding_a_addr = get_associated_token_address(&offering_a_addr, &offering_a.offer_mint);
    assert!(env
        .client
        .get_account(holding_a_addr)
        .await
        .unwrap()
        .is_none());
    let holding_b_addr = get_associated_token_address(&offering_b_addr, &offering_b.offer_mint);
    assert!(env
        .client
        .get_account(holding_b_addr)
        .await
        .unwrap()
        .is_none());

    // check post balances
    let owner_a_token_b_acc = get_token_acc(&mut env.client, &env.owner_a_token_b_account).await;
    assert_eq!(owner_a_token_b_acc.amount, offer_a_accept_at_least);
    let owner_b_token_a_acc = get_token_acc(&mut env.client, &env.owner_b_token_a_account).await;
    assert_eq!(owner_b_token_a_acc.amount, offer_b_accept_at_least);
}

// TODO: more tests
