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
- No checksum is valid for this candidate until the build is run and reproduced
  from this exact commit.
- Read `docs/SECURITY-AND-AUTHORITY.md` for the complete Base-only trust model.
- CI compares actual Wasm bytes from separate runners with separate paths.
  This is automated reproduction, **not independent third-party verification**.
- No governance submission, deposit, contract upload, or instantiation is made
  by publishing this release. Proposal #12238 is not modified.
