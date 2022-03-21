#!/usr/bin/env bash

set -euo pipefail
export RUSTFLAGS="-Zinstrument-coverage"
export LLVM_PROFILE_FILE="coverage-%p-%m.profraw"
rustup default nightly-2021-10-21
rustup toolchain list
cargo +nightly-2021-10-21 install cargo2junit
cargo test --all --tests --no-fail-fast --all-features -- -Z unstable-options --format json | cargo2junit > results.xml
