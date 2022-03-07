# simpledex

A simple dex design that makes full use of solana's massively parallel runtime

## Architecture

- There is no market
- Just traders creating offers of one token type for another at a specified limit rate
- Off-chain matchers are incentivized to match offers by the fees offered by the traders
    - To incentivize matchers to give traders the best rate, 50% of excess tokens from an offer matched at a rate better than the limit price is given to the matcher as a bonus.
- Maker-taker relationship is determined by slot at which the offer was made. The taker is simply the offer with the later slot
    - To incentivize liquidity provision, only the taker pays fees
    - If both offers were made in the same slot, the matcher gets half of the taker's fees and half of the maker's fees
    - Technically a taker can avoid fees by just appending the `Match` instruction to his transaction and set himself as the matcher, and that's ok.
- No settling of funds, tokens are credited directly to the order's specified token account
- Global hardcoded constant fee parameter in terms of bps

#### Benefits of this design:
- Parallel processing of orders. Only the accounts of those involved in the order - maker, taker, and matcher, are write-locked.
- Simple, complexity is moved off-chain.
- Decentralized matching of orders, anyone can run a matching algorithm and earn fees, or implement the matching in a webapp.

#### Disadvantanges:
- Complexity moved off-chain means more work off-chain. For example, to visualize an orderbook, you would need to `getProgramAccounts()` and filter by mints, and then aggregate those together.
- Live updates are a little more complicated. Instead of simply opening a websocket to listen for changes to a central orderbook account, you need to listen to program logs and update a locally cached state accordingly.
- Intense order-matching competition can lead to flooding of chain with failed match transactions(?)

## Accounts

### Offer

This is the only program account type.

An `Offer` account is located at PDA `[self.owner, self.offer_mint, self.accept_mint, self.seed, [self.bump]]`. An `owner` can have at most 65536 active offers at any instant for a (`offer_mint`, `accept_mint`) pair

| field | type | description |
| -- | -- | -- |
| `slot` | `Slot` | slot at which this offer was made |
| `offering` | `u64` | number of `offer_mint` tokens put up for offer in exchange for at least `accept_at_least` of `accept_mint` tokens, not including the taker fee. <br />Decreases as this offer is filled |
| `accept_at_least` | `u64` | min number of `accept_mint` tokens accepted in exchange for `offering` amount of `offer_mint` tokens. <br />Decreases as this offer is filled |
| `seed` | `u16` | seed for this offer account, unique for the given (owner, offer_mint, accept_mint) |
| `bump` | `u8` | bump seed for this offer account |
| `owner` | `Pubkey` | owner pubkey that created, and is authorized to cancel, this offer account |
| `offer_mint` | `Pubkey` | mint of the token offered |
| `accept_mint` | `Pubkey` | mint of the token accepted |
| `refund_to` | `Pubkey` | `offer_mint` token account to accept refunds of unspent taker fees and any excess balance in the holding account |
| `credit_to` | `Pubkey` | `accept_mint` token account to accept transfers of successful trades |
| `refund_rent_to` | `Pubkey` | pubkey to refund rent lamports of this account and the holding token account to |

### Holding account

This token account holds the `offer_mint` tokens offered by an `Offer`. 

A holding token account for an `Offer` is the `Offer`'s ATA.

#### Invariants:
- at any time, it should contain at least `(10_000 + taker_fee_bps) * offer.offering / 10_000` number of tokens.

## Instructions

Just 3 instructions.

### CreateOffer

Creates a new `Offer`.

#### Args:
- `bump`: `u8`
- `seed`: `u16`
- `offering`: `u64`
- `accept_at_least`: `u64`

#### Accounts:
- [w, s] payer. Pubkey paying for the new accounts' rent
- [s] owner
- [w] pay_from. Token account to transfer offered tokens from, should be controlled by owner.
- [w] offer
    - check PDA matches using find_program_address()
- [w] holding
    - check this is offer's ATA for offer_mint
- [] refund_to
    - check valid offer_mint token account
    - check not frozen
    - check not equal to `holding`
- [] credit_to
    - check valid accept_mint token account
    - check not frozen
- [] refund_rent_to
    - check not equal to `holding`
    - check not equal to `offer`
- [] offer_mint
    - check valid mint
- [] accept_mint
    - check valid mint
- [] token_program
    - check program_id
- [] associated_token_program
    - check program_id
- [] system_program
    - check program_id
#### Procedure:
- initialize rent-free offer account with args
- initialize rent-free holding account
- transfer `(10_000 + taker_fee_bps) * offer.offering / 10_000` tokens to holding account

### CancelOffer

Cancels an existing `Offer` and refunds the rent for the offer and holding accounts.

#### Accounts:
- [s] owner
- [w] offer
    - check owner
    - check PDA matches using create_program_address()
- [w] holding
    - check this is offer's ATA for offer_mint
- [w] refund_to
    - check this is offer's refund_to
- [w] refund_rent_to
    - check this is offer's refund_rent_to
- [] token_program
    - check program_id

#### Procedure:
- transfer remaining balance in holding account to refund_to
- close holding account, refund rent to refund_rent_to
- close offer account, refund rent to refund_rent_to

### Match

Permissionless instruction to match 2 `Offer`s.

#### Invariants:
- At least one of the 2 matched `Offer`s must be filled entirely and closed.

#### Accounts:
- [w] offering_a. The `Offer` account that is offering token A in exchange for token B
    - check PDA matches using create_program_address()
    - check `accept_mint` is mint_b
    - check `offer_mint` is mint_a
- [w] holding_a. token A holding account for offering_a
    - check is the mint_a ATA of offering_a
- [w] offering_b. The `Offer` account that is offering token B in exchange for token A
    - check PDA matches using create_program_address()
    - check `accept_mint` is mint_a
    - check `offer_mint` is mint_b
- [w] holding_b. token B holding account for offering_b
    - check is the mint_b ATA of offering_b
- [w] credit_to_a
    - check matches `offering_a.credit_to`
- [w] refund_to_a
    - check matches `offering_a.refund_to`
- [w] refund_rent_to_a
    - check matches `offering_a.refund_rent_to`
- [w] credit_to_b
    - check matches `offering_b.credit_to`
- [w] refund_to_b
    - check matches `offering_b.refund_to`
- [w] refund_rent_to_b
    - check matches `offering_b.refund_rent_to`
- [w] matcher_a. the matcher's token A account to credit fees to.
- [w] matcher_b. the matcher's token B account to credit fees to.
- [] mint_a
- [] mint_b
- [] token_program
    - check program_id
- [] associated_token_program
    - check program_id
#### Procedure:
- check that the limit prices for both offers are met by a swap.
    - `offering_a.offering / offering_a.accept_at_least >= offering_b.accept_at_least / offering_b.offering`
- determine the amount to swap for token A and token B. The swap should close at least one of the offers.
    - if `offering_a.offering >= offering_b.accept_at_least && offering_b.offering >= offering_a.accept_at_least`
        then `amt_a = offering_a.offering, amt_b = offering_b.offering` and both offers are closed.
    - else if `offering_a.offering >= offering_b.accept_at_least && offering_b.offering < offering_a.accept_at_least`,
        then `amt_a = offering_b.accept_at_least, amt_b = offering_b.offering` and only offering_b is closed. 
    - else if `offering_a.offering < offering_b.accept_at_least && offering_b.offering >= offering_a.accept_at_least`,
        then `amt_a = offering_a.offering, amt_b = offering_a.accept_at_least` and only offering_a is closed.
    - else price doesnt match 
    - Note that in the case of only one order filling, the one who gets his order completely filled gets the worse deal
- determine maker-taker relationship and matcher fees and bonuses.
    - token a excess = max amt of token A offering_a is willing to pay for `amt_b` - `amt_a`
    - token b excess = max amt of token B offering_b is willing to pay for `amt_a` - `amt_b`
- perform the swap: transfer `amt_a` from `holding_a` to `credit_to_b` and `amt_b` from `holding_b` to `credit_to_a`
- pay the matcher fees and bonuses: transfer tokens from `holding_a` to `matcher_a` or from `holding_b` to `matcher_b`, or both
- update the 2 offer accounts by decrementing their `offering` by `amt_a` and `amt_b` respectively and `accept_at_least` fields by the amount that maintains the same price.
- close the offer and holding accounts for filled offers (`offering` == 0 || `accept_at_least` == 0) and refund rent to their respective `refund_rent_to`.

## Logs

To provide traders with real-time market info, each successful instruction execution should emit a log that can be subscribed to in order to update a locally cached market/orderbook state.

Logs are human readable csv, token amounts are in decimals.

### CreateOffer

#### Format:

```
NEW:<OFFER-PUBKEY-BASE58>,<OFFERING-TOKEN-BASE58>,<OFFER-AMOUNT>,<ACCEPT-TOKEN-BASE58>,<ACCEPT-AT-LEAST>
```

#### Example:

Someone just created an offer exchanging 1 wSOL for at least 100 USDC at 4Rf9mGD7FeYknun5JczX5nGLTfQuS1GRjNVfkEMKE92b

```
Program log: NEW:4Rf9mGD7FeYknun5JczX5nGLTfQuS1GRjNVfkEMKE92b,So11111111111111111111111111111111111111112,1,EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v,100
```

### CancelOffer

#### Format:

```
CANCEL:<OFFER-PUBKEY-BASE58>,<OFFERING-TOKEN-BASE58>,<OFFER-AMOUNT>,<ACCEPT-TOKEN-BASE58>,<ACCEPT-AT-LEAST>
```

#### Example:

Someone just canceled an offer exchanging 1 wSOL for at least 100 USDC at 4Rf9mGD7FeYknun5JczX5nGLTfQuS1GRjNVfkEMKE92b

```
Program log: CANCEL:4Rf9mGD7FeYknun5JczX5nGLTfQuS1GRjNVfkEMKE92b,So11111111111111111111111111111111111111112,1,EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v,100
```

### Match

**Format:**

```
TRADE:<TOKEN-A-BASE58>,<TOKEN-A-AMOUNT>,<TOKEN-B-BASE58>,<TOKEN-B-AMOUNT>
OFFERS:<OFFERING-A-BASE58>,<OFFERING-A-NEW-OFFERING>,<OFFERING-A-NEW-ACCEPT-AT-LEAST>,<OFFERING-B-BASE58>,<OFFERING-B-NEW-OFFERING>,<OFFERING-B-NEW-ACCEPT-AT-LEAST>
```

**Example:**

100 USDC was just exchanged for 1 wSOL between offering_a 4Rf9mGD7FeYknun5JczX5nGLTfQuS1GRjNVfkEMKE92b and offering_b 9oKrJ9iiEnCC7bewcRFbcdo4LKL2PhUEqcu8gH2eDbVM

```
Program log: TRADE:So11111111111111111111111111111111111111112,1,EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v,100
Program log: OFFERS:4Rf9mGD7FeYknun5JczX5nGLTfQuS1GRjNVfkEMKE92b,0,0,9oKrJ9iiEnCC7bewcRFbcdo4LKL2PhUEqcu8gH2eDbVM,10,0.1
```

## QnA

### Frontrunning

Matchers can front run by placing their own order and claim the spread if they see that the highest bid is higher than the lowest ask.
- have i reinvented the mempool dark forest?
    - At least you can't Just-in-time liquidity attack since to do it the attacker would need to see the order and to see the order would mean that the attacker will be the taker and not earn any fees.
- 50% excess sharing maybe discourages this?
- is this just a form of acceptable arbitrage?

~~Traders can front run other traders by offering a slightly higher `taker_fee_bps`.~~ removed fee market, fees are global constants.

### Strict ordering - how do we ensure orders at the same/better price that are placed earlier get filled first?

We don't. The fastest matchers (which can be the trader himself) win and determine the trade order. That kinda sucks if you can't continuously attempt to match your own open offers.

Other options:
- having fees scale with time passed. May introduce perverse incentive of matchers waiting for max amount of time before matching.

### Revival attacks on closed offer accounts or holding accounts by griefing matchers

Options:
- ignore. Takes around 2000 SOL to completely block a pubkey from a market by revival attacking all possible 65536 offer and holding accounts for a token pair. Just have to wait for v1.9.0 to drop.
- check that only other match instructions can follow a match instruction in a transaction.
- handle zeroed out accounts. Only works for offer accounts because spl token initializeAccount is permissionless.
