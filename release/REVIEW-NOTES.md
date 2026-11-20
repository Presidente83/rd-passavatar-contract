# Review candidate: path-normalized Avatar Club 100

This candidate corrects build reproducibility and documents existing authority
and fee-quote risks. It is not an audited or mainnet-approved release.

- Rust contract behavior is unchanged from the published source snapshot.
- Paths for the source checkout, Cargo registry, and rebuilt Rust standard
  library are normalized during compilation.
- The old `6e2075f95c52b69b116e681d2a8b19778081f2b9de67819bb4063816ce120a84`
  identifies the old, path-dependent binary. It is superseded for this candidate,
  not silently overwritten or claimed to match.
- Read `docs/SECURITY-AND-AUTHORITY.md` for the immutable quote public key,
  compromised-signer economic risk, pause/offer powers, and migration policy.
- CI compares actual Wasm bytes from separate runners with separate paths.
  This is automated reproduction, **not independent third-party verification**.
- No governance submission, deposit, contract upload, or instantiation is made
  by publishing this release. Proposal #12238 is not modified.
