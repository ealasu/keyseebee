#!/usr/bin/env bash
nix-shell --run 'cd stuff && cargo test --target x86_64-unknown-linux-gnu -- --nocapture'

