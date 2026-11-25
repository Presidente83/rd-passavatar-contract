# RD Station — Avatar Club 100 Base-only

This package is the first, deliberately narrow on-chain module of RD Station.
It is a review candidate and is not audited or mainnet-approved.

## Collection rules

- Maximum supply: 100 tokens, `AV-000001`–`AV-000100`.
- Founding Pass IDs `001`–`100` each allow one free claim.
- The caller must be the current on-chain owner of the requested Pass.
- Each wallet can hold at most one Base Avatar.
- Claims are checked against the configured Founding Pass contract.
- No funds are accepted and no payments are distributed.

## Explicit exclusions

This package contains no marketplace, sale, evolution, signed quote, quote key,
pause control, Founder-sponsored claim, transfer, approval, burn, treasury
payment or operational admin message. Future functionality requires a separate
contract, review and governance decision.

## Configuration

Instantiation accepts only the Founding Pass contract address and the fixed
image SHA-256. The contract accepts only Terra Classic `columbus-5`.

## Release gate

Before any mainnet use, the exact Wasm must pass the repository build, unit
tests, `cosmwasm-check`, independent reproduction and a separate review of the
governance `MsgStoreCode` and instantiation messages.
