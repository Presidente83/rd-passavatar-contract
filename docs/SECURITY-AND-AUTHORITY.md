# Avatar Club 100 Base-only: security and authority model

This document describes the reduced Base-only governance candidate. It is not
an audit and does not claim mainnet approval.

## Scope

The contract supports one operation only:

`ClaimBaseForPass { pass_token_id }`

The caller must be the current owner of the requested Founding Pass. The Pass
identifier must be `001` through `100`. Each Pass can claim once and each wallet
can hold at most one Base Avatar. Supply is capped at 100 and the contract does
not accept funds.

## Removed trust surfaces

This candidate contains no:

- marketplace or secondary-market settlement;
- paid mint, evolution or collection-offer path;
- signed fee quote or quote key;
- pause switch or operational admin execute message;
- Founder-sponsored claim path;
- transfer, send, approval or burn message;
- bank message, treasury payment or fee routing;
- migration entry point in the Wasm.

The interface and state model were reduced so these functions are not merely
disabled behind a web interface; they are absent from the candidate contract.

## Instantiation

Instantiation accepts only the Founding Pass contract address and the fixed
Avatar image SHA-256. The contract requires the Terra Classic `columbus-5`
chain identifier. No Founder wallet or private key is stored or required.

The actual instantiation transaction must be reviewed separately. In
particular, the Founding Pass contract address must be checked before signing.
The deployment must not assign a chain-level migration administrator.

## External dependency

At claim time, the contract queries the configured Founding Pass contract using
the `OwnerOf` query and compares the returned owner with the transaction sender.
The application cannot replace this check with client-side authorization.

## Governance boundary

The revised proposal must contain only the governance-authorized `MsgStoreCode`
for the exact Wasm produced from this source snapshot. It must not modify
upload permissions, transfer funds, instantiate the contract or approve future
modules.

## Verification gates

Before any deposit or signature request:

1. run the pinned build and unit tests;
2. run `cosmwasm-check`;
3. record the SHA-256 of the resulting Wasm;
4. reproduce the build independently on a second environment;
5. compare the two Wasm files byte-for-byte;
6. ask at least one independent reviewer to inspect the source and deployment
   message;
7. publish the commit, Wasm, checksum, logs and review findings.

Until all gates pass, this repository is only a review candidate.
