#!/usr/bin/env bash
set -e

nix-shell --run 'cargo build --release'
#nix-shell --run 'arm-none-eabi-objcopy -O binary target/thumbv6m-none-eabi/release/keyseebee target/firmware.bin'
#nix-shell --run 'hf2 -v 230a -p 00e9 flash --address 0x2000 --file target/firmware.bin'
nix-shell --run 'hf2 -v 0x230a -p 0x00e9 elf target/thumbv6m-none-eabi/release/keyseebee'

#sudo bossac -i -d --port=ttyACM0 --offset=0x2000 -e -w -v -b target/firmware.bin -R
