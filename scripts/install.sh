#!/bin/bash
# OxideKit CLI Installer
# Usage: curl -fsSL https://oxidekit.com/install.sh | bash

set -e

VERSION="${OXIDE_VERSION:-latest}"
INSTALL_DIR="${OXIDE_INSTALL_DIR:-$HOME/.oxide/bin}"

# Detect platform
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$OS" in
  darwin)
    case "$ARCH" in
      arm64|aarch64) TARGET="aarch64-apple-darwin" ;;
      x86_64) TARGET="x86_64-apple-darwin" ;;
      *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
    esac
    ;;
  linux)
    case "$ARCH" in
      x86_64|amd64) TARGET="x86_64-unknown-linux-gnu" ;;
      *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
    esac
    ;;
  *)
    echo "Unsupported OS: $OS"
    echo "For Windows, download from: https://github.com/oxidekit/oxidekit-core/releases"
    exit 1
    ;;
esac

# Get latest version if not specified
if [ "$VERSION" = "latest" ]; then
  VERSION=$(curl -fsSL "https://api.github.com/repos/oxidekit/oxidekit-core/releases/latest" | grep '"tag_name"' | sed -E 's/.*"v([^"]+)".*/\1/')
fi

echo "Installing OxideKit CLI v$VERSION for $TARGET..."

# Create install directory
mkdir -p "$INSTALL_DIR"

# Download and extract
DOWNLOAD_URL="https://github.com/oxidekit/oxidekit-core/releases/download/v$VERSION/oxide-$TARGET.tar.gz"
echo "Downloading from: $DOWNLOAD_URL"

curl -fsSL "$DOWNLOAD_URL" | tar -xz -C "$INSTALL_DIR"

# Verify installation
if [ -x "$INSTALL_DIR/oxide" ]; then
  echo ""
  echo "OxideKit CLI installed successfully!"
  echo ""
  echo "Add to your PATH:"
  echo "  export PATH=\"\$HOME/.oxide/bin:\$PATH\""
  echo ""
  echo "Or add to your shell config (~/.bashrc, ~/.zshrc):"
  echo "  echo 'export PATH=\"\$HOME/.oxide/bin:\$PATH\"' >> ~/.zshrc"
  echo ""
  echo "Then run: oxide --version"
else
  echo "Installation failed"
  exit 1
fi
