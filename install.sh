#!/bin/bash
set -euo pipefail

REPO="viewerofall/machina"
INSTALL_DIR="${1:-.local/bin}"
INSTALL_PATH="$HOME/$INSTALL_DIR"

echo "🔍 Fetching latest release..."
RELEASE=$(curl -s "https://api.github.com/repos/$REPO/releases/latest")

if ! command -v jq &>/dev/null; then
  echo "❌ jq not found. Install it: sudo pacman -S jq"
  exit 1
fi

VERSION=$(echo "$RELEASE" | jq -r '.tag_name' | sed 's/v//')
BINARY_NAME="machina-$VERSION-x86_64-linux"
DOWNLOAD_URL="https://github.com/$REPO/releases/download/v$VERSION/$BINARY_NAME"

if [ -z "$VERSION" ] || [ "$VERSION" = "null" ]; then
  echo "❌ No releases found for $REPO"
  exit 1
fi

echo "📥 Installing machina $VERSION"

mkdir -p "$INSTALL_PATH"

TMPFILE=$(mktemp)
trap "rm -f $TMPFILE" EXIT

echo "⬇️  Downloading $DOWNLOAD_URL"
if ! curl -fL -o "$TMPFILE" "$DOWNLOAD_URL"; then
  echo "❌ Download failed"
  exit 1
fi

# Verify SHA256 if available
SHA256_URL="https://github.com/$REPO/releases/download/v$VERSION/SHA256SUMS"
if curl -fs "$SHA256_URL" | sha256sum -c - --ignore-missing - <&0 2>/dev/null; then
  echo "✓ Checksum verified"
else
  echo "⚠️  Checksum verification skipped (file not found)"
fi

chmod +x "$TMPFILE"
cp "$TMPFILE" "$INSTALL_PATH/machina"

if [ -f "$HOME/machina/machina.sh" ]; then
  echo "📝 Don't forget to source the cd-on-exit wrapper:"
  echo "   source ~/machina/machina.sh"
fi

echo "✅ machina $VERSION installed to $INSTALL_PATH/machina"
echo "🚀 Run: mc"
