#!/usr/bin/env bash
# Install mcsm from GitHub Releases (CntierTeam/MCSManager-Client).
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/CntierTeam/MCSManager-Client/main/scripts/install.sh | bash
#   ./scripts/install.sh                 # latest stable (non-prerelease)
#   ./scripts/install.sh --continuous    # continuous pre-release from main
#   ./scripts/install.sh --version v0.1.0
#   PREFIX=/usr/local ./scripts/install.sh
#
# Env:
#   MCSM_REPO      default CntierTeam/MCSManager-Client
#   MCSM_CHANNEL   stable|continuous  (default stable)
#   MCSM_VERSION   explicit tag, e.g. v0.1.0
#   PREFIX         install dir (default: ~/.local)
#   BINDIR         binary dir (default: $PREFIX/bin)
set -euo pipefail

REPO="${MCSM_REPO:-CntierTeam/MCSManager-Client}"
CHANNEL="${MCSM_CHANNEL:-stable}"
VERSION="${MCSM_VERSION:-}"
PREFIX="${PREFIX:-$HOME/.local}"
BINDIR="${BINDIR:-$PREFIX/bin}"
VERIFY_SHA="${MCSM_VERIFY_SHA:-1}"

usage() {
  sed -n '2,16p' "$0" | sed 's/^# \{0,1\}//'
  exit "${1:-0}"
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    -h|--help) usage 0 ;;
    --continuous|--nightly) CHANNEL=continuous; shift ;;
    --stable) CHANNEL=stable; shift ;;
    --version|-v)
      VERSION="${2:-}"
      [[ -n "$VERSION" ]] || { echo "error: --version needs a tag" >&2; exit 2; }
      shift 2
      ;;
    --prefix)
      PREFIX="${2:-}"
      BINDIR="$PREFIX/bin"
      shift 2
      ;;
    --no-verify) VERIFY_SHA=0; shift ;;
    *)
      echo "error: unknown arg: $1" >&2
      usage 2
      ;;
  esac
done

need() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "error: missing required command: $1" >&2
    exit 1
  }
}

need curl
need tar
need uname
need mktemp

os="$(uname -s | tr '[:upper:]' '[:lower:]')"
arch="$(uname -m)"

case "$os" in
  linux)  platform=linux ;;
  darwin) platform=macos ;;
  msys*|cygwin*|mingw*) platform=windows ;;
  *)
    echo "error: unsupported OS: $(uname -s)" >&2
    exit 1
    ;;
esac

case "$arch" in
  x86_64|amd64) cpu=amd64 ;;
  aarch64|arm64) cpu=arm64 ;;
  *)
    echo "error: unsupported arch: $arch" >&2
    exit 1
    ;;
esac

artifact="mcsm-${platform}-${cpu}"
if [[ "$platform" == "windows" ]]; then
  archive="${artifact}.zip"
  need unzip
else
  archive="${artifact}.tar.gz"
fi

# linux arm64 / windows from this script: only if assets exist
api="https://api.github.com/repos/${REPO}/releases"

json_get() {
  # tiny JSON string extractor without jq: find first "key": "value"
  local key="$1"
  sed -n "s/.*\"${key}\"[[:space:]]*:[[:space:]]*\"\\([^\"]*\\)\".*/\\1/p" | head -n1
}

resolve_tag() {
  if [[ -n "$VERSION" ]]; then
    echo "$VERSION"
    return
  fi
  if [[ "$CHANNEL" == "continuous" ]]; then
    echo "continuous"
    return
  fi
  # Prefer GitHub /releases/latest (latest non-prerelease). Fallback: first tag starting with v.
  local tag
  tag="$(curl -fsSL "${api}/latest" | json_get tag_name || true)"
  if [[ -n "$tag" ]]; then
    echo "$tag"
    return
  fi
  tag="$(curl -fsSL "${api}?per_page=20" \
    | tr ',' '\n' \
    | sed -n 's/.*"tag_name"[[:space:]]*:[[:space:]]*"\(v[^"]*\)".*/\1/p' \
    | head -n1 || true)"
  if [[ -z "$tag" ]]; then
    echo "error: no stable release found; try --continuous" >&2
    exit 1
  fi
  echo "$tag"
}

tag="$(resolve_tag)"
echo "repo=$REPO tag=$tag artifact=$artifact"

asset_url="https://github.com/${REPO}/releases/download/${tag}/${archive}"
sum_url="${asset_url}.sha256"

tmpdir="$(mktemp -d)"
cleanup() { rm -rf "$tmpdir"; }
trap cleanup EXIT

echo "download: $asset_url"
if ! curl -fsSL -o "$tmpdir/$archive" "$asset_url"; then
  echo "error: failed to download $archive for tag=$tag" >&2
  echo "hint: asset may not exist for this platform yet, or release still building" >&2
  exit 1
fi

if [[ "$VERIFY_SHA" == "1" ]]; then
  if curl -fsSL -o "$tmpdir/$archive.sha256" "$sum_url"; then
    need sha256sum
    (cd "$tmpdir" && sha256sum -c "$archive.sha256")
  else
    echo "warn: checksum file missing; skip verify" >&2
  fi
fi

mkdir -p "$BINDIR"
extract_dir="$tmpdir/extract"
mkdir -p "$extract_dir"

if [[ "$platform" == "windows" ]]; then
  unzip -q "$tmpdir/$archive" -d "$extract_dir"
  bin_src="$(find "$extract_dir" -type f \( -name mcsm.exe -o -name mcsm \) | head -n1)"
  dest="$BINDIR/mcsm.exe"
else
  tar -xzf "$tmpdir/$archive" -C "$extract_dir"
  bin_src="$(find "$extract_dir" -type f -name mcsm | head -n1)"
  dest="$BINDIR/mcsm"
fi

if [[ -z "${bin_src:-}" || ! -f "$bin_src" ]]; then
  echo "error: mcsm binary not found inside archive" >&2
  find "$extract_dir" -type f >&2 || true
  exit 1
fi

install -m 0755 "$bin_src" "$dest"
echo "installed: $dest"
"$dest" --version 2>/dev/null || "$dest" --help | head -n 3 || true

case ":$PATH:" in
  *":$BINDIR:"*) ;;
  *)
    echo
    echo "note: $BINDIR is not on PATH. Add for example:"
    echo "  export PATH=\"$BINDIR:\$PATH\""
    ;;
esac

echo "OK"
