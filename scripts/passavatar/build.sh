#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../.."
contract_dir=contracts/rd-passavatar
artifact_dir=artifacts/passavatar
rustup toolchain install 1.85.1 --profile minimal --component rust-src --component rustfmt --target wasm32-unknown-unknown
cargo +1.85.1 fmt --manifest-path "$contract_dir/Cargo.toml" --check
cargo +1.85.1 test --manifest-path "$contract_dir/Cargo.toml" --locked
# Rebuild std as well: prebuilt std can retain unsupported reference-type opcodes.
RUSTC_BOOTSTRAP=1 RUSTFLAGS="-C target-feature=-bulk-memory,-reference-types,-multivalue" \
  cargo +1.85.1 build -Z build-std=std,panic_abort --manifest-path "$contract_dir/Cargo.toml" \
  --locked --release --target wasm32-unknown-unknown --target-dir "$contract_dir/target-compatible"
mkdir -p "$artifact_dir"
cp "$contract_dir/target-compatible/wasm32-unknown-unknown/release/rd_passavatar.wasm" "$artifact_dir/"
expected_sha256="6e2075f95c52b69b116e681d2a8b19778081f2b9de67819bb4063816ce120a84"
actual_sha256="$(sha256sum "$artifact_dir/rd_passavatar.wasm" | awk '{print $1}')"
if [[ "$actual_sha256" != "$expected_sha256" ]]; then
  echo "Wasm SHA-256 mismatch: expected $expected_sha256, got $actual_sha256" >&2
  exit 1
fi
cosmwasm-check "$artifact_dir/rd_passavatar.wasm"
# Rebuild in an independent target directory and compare the actual bytes.
RUSTC_BOOTSTRAP=1 RUSTFLAGS="-C target-feature=-bulk-memory,-reference-types,-multivalue" \
  cargo +1.85.1 build -Z build-std=std,panic_abort --manifest-path "$contract_dir/Cargo.toml" \
  --locked --release --target wasm32-unknown-unknown --target-dir "$contract_dir/target-reproducible"
cmp "$artifact_dir/rd_passavatar.wasm" "$contract_dir/target-reproducible/wasm32-unknown-unknown/release/rd_passavatar.wasm"
(cd "$artifact_dir" && sha256sum rd_passavatar.wasm > checksums.txt && sha256sum --check checksums.txt)

