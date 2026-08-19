#!/usr/bin/env sh
# Installs the cdz binary + shell wrapper from the latest GitHub release.
#   curl -fsSL https://raw.githubusercontent.com/iwandejong/i/main/install.sh | sh
set -eu

REPO="iwandejong/i"
BIN_DIR="${CDZ_BIN_DIR:-$HOME/.local/bin}"
SHARE_DIR="${CDZ_SHARE_DIR:-$HOME/.local/share/cdz}"

os=$(uname -s)
arch=$(uname -m)

case "$os" in
  Darwin) platform="apple-darwin" ;;
  Linux) platform="unknown-linux-gnu" ;;
  *)
    echo "cdz: unsupported OS '$os' — build from source instead:" >&2
    echo "  https://github.com/$REPO#install" >&2
    exit 1
    ;;
esac

case "$arch" in
  x86_64|amd64) target="x86_64-$platform" ;;
  arm64|aarch64)
    if [ "$os" = "Linux" ]; then
      echo "cdz: no prebuilt Linux arm64 binary yet — build from source:" >&2
      echo "  https://github.com/$REPO#install" >&2
      exit 1
    fi
    target="aarch64-$platform"
    ;;
  *)
    echo "cdz: unsupported architecture '$arch' — build from source instead:" >&2
    echo "  https://github.com/$REPO#install" >&2
    exit 1
    ;;
esac

url="https://github.com/$REPO/releases/latest/download/cdz-$target.tar.gz"
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

echo "Downloading cdz for $target..."
curl -fsSL "$url" -o "$tmp/cdz.tar.gz"
tar xzf "$tmp/cdz.tar.gz" -C "$tmp"

mkdir -p "$BIN_DIR" "$SHARE_DIR"
cp "$tmp/cdz-$target/cdz" "$BIN_DIR/cdz"
chmod +x "$BIN_DIR/cdz"
cp "$tmp/cdz-$target/cdz.zsh" "$SHARE_DIR/cdz.zsh"
echo "Installed $BIN_DIR/cdz"

case ":$PATH:" in
  *":$BIN_DIR:"*) ;;
  *) echo "Note: $BIN_DIR isn't on your PATH yet — add: export PATH=\"$BIN_DIR:\$PATH\"" ;;
esac

source_line="source \"$SHARE_DIR/cdz.zsh\""
case "${SHELL:-}" in
  */zsh)
    rc="$HOME/.zshrc"
    if [ -f "$rc" ] && grep -qF "$SHARE_DIR/cdz.zsh" "$rc" 2>/dev/null; then
      echo "Shell integration already set up in $rc"
    else
      printf '\n# cdz — fuzzy recursive cd (https://github.com/%s)\n%s\n' "$REPO" "$source_line" >> "$rc"
      echo "Added to $rc — restart your shell or run: source $rc"
    fi
    ;;
  *)
    echo "Only zsh integration ships today. Add this to your shell rc file:"
    echo "  $source_line"
    ;;
esac
