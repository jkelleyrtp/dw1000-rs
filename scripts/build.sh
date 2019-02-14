#!/usr/bin/env bash

export RUSTFLAGS="-D warnings"

cargo test --verbose
