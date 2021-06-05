#!/usr/bin/env bash
set -e

nix-shell --run 'cargo build --release'
nix-shell --run 'arm-none-eabi-size target/thumbv6m-none-eabi/release/keyseebee'
nix-shell --run 'arm-none-eabi-objcopy -O binary target/thumbv6m-none-eabi/release/keyseebee target/firmware.bin'
nix-shell --run 'JLinkExe -device ATSAMD21E17 -if swd -speed 4000 flash.jlink'

