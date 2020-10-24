#!/usr/bin/env bash
nix-shell --run 'cargo test --target x86_64-unknown-linux-gnu'
