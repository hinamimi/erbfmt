#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage:
  scripts/publish-rubygems.sh --version VERSION --asset-dir DIR [--dry-run]
  scripts/publish-rubygems.sh --version VERSION --asset-dir DIR --yes

Publishes the four platform-specific erbfmt gems to RubyGems.org.

The RubyGems API key is read from the first available source:

  1. RUBYGEMS_API_KEY environment variable
  2. GEM_HOST_API_KEY environment variable used by RubyGems itself
  3. API_KEY environment variable
  4. .env entries named RUBYGEMS_API_KEY, GEM_HOST_API_KEY, or API_KEY

The key is never printed. Before running gem push, the script exports the chosen
key as GEM_HOST_API_KEY because that is the environment variable read by
RubyGems.
USAGE
}

version=""
asset_dir=""
dry_run=false
yes=false

while [[ "$#" -gt 0 ]]; do
  case "$1" in
    --version)
      version="${2:-}"
      shift 2
      ;;
    --asset-dir)
      asset_dir="${2:-}"
      shift 2
      ;;
    --dry-run)
      dry_run=true
      shift
      ;;
    --yes)
      yes=true
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "unexpected argument: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

if [[ -z "$version" || -z "$asset_dir" ]]; then
  usage >&2
  exit 1
fi

if [[ ! "$version" =~ ^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$ ]]; then
  echo "invalid release version: $version" >&2
  exit 1
fi

if [[ ! -d "$asset_dir" ]]; then
  echo "asset directory does not exist: $asset_dir" >&2
  exit 1
fi

trim() {
  local value="$1"
  value="${value#"${value%%[![:space:]]*}"}"
  value="${value%"${value##*[![:space:]]}"}"
  printf '%s' "$value"
}

read_env_file_value() {
  local key="$1"
  local line name value

  [[ -f .env ]] || return 1

  while IFS= read -r line || [[ -n "$line" ]]; do
    line="$(trim "$line")"
    [[ -z "$line" || "$line" == \#* || "$line" != *=* ]] && continue

    name="$(trim "${line%%=*}")"
    value="$(trim "${line#*=}")"
    [[ "$name" == "$key" ]] || continue

    if [[ "$value" == \"*\" && "$value" == *\" ]]; then
      value="${value:1:${#value}-2}"
    elif [[ "$value" == \'*\' && "$value" == *\' ]]; then
      value="${value:1:${#value}-2}"
    fi

    printf '%s' "$value"
    return 0
  done < .env

  return 1
}

api_key="${RUBYGEMS_API_KEY:-${GEM_HOST_API_KEY:-${API_KEY:-}}}"
if [[ -z "$api_key" ]]; then
  for key_name in RUBYGEMS_API_KEY GEM_HOST_API_KEY API_KEY; do
    if api_key="$(read_env_file_value "$key_name")" && [[ -n "$api_key" ]]; then
      break
    fi
  done
fi

if [[ -z "$api_key" ]]; then
  echo "RubyGems API key not found. Set RUBYGEMS_API_KEY or put API_KEY in .env." >&2
  exit 1
fi

expected=(
  "erbfmt-${version}-x86_64-linux-gnu.gem"
  "erbfmt-${version}-x86_64-darwin.gem"
  "erbfmt-${version}-arm64-darwin.gem"
  "erbfmt-${version}-x64-mingw-ucrt.gem"
)

gems=()
for asset in "${expected[@]}"; do
  gem_path="${asset_dir%/}/$asset"
  if [[ ! -f "$gem_path" ]]; then
    echo "missing RubyGem asset: $gem_path" >&2
    exit 1
  fi
  gems+=("$gem_path")
done

echo "RubyGems assets ready for erbfmt $version:"
for gem_path in "${gems[@]}"; do
  echo "  $(basename "$gem_path")"
done

if [[ "$dry_run" == true ]]; then
  echo "dry run: no gems were pushed."
  exit 0
fi

if [[ "$yes" != true ]]; then
  read -r -p "Type $version to publish these gems to RubyGems.org: " confirmation
  if [[ "$confirmation" != "$version" ]]; then
    echo "confirmation did not match; aborting." >&2
    exit 1
  fi
fi

export GEM_HOST_API_KEY="$api_key"

for gem_path in "${gems[@]}"; do
  gem push "$gem_path" --host https://rubygems.org
done
