#!/bin/bash
set -xeu

ARCH=${1}
echo "Installing cross..."
cargo install cross --git https://github.com/cross-rs/cross
echo "Building for arch ${ARCH}..."
sudo cross build --release --target ${ARCH}-unknown-linux-gnu --bin daemon --bin routing --bin runner --bin cron --bin logs --bin downloader --all-features
echo "Copying bin files..."
OUT_FOLDER="alfred-core_${ARCH}"
BIN_FOLDER="target/${ARCH}-unknown-linux-gnu/release"
mkdir $OUT_FOLDER
cp $BIN_FOLDER/daemon $OUT_FOLDER/
cp $BIN_FOLDER/cron $OUT_FOLDER/
cp $BIN_FOLDER/downloader $OUT_FOLDER/
cp $BIN_FOLDER/logs $OUT_FOLDER/
cp $BIN_FOLDER/routing $OUT_FOLDER/
cp $BIN_FOLDER/runner $OUT_FOLDER/

tar czf alfred-core_${ARCH}.tar.gz $OUT_FOLDER