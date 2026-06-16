#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

version="$(awk -F '"' '/^version =/ { print $2; exit }' Cargo.toml)"
target="${1:-$(rustc -vV | awk '/^host:/ { print $2 }')}"
dist_dir="dist"
binary_name="erbfmt"

write_sha256() {
  local file="$1"
  local checksum_file="$2"
  local base

  base="$(basename "$file")"

  if command -v sha256sum >/dev/null 2>&1; then
    (cd "$(dirname "$file")" && sha256sum "$base" > "$(basename "$checksum_file")")
  else
    (cd "$(dirname "$file")" && shasum -a 256 "$base" > "$(basename "$checksum_file")")
  fi
}

if [[ "$target" == *"windows"* ]]; then
  binary_name="erbfmt.exe"
fi

archive_base="erbfmt-${version}-${target}"

cargo build --release --locked --target "$target"

mkdir -p "$dist_dir/$archive_base"
cp "target/$target/release/$binary_name" "$dist_dir/$archive_base/$binary_name"
cp LICENSE.txt "$dist_dir/$archive_base/LICENSE.txt"
cp README.md "$dist_dir/$archive_base/README.md"

if [[ "$target" == *"windows"* ]]; then
  (cd "$dist_dir" && zip -qr "${archive_base}.zip" "$archive_base")
  checksum_file="${archive_base}.zip.sha256"
  write_sha256 "$dist_dir/${archive_base}.zip" "$dist_dir/$checksum_file"
else
  (cd "$dist_dir" && tar -czf "${archive_base}.tar.gz" "$archive_base")
  checksum_file="${archive_base}.tar.gz.sha256"
  write_sha256 "$dist_dir/${archive_base}.tar.gz" "$dist_dir/$checksum_file"
fi

rm -rf "$dist_dir/$archive_base"

echo "Created:"
if [[ "$target" == *"windows"* ]]; then
  echo "  $dist_dir/${archive_base}.zip"
else
  echo "  $dist_dir/${archive_base}.tar.gz"
fi
echo "  $dist_dir/$checksum_file"
