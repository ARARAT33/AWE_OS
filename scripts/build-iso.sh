#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT="$ROOT/dist"
ISO="$OUT/aweos-x86_64.iso"
KERNEL="$ROOT/target/x86_64-unknown-none/release/aweos"

rustup target add x86_64-unknown-none
cargo build --release -p aweos-kernel-bin --target x86_64-unknown-none -Zbuild-std=core,compiler_builtins 2>/dev/null || \
cargo build --release -p aweos-kernel-bin --target x86_64-unknown-none

rm -rf "$OUT/iso"
mkdir -p "$OUT/iso/boot/grub"
cp "$KERNEL" "$OUT/iso/boot/aweos"
cp "$ROOT/kernel-bin/grub.cfg" "$OUT/iso/boot/grub/grub.cfg"

grub-mkrescue -o "$ISO" "$OUT/iso"
printf 'AWEOS ISO: %s\n' "$ISO"
