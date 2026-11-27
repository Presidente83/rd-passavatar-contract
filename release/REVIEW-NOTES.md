# Review candidate: Avatar Club 100 Base-only

This candidate narrows the contract to the first governance module: a free,
self-serve Base claim for eligible Founding Pass holders. It is not audited or
mainnet-approved.

- The previous paid and administrative execution paths have been removed from
  the source: marketplace, evolutions, signed quotes, pause, secondary-market
  controls, transfers, approvals, burns and payout messages.
- The claimant must be the current owner of the requested Pass 001–100. The
  contract performs the owner query itself and does not trust a Founder wallet
  to submit claims on behalf of users.
- The contract receives no funds and emits no bank messages.
- Paths for the source checkout, Cargo registry, and rebuilt Rust standard
  library are normalized during compilation, as are the Rustup installation and
  temporary Cargo target directory.
- The build fails if runner, home, temporary or workspace paths remain embedded
  in the Wasm.
- Verified CI candidate commit: `9a473efe9c92ae172be3d5c580879d6cf40a386c`.
- Verified Wasm SHA-256: `e6ff051cb7358f1272bc36981849756cefae54f96a66e6b12ba83cc214d70d5c`.
- GitHub Actions run: https://github.com/Presidente83/rd-passavatar-contract/actions/runs/45984431710
- Read `docs/SECURITY-AND-AUTHORITY.md` for the complete Base-only trust model.
- CI compares actual Wasm bytes from separate runners with separate paths.
  This is automated reproduction, **not independent third-party verification**.
- The comparison passed on Ubuntu 22.04 and Ubuntu 24.04; both runners also
  passed tests and `cosmwasm-check`.
- No governance submission, deposit, contract upload, or instantiation is made
  by publishing this release. Proposal #12238 is not modified.
