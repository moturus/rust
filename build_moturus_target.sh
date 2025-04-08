#!/bin/sh

TARGET=x86_64-unknown-moturus \
CARGO_CFG_TARGET_ARCH=x86_64 \
CARGO_CFG_TARGET_VENDOR=unknown \
CARGO_CFG_TARGET_OS=moturus \
CARGO_CFG_TARGET_ENV="" \
./x.py build --stage 2 clippy library

