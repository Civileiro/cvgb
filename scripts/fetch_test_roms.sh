#!/usr/bin/env bash
set -euo pipefail

TAG="v7.0"
DEST="tests/roms"

if [ -f "$DEST/.fetched-$TAG" ]; then
    echo "test ROMs $TAG already present";
    exit 0;
fi

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT


download_link="https://github.com/c-sp/game-boy-test-roms/releases/download/$TAG/game-boy-test-roms-$TAG.zip"
echo "downloading $download_link"

rm -rf "$DEST"
mkdir -p "$DEST"

curl -fSL -o "$tmp/roms.zip" "$download_link"
unzip -q "$tmp/roms.zip" -d "$DEST"


touch "$DEST/.fetched-$TAG"
