#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TARGET="${LAMBDA_TARGET:-aarch64-unknown-linux-gnu}"
PACKAGE_DIR="$ROOT_DIR/target/lambda/public-api"
ZIP_PATH="$ROOT_DIR/target/lambda/public-api-arm64.zip"

cd "$ROOT_DIR"

cargo zigbuild --release --target "$TARGET" -p mwt-public-api

mkdir -p "$PACKAGE_DIR"
cp "$ROOT_DIR/target/$TARGET/release/mwt-public-api" "$PACKAGE_DIR/bootstrap"

(
  cd "$PACKAGE_DIR"
  zip -9 -FS "$ZIP_PATH" bootstrap
)

file "$ROOT_DIR/target/$TARGET/release/mwt-public-api"
unzip -l "$ZIP_PATH"
