#!/usr/bin/env bash

export RUSTFLAGS="-D warnings"

(
    cd dw1000 &&
    cargo test --verbose &&
    cargo doc)

(
    cd dwm1001 &&
    cargo build --verbose --examples --all-features)
