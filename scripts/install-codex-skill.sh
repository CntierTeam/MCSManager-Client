#!/usr/bin/env bash
# Install the mcsmanager-cli Codex skill into $CODEX_HOME/skills (default ~/.codex/skills).
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
SRC="$REPO_ROOT/skills/mcsmanager-cli"
CODEX_HOME="${CODEX_HOME:-$HOME/.codex}"
DEST_ROOT="${CODEX_HOME}/skills"
DEST="$DEST_ROOT/mcsmanager-cli"
MODE="${1:-copy}" # copy | link

if [[ ! -f "$SRC/SKILL.md" ]]; then
  echo "error: skill not found at $SRC/SKILL.md" >&2
  exit 1
fi

mkdir -p "$DEST_ROOT"

if [[ -e "$DEST" || -L "$DEST" ]]; then
  echo "removing existing: $DEST"
  rm -rf "$DEST"
fi

case "$MODE" in
  copy)
    mkdir -p "$DEST"
    # portable copy without requiring GNU cp -a across all hosts
    if command -v rsync >/dev/null 2>&1; then
      rsync -a --delete "$SRC/" "$DEST/"
    else
      cp -R "$SRC/." "$DEST/"
    fi
    echo "installed (copy): $DEST"
    ;;
  link)
    ln -sfn "$SRC" "$DEST"
    echo "installed (symlink): $DEST -> $SRC"
    ;;
  *)
    echo "usage: $0 [copy|link]" >&2
    exit 2
    ;;
esac

echo "Codex home: $CODEX_HOME"
echo "Skill will be available on the next Codex turn."
ls -la "$DEST"
test -f "$DEST/SKILL.md"
echo "OK: SKILL.md present"
