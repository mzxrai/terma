use axum::{
    extract::Path,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};

pub async fn install_script(Path(room_id): Path<String>) -> Response {
    let script = format!(r#"#!/bin/bash
set -e

ROOM_ID="{}"

HOST="${{HOST:-localhost:3000}}"
PLATFORM="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

case "$ARCH" in
    x86_64)
        ARCH="x86_64"
        ;;
    arm64|aarch64)
        ARCH="aarch64"
        ;;
    *)
        echo "Unsupported architecture: $ARCH"
        exit 1
        ;;
esac

BINARY_NAME="terma-client-${{PLATFORM}}-${{ARCH}}"
DOWNLOAD_URL="http://${{HOST}}/download/${{BINARY_NAME}}"

echo "Downloading terma client..."
TEMP_FILE=$(mktemp)

if command -v curl > /dev/null 2>&1; then
    curl -sSL "$DOWNLOAD_URL" -o "$TEMP_FILE"
elif command -v wget > /dev/null 2>&1; then
    wget -q "$DOWNLOAD_URL" -O "$TEMP_FILE"
else
    echo "Error: Neither curl nor wget is available"
    exit 1
fi

chmod +x "$TEMP_FILE"

# Move to a permanent location
INSTALL_DIR="${{HOME}}/.local/bin"
mkdir -p "$INSTALL_DIR"
BINARY_PATH="${{INSTALL_DIR}}/terma-client"
mv "$TEMP_FILE" "$BINARY_PATH"

echo ""
echo "✓ Terma client installed to $BINARY_PATH"
echo ""

# Check if ~/.local/bin is in PATH
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo "Note: Add $INSTALL_DIR to your PATH to run 'terma-client' from anywhere:"
    echo "  echo 'export PATH=\"\$HOME/.local/bin:\$PATH\"' >> ~/.bashrc"
    echo "  source ~/.bashrc"
    echo ""
fi

echo "✓ Connecting to room $ROOM_ID..."
echo ""

# Now stdin is the real TTY, so we can exec directly
exec "$BINARY_PATH" "$ROOM_ID"
"#, room_id);

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/x-shellscript")],
        script,
    )
        .into_response()
}
