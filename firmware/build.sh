#!/usr/bin/env bash
nix-shell --run 'cargo build --release'
arm-none-eabi-size target/thumbv6m-none-eabi/release/keyseebee

