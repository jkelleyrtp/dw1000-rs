#!/usr/bin/env bash

export RUSTFLAGS="-D warnings"

# `.cargo/config` defaults us to the microcontroller's target triple. We need
# to override this here, to run `cargo test`. You may need to adapt this,
# depending on you platform.
TARGET=x86_64-unknown-linux-gnu

cargo test --verbose --target=$TARGET &&
cargo doc --verbose --target=$TARGET
