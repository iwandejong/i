#!/usr/bin/env sh
# Installs the i binary + shell wrapper from the latest GitHub release.
#   curl -fsSL https://raw.githubusercontent.com/iwandejong/i/main/install.sh | sh
set -eu

REPO="iwandejong/i"
BIN_DIR="${I_BIN_DIR:-$HOME/.local/bin}"
SHARE_DIR="${I_SHARE_DIR:-$HOME/.local/share/i}"

os=$(uname -s)
arch=$(uname -m)

case "$os" in
  Darwin) platform="apple-darwin" ;;
  Linux) platform="unknown-linux-gnu" ;;
  *)
    echo "i: unsupported OS '$os' — build from source instead:" >&2
    echo "  https://github.com/$REPO#install" >&2
    exit 1
    ;;
esac

case "$arch" in
  x86_64|amd64) target="x86_64-$platform" ;;
  arm64|aarch64)
    if [ "$os" = "Linux" ]; then
      echo "i: no prebuilt Linux arm64 binary yet — build from source:" >&2
      echo "  https://github.com/$REPO#install" >&2
      exit 1
    fi
    target="aarch64-$platform"
    ;;
  *)
    echo "i: unsupported architecture '$arch' — build from source instead:" >&2
    echo "  https://github.com/$REPO#install" >&2
    exit 1
    ;;
esac

url="https://github.com/$REPO/releases/latest/download/i-$target.tar.gz"
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

echo "Downloading i for $target..."
curl -fsSL "$url" -o "$tmp/i.tar.gz"
tar xzf "$tmp/i.tar.gz" -C "$tmp"

mkdir -p "$BIN_DIR" "$SHARE_DIR"
cp "$tmp/i-$target/i" "$BIN_DIR/i"
chmod +x "$BIN_DIR/i"
cp "$tmp/i-$target/i.zsh" "$SHARE_DIR/i.zsh"
cp "$tmp/i-$target/i.bash" "$SHARE_DIR/i.bash"
echo "Installed $BIN_DIR/i"

case ":$PATH:" in
  *":$BIN_DIR:"*) ;;
  *) echo "Note: $BIN_DIR isn't on your PATH yet — add: export PATH=\"$BIN_DIR:\$PATH\"" ;;
esac

case "${SHELL:-}" in
  */zsh) rc="$HOME/.zshrc"; script="$SHARE_DIR/i.zsh" ;;
  */bash) rc="$HOME/.bashrc"; script="$SHARE_DIR/i.bash" ;;
  *)
    echo "Only zsh/bash integration ships today. Add one of these to your shell rc file:"
    echo "  source \"$SHARE_DIR/i.zsh\"   # zsh"
    echo "  source \"$SHARE_DIR/i.bash\"  # bash"
    exit 0
    ;;
esac

source_line="source \"$script\""
if [ -f "$rc" ] && grep -qF "$script" "$rc" 2>/dev/null; then
  echo "Shell integration already set up in $rc"
else
  printf '\n# i — fuzzy recursive cd (https://github.com/%s)\n%s\n' "$REPO" "$source_line" >> "$rc"
  echo "Added to $rc — restart your shell or run: source $rc"
fi
