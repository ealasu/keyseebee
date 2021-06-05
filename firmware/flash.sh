#!/usr/bin/env bash
set -e

nix-shell --run 'cargo build --release'
arm-none-eabi-size target/thumbv6m-none-eabi/release/keyseebee
nix-shell --run 'arm-none-eabi-objcopy -O binary target/thumbv6m-none-eabi/release/keyseebee target/firmware.bin'
nix-shell --run 'JLinkExe -device ATSAMD21E17 -if swd -speed 4000 flash.jlink'

#~/.cargo/bin/hf2 flash --file target/firmware.bin
#sudo bossac -i -d --port=ttyACM0 --offset=0x2000 -e -w -v -b target/firmware.bin -R
