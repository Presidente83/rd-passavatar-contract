# RD Station — Avatar Club 100 Base-only CosmWasm contract

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

The checksum in `release/checksums.txt` must only be added after this exact
source snapshot has been built and independently reproduced. No previous Wasm
checksum applies to this Base-only source.

Outputs: `artifacts/passavatar/rd_passavatar.wasm`, `checksums.txt`, and
`build.log`. CI builds on separate Ubuntu runners with different checkout,
Cargo and Rustup paths, then compares the bytes. The release workflow attaches
the exact Wasm, checksums and both build logs to a tagged prerelease only after
all jobs pass. Reviewers should reproduce the release tag on their own machine.
Automated CI agreement does not replace that independent verification.

## Contract and authority

Read [Security and authority](docs/SECURITY-AND-AUTHORITY.md) before voting.
The contract contains only a self-serve Base claim for the current owner of an
eligible Pass 001–100. It has no paid execution path, quote key, marketplace,
evolution system, pause switch, admin execute message, transfer function or
funds-distribution logic. No independent third-party audit is supplied.

## Governance deployment model

The intended new proposal stores **only the reviewed Wasm** using governance
MsgStoreCode, without adding a developer wallet to code-upload permissions.
The payload must be checked against the live Terra Classic message schema,
limits and permissions and simulated before any deposit/signature request.
Instantiation is separate: the reviewed Founding Pass contract address and
approved image hash only. No Founder wallet, quote key, mint price or operational
admin is required by this Base-only module. The deployment must also specify no
chain-level migration admin.
No deployment is performed by this repository's build/release workflow.

The prior on-chain proposal #12238 is not changed by source or forum edits.
See [review notes](release/REVIEW-NOTES.md).
