#!/usr/bin/env bash

export RUSTFLAGS="-D warnings"

(
    cd dw1000 &&
    cargo test --verbose &&
    cargo doc)
