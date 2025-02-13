#!/bin/sh

# Define variables
REPO="fabiansolheim/zap"
BIN_NAME="zap"
INSTALL_DIR="$HOME/.local/bin"

# Detect architecture
ARCH=$(uname -m)
case "$ARCH" in
    arm64)  TARGET="aarch64-apple-darwin" ;;
    x86_64) TARGET="x86_64-apple-darwin" ;;
    *)      echo "Unsupported architecture: $ARCH"; exit 1 ;;
esac

VERSION=$(curl -sS https://api.github.com/repos/$REPO/releases/latest | grep '"tag_name"' | sed -E 's/.*"tag_name": "([^"]+)".*/\1/')

[ -z "$VERSION" ] && { echo "Failed to fetch the latest release version. Exiting."; exit 1; }

URL=$(curl -sS https://api.github.com/repos/$REPO/releases/latest | grep "browser_download_url" | grep "$TARGET" | cut -d '"' -f 4)

[ -z "$URL" ] && { echo "Error: Could not find a suitable binary for $TARGET."; exit 1; }

echo "Installing $BIN_NAME version $VERSION for macOS ($TARGET)..."
echo "Downloading from: $URL"

TMP_DIR=$(mktemp -d)
cd "$TMP_DIR" || exit 1

if ! curl -sSL -o "$BIN_NAME.tar.gz" "$URL"; then
    echo "Error: Failed to download the binary."
    exit 1
fi

if ! tar -xzf "$BIN_NAME.tar.gz"; then
    echo "Error: Failed to extract the archive."
    exit 1
fi

[ ! -f "$BIN_NAME" ] && { echo "Error: Binary '$BIN_NAME' was not found after extraction."; exit 1; }

chmod +x "$BIN_NAME"

mkdir -p "$INSTALL_DIR"

if [ -f "$INSTALL_DIR/$BIN_NAME" ]; then
    read -p "Warning: $BIN_NAME already exists in $INSTALL_DIR. Overwrite? (y/n): " choice
    case "$choice" in 
        y|Y ) echo "Proceeding with overwrite...";;
        n|N ) echo "Installation aborted."; exit 1;;
        * ) echo "Invalid input. Installation aborted."; exit 1;;
    esac
fi

mv "$BIN_NAME" "$INSTALL_DIR/$BIN_NAME"

xattr -d com.apple.quarantine "$INSTALL_DIR/$BIN_NAME" 2>/dev/null || echo "Skipping quarantine removal."

if ! echo "$PATH" | grep -q "$INSTALL_DIR"; then
    echo "export PATH=\"$INSTALL_DIR:\$PATH\"" >> "$HOME/.zshrc"
    echo "export PATH=\"$INSTALL_DIR:\$PATH\"" >> "$HOME/.bashrc"
    echo "Added $INSTALL_DIR to PATH. Restart your terminal or run: source ~/.zshrc"
fi

echo "$BIN_NAME installed successfully! You can now run it with '$BIN_NAME'."

cd - > /dev/null
rm -rf "$TMP_DIR"
