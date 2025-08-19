#!/usr/bin/env bash
set -euo pipefail

mkdir -p "$HOME/.cargo/bin"

if ! command -v cargo-binstall >/dev/null 2>&1; then
  tmpdir="$(mktemp -d)"
  curl -L "https://github.com/cargo-bins/cargo-binstall/releases/latest/download/cargo-binstall-x86_64-unknown-linux-gnu.tgz" \
    | tar -xz -C "$tmpdir"
  install -m 755 "$tmpdir/cargo-binstall" "$HOME/.cargo/bin/"
  rm -rf "$tmpdir"
fi

rustup component add rustfmt clippy
cargo binstall cargo-machete --no-confirm

if ! command -v wrkflw >/dev/null 2>&1; then
  cargo binstall wrkflw@0.7.0 --no-confirm || {
    tmpdir="$(mktemp -d)"
    curl -L "https://github.com/bahdotsh/wrkflw/releases/download/v0.7.0/wrkflw-v0.7.0-linux-x86_64.tar.gz" \
      | tar -xz -C "$tmpdir"
    install -m 755 "$tmpdir/wrkflw" "$HOME/.cargo/bin/"
    rm -rf "$tmpdir"
  }
fi
