use simpledex::{instructions::create_offer, pda::try_find_offer_pda, state::Offer};
use solana_program::{hash::Hash, program_pack::Pack, pubkey::Pubkey};
use solana_program_test::BanksClient;
use solana_sdk::{signature::Keypair, signer::Signer, transaction::Transaction};

pub async fn create_and_get_offer(
    client: &mut BanksClient,
    recent_blockhash: &Hash,
    payer: &Keypair,
    owner: &Keypair,
    token_a_account: &Pubkey,
    token_b_account: &Pubkey,
    token_a: &Pubkey,
    token_b: &Pubkey,
    seed: u16,
    offering: u64,
    accept_at_least: u64,
) -> (Pubkey, Offer) {
    let create_ix = create_offer(
        &payer.pubkey(),
        &owner.pubkey(),
        token_a_account,
        token_a_account,
        token_b_account,
        &payer.pubkey(),
        token_a,
        token_b,
        seed,
        offering,
        accept_at_least,
    )
    .unwrap();
    let mut create_tx = Transaction::new_with_payer(&[create_ix], Some(&payer.pubkey()));
    create_tx.sign(&[payer, owner], *recent_blockhash);
    client.process_transaction(create_tx).await.unwrap();
    let (offer_addr, _bump) = try_find_offer_pda(&owner.pubkey(), token_a, token_b, seed).unwrap();
    let created_offer = client.get_account(offer_addr).await.unwrap().unwrap();
    (
        offer_addr,
        Offer::unpack_from_slice(created_offer.data.as_slice()).unwrap(),
    )
}
