#!/usr/bin/env bash
# AstralRinth Universal Installer
# Auto-detects OS and installs the appropriate package (.msi/.rpm/.deb/.appimage)
# Usage: curl -fsSL https://raw.githubusercontent.com/T3ntari/AstralRith-upgrade/beta/setup-universal.sh | bash

set -euo pipefail

REPO="T3ntari/AstralRith-upgrade"
APP_NAME="AstralRinth"
API_URL="https://api.github.com/repos/$REPO/releases/latest"
TMP_DIR=$(mktemp -d)

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
CYAN='\033[0;36m'
NC='\033[0m'

log()  { echo -e "${CYAN}[AR]${NC} $*"; }
ok()   { echo -e "${GREEN}[AR]${NC} $*"; }
warn() { echo -e "${YELLOW}[AR]${NC} $*"; }
err()  { echo -e "${RED}[AR]${NC} $*" >&2; }

cleanup() { rm -rf "$TMP_DIR"; }
trap cleanup EXIT

# ── Detect OS ────────────────────────────────────────────────────────────────
detect_os() {
    case "$(uname -s)" in
        Linux*)
            if [ -f /etc/os-release ]; then
                . /etc/os-release
                case "$ID" in
                    fedora|rhel|centos|rocky|almalinux)
                        echo "fedora"
                        ;;
                    debian|ubuntu|mint|pop|elementary|zorin|linuxmint)
                        echo "debian"
                        ;;
                    arch|manjaro|endeavouros)
                        echo "arch"
                        ;;
                    opensuse*|suse)
                        echo "opensuse"
                        ;;
                    *)
                        echo "linux-unknown"
                        ;;
                esac
            else
                echo "linux-unknown"
            fi
            ;;
        Darwin*)
            echo "macos"
            ;;
        CYGWIN*|MINGW*|MSYS*|Windows_NT)
            echo "windows"
            ;;
        *)
            echo "unknown"
            ;;
    esac
}

OS=$(detect_os)
log "Detected OS: $OS"

# ── Fetch latest release info ────────────────────────────────────────────────
log "Fetching latest release..."
RELEASE_JSON=$(curl -fsSL "$API_URL" || { err "Failed to fetch release info"; exit 1; })

# ── Select asset URL based on OS ─────────────────────────────────────────────
case "$OS" in
    fedora)
        ASSET_URL=$(echo "$RELEASE_JSON" | grep -o '"browser_download_url": *"[^"]*\.rpm"' | head -1 | cut -d'"' -f4)
        PKG_TYPE="rpm"
        ;;
    debian)
        ASSET_URL=$(echo "$RELEASE_JSON" | grep -o '"browser_download_url": *"[^"]*\.deb"' | head -1 | cut -d'"' -f4)
        PKG_TYPE="deb"
        ;;
    arch)
        # Prefer .appimage for Arch (or .pkg.tar.zst if available)
        ASSET_URL=$(echo "$RELEASE_JSON" | grep -o '"browser_download_url": *"[^"]*\.AppImage"' | head -1 | cut -d'"' -f4)
        PKG_TYPE="appimage"
        ;;
    macos)
        ASSET_URL=$(echo "$RELEASE_JSON" | grep -o '"browser_download_url": *"[^"]*\.dmg"' | head -1 | cut -d'"' -f4)
        PKG_TYPE="dmg"
        ;;
    windows)
        ASSET_URL=$(echo "$RELEASE_JSON" | grep -o '"browser_download_url": *"[^"]*\.msi"' | head -1 | cut -d'"' -f4)
        PKG_TYPE="msi"
        ;;
    *)
        # Fallback: AppImage (works everywhere on Linux)
        ASSET_URL=$(echo "$RELEASE_JSON" | grep -o '"browser_download_url": *"[^"]*\.AppImage"' | head -1 | cut -d'"' -f4)
        PKG_TYPE="appimage"
        warn "Unknown distro — falling back to AppImage"
        ;;
esac

if [ -z "$ASSET_URL" ]; then
    err "No suitable asset found for $OS"
    exit 1
fi

log "Found $PKG_TYPE asset: $ASSET_URL"

# ── Download ──────────────────────────────────────────────────────────────────
FILENAME=$(basename "$ASSET_URL")
DEST="$TMP_DIR/$FILENAME"
log "Downloading $FILENAME..."
curl -fSL "$ASSET_URL" -o "$DEST" || { err "Download failed"; exit 1; }
ok "Downloaded: $FILENAME"

# ── Install based on package type ────────────────────────────────────────────
case "$PKG_TYPE" in
    rpm)
        log "Installing RPM via dnf..."
        sudo dnf install -y "$DEST"
        ok "Installed via dnf"
        ;;
    deb)
        log "Installing DEB via dpkg/apt..."
        sudo dpkg -i "$DEST" 2>/dev/null || sudo apt-get install -f -y
        ok "Installed via apt"
        ;;
    appimage)
        INSTALL_DIR="$HOME/.local/bin"
        mkdir -p "$INSTALL_DIR"
        cp "$DEST" "$INSTALL_DIR/$APP_NAME.AppImage"
        chmod +x "$INSTALL_DIR/$APP_NAME.AppImage"
        ok "AppImage installed to $INSTALL_DIR"
        ;;
    dmg)
        log "Mounting DMG and installing..."
        hdiutil attach "$DEST" -quiet -mountpoint "$TMP_DIR/mount"
        APP_PATH=$(find "$TMP_DIR/mount" -name "*.app" -maxdepth 1 | head -1)
        if [ -n "$APP_PATH" ]; then
            cp -R "$APP_PATH" /Applications/
            hdiutil detach "$TMP_DIR/mount" -quiet
            ok "Installed to /Applications"
        else
            err "No .app found in DMG"
            exit 1
        fi
        ;;
    msi)
        log "Installing MSI via msiexec..."
        msiexec /i "$DEST" /quiet /norestart
        ok "Installed via msiexec"
        ;;
esac

# ── Post-install: create desktop entry (Linux only) ─────────────────────────
if [[ "$OS" != "macos" && "$OS" != "windows" ]]; then
    log "Creating desktop entry..."

    # Find installed binary
    BINARY=""
    for candidate in "/usr/bin/astralrinth-app" "/usr/bin/astralrinth" "/usr/local/bin/astralrinth" "$HOME/.local/bin/astralrinth" "$HOME/.local/bin/AstralRinth.AppImage"; do
        if [ -x "$candidate" ]; then
            BINARY="$candidate"
            break
        fi
    done
    [ -z "$BINARY" ] && BINARY=$(dpkg -L astral-rinth-app 2>/dev/null | grep -E '/bin/' | head -1 || true)
    [ -z "$BINARY" ] && BINARY=$(rpm -ql astralrinth-app 2>/dev/null | grep -E '/bin/' | head -1 || true)

    if [ -n "$BINARY" ]; then
        DESKTOP_DIR="$HOME/.local/share/applications"
        mkdir -p "$DESKTOP_DIR"
        ICON_DIR="$HOME/.local/share/icons"
        mkdir -p "$ICON_DIR"

        # Use GPU-friendly exec line (works on NVIDIA + dGPU desktops)
        EXEC_LINE="env WEBKIT_DISABLE_DMABUF_RENDERER=1 $BINARY"

        cat > "$DESKTOP_DIR/astralrinth.desktop" <<EOF
[Desktop Entry]
Name=AstralRinth
Comment=AstralRinth Minecraft Launcher
Exec=$EXEC_LINE
Icon=astralrinth
Terminal=false
Type=Application
Categories=Game;
MimeType=application/zip+mrpack;x-scheme-handler/modrinth;
StartupNotify=false
EOF
        chmod +x "$DESKTOP_DIR/astralrinth.desktop"
        update-desktop-database "$DESKTOP_DIR" 2>/dev/null || true

        # Download icon if missing
        [ ! -f "$ICON_DIR/astralrinth.png" ] && curl -fSL "https://raw.githubusercontent.com/$REPO/beta/apps/app/src-tauri/icons/icon.png" -o "$ICON_DIR/astralrinth.png" 2>/dev/null || true

        ok "Desktop entry created"
    else
        warn "Could not locate binary for desktop entry"
    fi
fi

ok "Installation complete! Launch AstralRinth from your app menu."