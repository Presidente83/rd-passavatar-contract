#!/usr/bin/env bash
# Pinned native build with canonical paths, including rebuilt Rust std.
set -euo pipefail
repo_dir="$(cd "$(dirname "$0")/../.." && pwd -P)"
contract_dir="$repo_dir/contracts/rd-passavatar"
artifact_dir="$repo_dir/artifacts/passavatar"
mkdir -p "$artifact_dir"
exec > >(tee "$artifact_dir/build.log") 2>&1
set -x
rustup toolchain install 1.85.1 --no-self-update --profile minimal --component rust-src --component rustfmt --target wasm32-unknown-unknown
rustc +1.85.1 -vV
cargo +1.85.1 -V
uname -sm
# Ambient flags or profiles must not silently change the release recipe.
unset RUSTFLAGS CARGO_ENCODED_RUSTFLAGS CARGO_BUILD_RUSTFLAGS
if env | grep -q '^CARGO_PROFILE_'; then
  echo 'Remove CARGO_PROFILE_* environment overrides before a release build.' >&2
  exit 1
fi
cargo +1.85.1 fmt --manifest-path "$contract_dir/Cargo.toml" --check
cargo +1.85.1 test --manifest-path "$contract_dir/Cargo.toml" --locked
sysroot="$(rustc +1.85.1 --print sysroot)"
cargo_home="$(cd "${CARGO_HOME:-$HOME/.cargo}" && pwd -P)"
# Keep every path that can be embedded by rustc/Cargo stable across machines.
# In particular, build_target is deliberately random, so it must be remapped
# just like the checkout, registry and Rust sysroot paths.
build_target="$(mktemp -d)"
rustup_home="$(cd "${RUSTUP_HOME:-$HOME/.rustup}" && pwd -P)"
# Encoded flags preserve paths containing spaces. Most-specific mapping is last.
flags=("-C" "target-feature=-bulk-memory,-reference-types,-multivalue"
  "--remap-path-prefix=$repo_dir=/source"
  "--remap-path-prefix=$cargo_home=/cargo"
  "--remap-path-prefix=$rustup_home=/rustup"
  "--remap-path-prefix=$sysroot=/rust-toolchain"
  "--remap-path-prefix=$build_target=/target")
printf -v CARGO_ENCODED_RUSTFLAGS '%s\x1f' "${flags[@]}"
export CARGO_ENCODED_RUSTFLAGS="${CARGO_ENCODED_RUSTFLAGS%$'\x1f'}"
# Never reuse build outputs. This is one build, not an independent reproduction.
trap 'rm -rf "$build_target"' EXIT
cd "$contract_dir"
# Do not inherit a machine-specific timestamp from the caller.
export SOURCE_DATE_EPOCH=0
RUSTC_BOOTSTRAP=1 cargo +1.85.1 build -Z build-std=std,panic_abort --locked --release --target wasm32-unknown-unknown --target-dir "$build_target"
cp "$build_target/wasm32-unknown-unknown/release/rd_passavatar.wasm" "$artifact_dir/"
cd "$artifact_dir"
sha256sum rd_passavatar.wasm > checksums.txt
cat checksums.txt
# A successful build must not carry machine-specific absolute paths. This is a
# guardrail for future recipe changes: the normalised paths above are allowed,
# but runner, home, temporary and workspace paths are not.
if strings -a rd_passavatar.wasm | grep -Eq '/(home|Users|private|tmp|workspace|runner)/'; then
  echo 'Build contains an unnormalised machine-specific path.' >&2
  exit 1
fi
# An initial candidate can be built before its measured hash is recorded.
# Published candidates must have a committed expected checksum.
if [[ -f "$repo_dir/release/checksums.txt" ]]; then
  sha256sum --check "$repo_dir/release/checksums.txt"
fi
