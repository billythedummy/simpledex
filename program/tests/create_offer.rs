#![cfg(feature = "test-bpf")]

mod helpers;

use std::assert_eq;

use helpers::{create_token_account, create_two_mints, mint_tokens, program_test};
use simpledex::{instructions::create_offer, pda::try_find_offer_pda, state::Offer};
use solana_program::{hash::Hash, program_pack::Pack, pubkey::Pubkey};
use solana_program_test::{tokio, BanksClient};
use solana_sdk::{signature::Keypair, signer::Signer, transaction::Transaction};

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
    let seed = 0;
    let offering = 45;
    let accept_at_least = 2;
    let mut env = setup(offering + 5, 0).await;
    let ix = create_offer(
        &env.payer.pubkey(),
        &env.owner.pubkey(),
        &env.token_a_account,
        &env.token_a_account,
        &env.token_b_account,
        &env.payer.pubkey(),
        &env.token_a,
        &env.token_b,
        seed,
        offering,
        accept_at_least,
    )
    .unwrap();
    let mut transaction = Transaction::new_with_payer(&[ix], Some(&env.payer.pubkey()));
    transaction.sign(&[&env.payer, &env.owner], env.recent_blockhash);
    env.client.process_transaction(transaction).await.unwrap();
    let (offer_addr, bump) =
        try_find_offer_pda(&env.owner.pubkey(), &env.token_a, &env.token_b, seed).unwrap();
    let created_offer = env.client.get_account(offer_addr).await.unwrap().unwrap();
    let offer = Offer::unpack_from_slice(created_offer.data.as_slice()).unwrap();
    assert_eq!(offer.offering, offering);
    assert_eq!(offer.accept_at_least, accept_at_least);
    assert_eq!(offer.seed, seed);
    assert_eq!(offer.bump, bump);
    assert_eq!(offer.owner, env.owner.pubkey());
    assert_eq!(offer.offer_mint, env.token_a);
    assert_eq!(offer.accept_mint, env.token_b);
    assert_eq!(offer.refund_to, env.token_a_account);
    assert_eq!(offer.credit_to, env.token_b_account);
    assert_eq!(offer.refund_rent_to, env.payer.pubkey());
}

// TODO: more tests
