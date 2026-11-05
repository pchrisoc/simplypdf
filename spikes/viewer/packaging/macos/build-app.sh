#!/bin/sh
set -eu

if [ "$(uname -s)" != Darwin ]; then
    echo "Build the macOS app on a Mac." >&2
    exit 1
fi

packaging_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
viewer_dir=$(CDPATH= cd -- "$packaging_dir/../.." && pwd)
project_dir=$(CDPATH= cd -- "$viewer_dir/../.." && pwd)
output_dir="$project_dir/build/macos"
target_dir="$viewer_dir/target"

cargo build --release --locked --manifest-path "$viewer_dir/Cargo.toml" --target-dir "$target_dir"

mkdir -p "$output_dir"
staging=$(mktemp -d "$output_dir/.package.XXXXXX")
trap 'rm -rf "$staging"' EXIT
bundle="$staging/SimplyPDF.app"
mkdir -p "$bundle/Contents/MacOS" "$bundle/Contents/Resources"
cp "$packaging_dir/Info.plist" "$bundle/Contents/Info.plist"
cp "$target_dir/release/simplypdf-viewer-spike" "$bundle/Contents/MacOS/SimplyPDF"
cp "$project_dir/tests/fixtures/page-geometry.pdf" "$bundle/Contents/Resources/"
cp -R "$project_dir/crates/simplypdf-draw/licenses" "$bundle/Contents/Resources/Draw-Licenses"
plutil -lint "$bundle/Contents/Info.plist"
codesign --force --sign - "$bundle"
codesign --verify --strict "$bundle"

destination="$output_dir/SimplyPDF.app"
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
