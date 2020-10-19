#!/usr/bin/env bash
set -e

arm-none-eabi-objcopy -O binary target/thumbv6m-none-eabi/release/keyseebee target/firmware.bin
~/.cargo/bin/hf2 flash --file target/firmware.bin

#sudo bossac -i -d --port=ttyACM0 --offset=0x2000 -e -w -v -b target/firmware.bin -R
