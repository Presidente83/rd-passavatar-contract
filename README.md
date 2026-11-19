# RD Station — Avatar Club 100 CosmWasm contract

This repository publishes the contract source and build recipe for independent review. It contains only the Avatar Club 100 contract package, not the RD Station application.

## Release target

- Chain: Terra Classic `columbus-5`
- Expected Wasm SHA-256: `6e2075f95c52b69b116e681d2a8b19778081f2b9de67819bb4063816ce120a84`
- Source snapshot: `Presidente83/rd-station-lunc`, commit `7d16eb41ba733e0900a4f7c12fa37e151c6f374b`
- Contract package: `contracts/rd-passavatar`

The SHA-256 is a release identifier. The build script checks that the compiled Wasm matches this exact value and then independently rebuilds and byte-compares the output. It does not claim that the chain itself enforces the checksum.

## Reproduce the Wasm

On Ubuntu or another Linux system with Git, Rustup, and the standard build tools:

```bash
cargo install cosmwasm-check --version 1.5.11 --locked
bash scripts/passavatar/build.sh
```

The script installs Rust 1.85.1, `rust-src`, `rustfmt`, and the `wasm32-unknown-unknown` target; runs formatting and unit tests; builds the Wasm; checks the expected SHA-256; validates the Wasm; repeats the build in a separate target directory; and compares the two outputs byte for byte.

A GitHub Actions workflow runs the same build on pushes and pull requests. Its generated Wasm and checksum are uploaded as a workflow artifact.

## Governance deployment model

The intended deployment is a governance-authorized `MsgStoreCode` containing this exact Wasm. The governance module is the uploader; no developer wallet receives a standing `code_upload_access` permission. Instantiation is a separate, subsequent chain message and should be described and reviewed separately.

## Security review status

No independent third-party audit is included in this repository. Please review the source and raise technical findings in the linked Agora discussion before any governance resubmission or mainnet deployment.
