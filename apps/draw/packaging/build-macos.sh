#!/bin/sh
set -eu

if [ "$(uname -s)" != Darwin ]; then
    echo "Build the macOS bundle on a Mac." >&2
    exit 1
fi

packaging=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
app_dir=$(CDPATH= cd -- "$packaging/.." && pwd)
repo=$(CDPATH= cd -- "$app_dir/../.." && pwd)
output="$repo/build/macos"
target_dir=${CARGO_TARGET_DIR:-"$app_dir/target"}
cargo build --release --locked --manifest-path "$app_dir/Cargo.toml" --target-dir "$target_dir"

mkdir -p "$output"
staging=$(mktemp -d "$output/.draw-rust.XXXXXX")
trap 'rm -rf "$staging"' EXIT
bundle="$staging/Draw Rust.app"
mkdir -p "$bundle/Contents/MacOS" "$bundle/Contents/Resources"
cp "$packaging/Info.plist" "$bundle/Contents/Info.plist"
cp "$target_dir/release/draw-rust" "$bundle/Contents/MacOS/"
cp -R "$repo/crates/simplypdf-draw/licenses" "$bundle/Contents/Resources/Draw-Licenses"
plutil -lint "$bundle/Contents/Info.plist"
codesign --force --sign - "$bundle"
codesign --verify --strict "$bundle"

destination="$output/Draw Rust.app"
if [ -e "$destination" ]; then
    mv "$destination" "$staging/previous.app"
fi
if ! mv "$bundle" "$destination"; then
    if [ -e "$staging/previous.app" ]; then
        mv "$staging/previous.app" "$destination"
    fi
    exit 1
fi
printf '%s\n' "$destination"
