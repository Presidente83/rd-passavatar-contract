# Avatar Club 100: authority and signed fee quote disclosure

This describes the executable code, not a promise that the software is audited.
It applies to this repository's `contracts/rd-passavatar/src/contract.rs`,
`msg.rs`, and `state.rs`. No mainnet instance is established by this document.

## Current product and trust boundary

The base collection consists of at most 100 avatars. Only the hard-coded Founder
`terra1nv0g2ap3fmll0m29f8fqr8q5pz4hejjrkjne0j` can instantiate on `columbus-5`
and sponsor `ClaimBaseForPass`. The contract queries the configured Founding Pass
contract for the current owner of Pass 001–100. Each Pass can be claimed once;
each recipient can hold one avatar. NFT000 cannot receive these claims.
Recipients attach no funds. The Founder signs and pays the transaction fee.
**These base claims do not call the quote verifier or require a quote service.**
The initial Founding Pass contract address is supplied at instantiation; reviewers
must inspect it before signing. A code upload alone does not set that address.

`BuyBase` always returns Unavailable. The secondary market starts disabled.
However, its code and paid evolutions are present in this Wasm: they are not
removed by a disabled web interface. Paid evolutions require a published offer,
an owned avatar, and a valid quote. The Founder can publish offers and enable
the secondary market without a further governance vote.

## Who holds the quote key?

The contract stores only a 32-byte Ed25519 **public** key supplied at instantiation.
It cannot identify the human custodian of the corresponding private key.

The RD Station application snapshot `7d16eb41ba733e0900a4f7c12fa37e151c6f374b`
(`lib/passavatar/quote-key.server.ts`, `quote.server.ts`, and the quote API route)
inspected for this review loads a separate
Ed25519 private key from a server environment variable named
`PASSAVATAR_GENESIS_QUOTE_PRIVATE_KEY_PKCS8`. This is not the Founder's wallet
key and it is not intended to be sent to the browser. Its trust boundary is the
RD Station backend operator and everyone with access to that deployment's
secrets. Source inspection does **not** verify who currently has access to the
production secret, whether it exists, its storage protections, or its backups.
Those operational details require confirmation by the operator before paid
features can be represented as production-ready.

At the inspected application snapshot, `/api/passavatar/quote` returns HTTP 410
(`paid_base_mint_disabled`); the retained quote service implementation prepares
`buy_base` quotes, which this contract refuses. It is not evidence of an active
production quote service for evolutions or resale. No private key was read or
published in preparing this disclosure.

## What a signed quote proves (and does not prove)

The signature covers the serialized FeeQuote: chain, contract, buyer, action,
token ID, revision, gross amount, estimated network fee, and expiry. The contract
checks the chain/contract/buyer, signature, and expiry (within 120 seconds).
It checks action, price, token and listing revision in the purchase handlers.
**The contract does not measure the actual gas fee paid by the transaction.**
An authentic signature proves approval by the quote key, not fee correctness.

For resale, attached funds are gross minus quoted network fee; the seller
receives gross minus that fee minus the fixed 5% commission. For evolutions,
the Founder receives gross minus quoted network fee. Thus an attacker holding
the quote key can sign inflated fee estimates for a buyer they control and
purchase at an unfairly reduced payment within the contract's arithmetic
bounds. This can harm sellers or the Founder. They cannot use this key alone
to sign a victim's wallet transaction, sponsor base claims, publish collections,
or pause/unpause. Wallet authorization and the listed-token checks still apply.
A short expiry does not stop an attacker generating fresh signatures. The test
`compromised_quote_signer_can_reduce_seller_payment` demonstrates this limit:
a 100,000 uluna listing can leave the seller only 1 uluna when the trusted
signer certifies an inflated 94,999 uluna fee. This is disclosed, not fixed by
the build correction.

## Loss, theft, and recovery

There is **no ExecuteMsg for quote-key rotation** in the current contract.

- Lost key without a working backup: quoted purchases become unavailable;
  free sponsored claims do not depend on that key. There is no in-place key
  recovery function.
- Compromised key: the Founder can immediately pause execution and disable
  secondary trading. Pausing also blocks base claims and evolutions; it is
  not a selective quote-only pause. Cancelling a listing remains possible.
- Changing the backend key alone will not change the on-chain public key.
  Resume only after a separately reviewed recovery/deployment strategy. Do not
  claim that a replacement deployment automatically transfers existing state.
- Loss of the Founder wallet prevents further sponsored claims and administrative
  actions. Compromise of that wallet gives the attacker those powers.

These are material limitations. This release is a review candidate, not an
assertion that paid features are safe to activate. Removing unused paid paths
or introducing a reviewed recovery design would change the source and Wasm
hash and must be disclosed as a different candidate.

## Administrative powers in this Wasm

| Action | Authorized party | Effect |
| --- | --- | --- |
| Instantiate | Hard-coded Founder only | Chooses Pass contract, quote public key, mint_price; image hash and cap are fixed |
| ClaimBaseForPass | Founder only | Mints to verified eligible current Pass owner, with uniqueness and cap checks |
| SetPaused | Founder only | Blocks ordinary execution; listing cancellation and admin toggles remain available |
| SetSecondaryEnabled | Founder only | Enables/disables listing creation and settlement |
| PublishCollection | Founder only | Publishes evolution metadata and offers; existing collection metadata cannot change, offers can be revised |
| Change quote key / Founder / Pass contract / image hash / mint_price | No execute method | Cannot be changed through this contract's execute interface |
| Transfer / Send / Approve / Burn | Refused | Transfers are available only through enabled internal resale settlement |

The `mint_price` field is retained but `BuyBase` is disabled. Collection cap is
100. Pause authorization is the hard-coded wallet, not dynamically the current
owner of NFT000.

## Chain-level migration administrator is a separate decision

This Wasm exports instantiate/execute/query and no migrate entry point. That is
**not** a guarantee against later migration if a chain-level admin is assigned:
a different destination Wasm can implement migration. The proposed deployment
policy is to instantiate **without a chain-level admin**. This must be checked
in the actual Instantiate transaction and resulting on-chain contract info;
it is not enforced by the current Rust instantiate function. Existing Marketplace
and Primary Sale contracts are separate and their admin rights are not changed.

## Review gates

Independent reproducibility and a second security reviewer remain required.
Unit tests and matching builds are not an audit. Resolve custody, recovery,
paid-path risks, and the exact instantiate configuration with reviewers before
mainnet use. The governance upload request must contain only MsgStoreCode for
the reviewed bytes; it must not add a wallet to upload permissions.
