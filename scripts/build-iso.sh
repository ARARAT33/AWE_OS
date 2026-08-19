#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT="$ROOT/dist"
BUILD="$ROOT/build"
ISO="$OUT/aweos-x86_64.iso"
IMG="$OUT/aweos-x86_64.img"
KERNEL="$ROOT/target/x86_64-unknown-none/release/aweos"

mkdir -p "$OUT" "$BUILD"

rustup target add x86_64-unknown-none 2>/dev/null || true
rustup target add x86_64-unknown-uefi 2>/dev/null || true

printf '==> Building AWEOS Kernel (x86_64-unknown-none)...\n'
cargo build --release -p aweos-kernel-bin --target x86_64-unknown-none

printf '==> Building AWEOS EFI Loader (x86_64-unknown-uefi)...\n'
cargo build --release -p aweloader --target x86_64-unknown-uefi || true

printf '==> Generating AWEOS ISO and IMG images...\n'
python3 "$ROOT/scripts/make_images.py"

if command -v grub-mkrescue >/dev/null 2>&1; then
    printf '==> Running grub-mkrescue for GRUB ISO...\n'
    rm -rf "$OUT/iso"
    mkdir -p "$OUT/iso/boot/grub"
    cp "$KERNEL" "$OUT/iso/boot/aweos"
    cp "$ROOT/kernel-bin/grub.cfg" "$OUT/iso/boot/grub/grub.cfg"
    grub-mkrescue -o "$ISO" "$OUT/iso" 2>/dev/null || true
fi

printf 'AWEOS ISO: %s\n' "$ISO"
printf 'AWEOS IMG: %s\n' "$IMG"
printf 'AWEOS UEFI IMG: %s/aweos-uefi.img\n' "$OUT"
printf 'AWEOS BIOS IMG: %s/aweos-bios.img\n' "$OUT"
