#!/usr/bin/env bash

nix-shell --run 'arm-none-eabi-objcopy -O binary target/thumbv6m-none-eabi/release/keyseebee target/firmware.bin'
nix-shell --run 'openocd'
