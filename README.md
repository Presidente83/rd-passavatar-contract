# RD Station — Avatar Club 100 CosmWasm contract

Public contract source for independent review. This repository does not include
the RD Station application. **Review candidate; not audited or mainnet approved.**

## Reproduce the candidate

Use Linux x86_64, Git, Rustup, a C compiler and standard build tools. From the
repository root:

```bash
bash scripts/passavatar/build.sh
cargo +1.85.1 install cosmwasm-check --version 1.5.11 --locked
cosmwasm-check artifacts/passavatar/rd_passavatar.wasm
(cd artifacts/passavatar && sha256sum --check ../../release/checksums.txt)
```

Rust 1.85.1, Cargo.lock and compilation flags are pinned. The script normalizes
source, Cargo registry and Rust sysroot paths, including std built with
`-Z build-std=std,panic_abort`. This is the **path-normalization alternative**,
not a claim to use the standard cosmwasm/optimizer image. Do not compare an
optimizer-produced binary against this native recipe's hash.

The previous recipe embedded local Rust/Cargo paths and compared two builds
sharing the same installation. Its hash
`6e2075f95c52b69b116e681d2a8b19778081f2b9de67819bb4063816ce120a84`
is **not the expected hash for this corrected recipe**. The new measured
checksum is recorded in `release/checksums.txt` when verified.

Outputs: `artifacts/passavatar/rd_passavatar.wasm`, `checksums.txt`, and
`build.log`. CI builds on separate Ubuntu runners with different checkout,
Cargo and Rustup paths, then compares the bytes. The release workflow attaches
the exact Wasm, checksums and both build logs to a tagged prerelease only after
all jobs pass. Reviewers should reproduce the release tag on their own machine.
Automated CI agreement does not replace that independent verification.

## Contract and authority

Read [Security and authority](docs/SECURITY-AND-AUTHORITY.md) before voting.
Base claims are free for eligible Pass 001–100 holders and sponsored by NFT000.
The quote signer is not used by those claims. Paid execution paths remain in the
source, have a non-rotatable quote key, and require further security review.
No independent third-party audit is supplied.

## Governance deployment model

The intended new proposal stores **only the reviewed Wasm** using governance
MsgStoreCode, without adding a developer wallet to code-upload permissions.
The payload must be checked against the live Terra Classic message schema,
limits and permissions and simulated before any deposit/signature request.
Instantiation is separate: Founder sender, correct Pass address, mint_price 0,
approved image, reviewed quote key and **no chain-level migration admin**.
No deployment is performed by this repository's build/release workflow.

The prior on-chain proposal #12238 is not changed by source or forum edits.
See [review notes](release/REVIEW-NOTES.md).
