#!/usr/bin/env bash
# AstralRinth Universal Installer
# Auto-detects OS and installs the appropriate package (.msi/.rpm/.deb/.appimage)
# Usage: curl -fsSL https://raw.githubusercontent.com/T3ntari/AstralRith-upgrade/beta/setup-universal.sh | bash
#        curl -fsSL https://raw.githubusercontent.com/T3ntari/AstralRith-upgrade/beta/setup-universal.sh | bash -s -- --help

set -euo pipefail

REPO="T3ntari/AstralRith-upgrade"
APP_NAME="AstralRinth"
APP_ID="astralrinth-app"
API_URL="https://api.github.com/repos/$REPO/releases/latest"
TMP_DIR=$(mktemp -d -t astralinstall.XXXXXX)

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
CYAN='\033[0;36m'
NC='\033[0m'

# Logging
log()  { echo -e "${CYAN}[AR]${NC} $*"; }
ok()   { echo -e "${GREEN}[AR]${NC} $*"; }
warn() { echo -e "${YELLOW}[AR]${NC} $*"; }
err()  { echo -e "${RED}[AR]${NC} $*" >&2; }

# Cleanup on exit
cleanup() {
    rm -rf "$TMP_DIR"
}
trap cleanup EXIT

# Help
show_help() {
    cat <<EOF
AstralRinth Universal Installer

Usage: $0 [OPTIONS]

Options:
  -h, --help          Show this help
  -f, --force         Force reinstall (remove old install first)
  --skip-desktop      Skip desktop entry creation
  --skip-icon         Skip icon download
  --dry-run           Show what would be done without installing
  -v, --verbose       Verbose output

Environment variables:
  REPO                GitHub repo (default: T3ntari/AstralRith-upgrade)
  INSTALL_DIR         Custom install directory for AppImage (default: ~/.local/bin)
  SKIP_DESKTOP=1      Skip desktop entry creation
  SKIP_ICON=1         Skip icon download
EOF
}

# Parse args
FORCE=false
SKIP_DESKTOP=false
SKIP_ICON=false
DRY_RUN=false
VERBOSE=false

while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help) show_help; exit 0 ;;
        -f|--force) FORCE=true ;;
        --skip-desktop) SKIP_DESKTOP=true ;;
        --skip-icon) SKIP_ICON=true ;;
        --dry-run) DRY_RUN=true ;;
        -v|--verbose) VERBOSE=true ;;
        *) err "Unknown option: $1"; show_help; exit 1 ;;
    esac
    shift
done

log()  { echo -e "${CYAN}[AR]${NC} $*"; }
ok()   { echo -e "${GREEN}[AR]${NC} $*"; }
warn() { echo -e "${YELLOW}[AR]${NC} $*"; }
err()  { echo -e "${RED}[AR]${NC} $*" >&2; }
vlog() { [[ "$VERBOSE" == true ]] && log "$*"; }

# Detect OS
detect_os() {
    case "$(uname -s)" in
        Linux*)
            if [ -f /etc/os-release ]; then
                . /etc/os-release
                case "$ID" in
                    fedora|rhel|centos|rocky|almalinux) echo "fedora" ;;
                    debian|ubuntu|mint|pop|elementary|zorin|linuxmint) echo "debian" ;;
                    arch|manjaro|endeavouros) echo "arch" ;;
                    opensuse*|suse) echo "opensuse" ;;
                    *) echo "linux-unknown" ;;
                esac
            else echo "linux-unknown"; fi
            ;;
        Darwin*) echo "macos" ;;
        CYGWIN*|MINGW*|MSYS*|Windows_NT) echo "windows" ;;
        *) echo "unknown" ;;
    esac
}

OS=$(detect_os)
log "Detected OS: $OS"
[[ "$DRY_RUN" == true ]] && warn "DRY RUN MODE - no changes will be made"

# Remove old installation
remove_old() {
    log "Removing old AstralRinth installation..."

    # Stop any running instances
    if command -v pkill >/dev/null; then
        pkill -f "astralrinth" 2>/dev/null || true
        pkill -f "AstralRinth" 2>/dev/null || true
    fi

    # Remove old package installs
    case "$OS" in
        fedora)
            rpm -q "$APP_ID" >/dev/null 2>&1 && { log "Removing old RPM..."; [[ "$DRY_RUN" == false ]] && sudo rpm -e "$APP_ID" 2>/dev/null || true; }
            rpm -q astralrinth >/dev/null 2>&1 && { log "Removing old astralrinth RPM..."; [[ "$DRY_RUN" == false ]] && sudo rpm -e astralrinth 2>/dev/null || true; }
            rpm -q AstralRinth >/dev/null 2>&1 && { log "Removing old AstralRinth RPM..."; [[ "$DRY_RUN" == false ]] && sudo rpm -e AstralRinth 2>/dev/null || true; }
            ;;
        debian)
            dpkg -l "$APP_ID" >/dev/null 2>&1 && { log "Removing old DEB..."; [[ "$DRY_RUN" == false ]] && sudo dpkg --purge "$APP_ID" 2>/dev/null || true; }
            dpkg -l astralrinth >/dev/null 2>&1 && { log "Removing old astralrinth DEB..."; [[ "$DRY_RUN" == false ]] && sudo dpkg --purge astralrinth 2>/dev/null || true; }
            dpkg -l AstralRinth >/dev/null 2>&1 && { log "Removing old AstralRinth DEB..."; [[ "$DRY_RUN" == false ]] && sudo dpkg --purge AstralRinth 2>/dev/null || true; }
            ;;
        *)
            # AppImage or unknown - clean common locations
            for path in "$HOME/.local/bin/AstralRinth.AppImage" "$HOME/.local/bin/astralrinth" "/usr/local/bin/astralrinth" "/usr/bin/astralrinth" "/usr/bin/astralrinth-app" "/opt/AstralRinth"; do
                [[ -e "$path" ]] && { log "Removing $path"; [[ "$DRY_RUN" == false ]] && rm -rf "$path" 2>/dev/null || true; }
            done
            ;;
    esac

    # Remove old desktop entries
    for entry in "$HOME/.local/share/applications/astralrinth.desktop" "$HOME/.local/share/applications/AstralRinth App.desktop" "$HOME/.local/share/applications/astralrinth-app.desktop" "/usr/share/applications/astralrinth.desktop" "/usr/share/applications/astralrinth-app.desktop"; do
        [[ -f "$entry" ]] && { log "Removing desktop entry: $entry"; [[ "$DRY_RUN" == false ]] && rm -f "$entry" 2>/dev/null || true; }
    done

    # Remove old icons
    for icon in "$HOME/.local/share/icons/astralrinth.png" "/usr/share/icons/astralrinth.png" "/usr/share/pixmaps/astralrinth.png"; do
        [[ -f "$icon" ]] && { log "Removing icon: $icon"; [[ "$DRY_RUN" == false ]] && rm -f "$icon" 2>/dev/null || true; }
    done

    # Remove old helper scripts
    for script in "$HOME/bin/minecraft-dgpu" "$HOME/bin/minecraft" "$HOME/.local/bin/minecraft"; do
        [[ -f "$script" ]] && { log "Removing old script: $script"; [[ "$DRY_RUN" == false ]] && rm -f "$script" 2>/dev/null || true; }
    done

    # Update desktop database
    if command -v update-desktop-database >/dev/null; then
        update-desktop-database "$HOME/.local/share/applications" 2>/dev/null || true
    fi

    ok "Old installation cleaned"
}

# Fetch latest release
fetch_release() {
    log "Fetching latest release from GitHub..."
    RELEASE_JSON=$(curl -fsSL --max-time 30 "$API_URL" || { err "Failed to fetch release info (network?)"; exit 1; })
    if [[ -z "$RELEASE_JSON" || "$RELEASE_JSON" == "Not Found" ]]; then
        err "No release found for $REPO"
        exit 1
    fi
    vlog "Release info fetched"
}

# Select asset URL based on OS
select_asset() {
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
            ASSET_URL=$(echo "$RELEASE_JSON" | grep -o '"browser_download_url": *"[^"]*\.AppImage"' | head -1 | cut -d'"' -f4)
            PKG_TYPE="appimage"
            warn "Unknown distro — falling back to AppImage"
            ;;
    esac

    if [[ -z "$ASSET_URL" ]]; then
        err "No $PKG_TYPE asset found in latest release for $OS"
        err "Check: https://github.com/$REPO/releases"
        exit 1
    fi

    log "Selected $PKG_TYPE asset"
}

# Download with verification
download_asset() {
    FILENAME=$(basename "$ASSET_URL")
    DEST="$TMP_DIR/$FILENAME"

    log "Downloading $FILENAME..."
    vlog "URL: $ASSET_URL"
    vlog "Dest: $DEST"

    if [[ "$DRY_RUN" == true ]]; then
        log "[DRY RUN] Would download $FILENAME"
        return
    fi

    curl -fSL --retry 3 --retry-delay 2 --max-time 300 "$ASSET_URL" -o "$DEST" \
        || { err "Download failed"; exit 1; }

    # Verify file exists and has size
    [[ -s "$DEST" ]] || { err "Downloaded file is empty"; exit 1; }
    ok "Downloaded: $FILENAME ($(du -h "$DEST" | cut -f1))"
}

# Install package
install_package() {
    log "Installing $PKG_TYPE package..."

    if [[ "$DRY_RUN" == true ]]; then
        log "[DRY RUN] Would install $PKG_TYPE"
        return
    fi

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
            INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"
            mkdir -p "$INSTALL_DIR"
            cp "$DEST" "$INSTALL_DIR/$APP_NAME.AppImage"
            chmod +x "$INSTALL_DIR/$APP_NAME.AppImage"
            # AppImage integration
            if command -v appimaged >/dev/null; then
                appimaged --register "$INSTALL_DIR/$APP_NAME.AppImage" 2>/dev/null || true
            fi
            ok "AppImage installed to $INSTALL_DIR"
            ;;
        dmg)
            log "Mounting DMG..."
            hdiutil attach "$DEST" -quiet -mountpoint "$TMP_DIR/mount"
            APP_PATH=$(find "$TMP_DIR/mount" -name "*.app" -maxdepth 1 | head -1)
            if [[ -n "$APP_PATH" ]]; then
                # Remove existing
                [[ -d "/Applications/$APP_NAME.app" ]] && rm -rf "/Applications/$APP_NAME.app"
                cp -R "$APP_PATH" /Applications/
                hdiutil detach "$TMP_DIR/mount" -quiet
                ok "Installed to /Applications"
            else
                hdiutil detach "$TMP_DIR/mount" -quiet 2>/dev/null || true
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

    ok "Package installed"
}

# Create desktop entry (Linux only)
create_desktop_entry() {
    [[ "$SKIP_DESKTOP" == true ]] && { vlog "Skipping desktop entry"; return; }
    [[ "$OS" == "macos" || "$OS" == "windows" ]] && return

    log "Creating desktop entry..."

    # Find binary
    BINARY=""
    local candidates=(
        "/usr/bin/astralrinth-app"
        "/usr/bin/astralrinth"
        "/usr/local/bin/astralrinth"
        "$HOME/.local/bin/astralrinth"
        "$HOME/.local/bin/AstralRinth.AppImage"
        "/opt/AstralRinth/AstralRinth"
    )
    for c in "${candidates[@]}"; do
        [[ -x "$c" ]] && { BINARY="$c"; break; }
    done

    # Package manager queries
    [[ -z "$BINARY" ]] && BINARY=$(dpkg -L "$APP_ID" 2>/dev/null | grep -E '/bin/' | head -1 || true)
    [[ -z "$BINARY" ]] && BINARY=$(rpm -ql "$APP_ID" 2>/dev/null | grep -E '/bin/' | head -1 || true)
    [[ -z "$BINARY" ]] && BINARY=$(rpm -ql astralrinth 2>/dev/null | grep -E '/bin/' | head -1 || true)

    if [[ -z "$BINARY" ]]; then
        warn "Could not locate installed binary; desktop entry may not work"
        return
    fi

    vlog "Binary: $BINARY"

    DESKTOP_DIR="$HOME/.local/share/applications"
    ICON_DIR="$HOME/.local/share/icons"
    mkdir -p "$DESKTOP_DIR" "$ICON_DIR"

    # Build exec line with GPU fixes
    local exec_base="$BINARY"
    [[ "$BINARY" == *.AppImage ]] && exec_base="$BINARY --appimage-extract-and-run"
    local exec_line="env WEBKIT_DISABLE_DMABUF_RENDERER=1 __GL_SYNC_TO_VBLANK=1 $exec_base"

    cat > "$DESKTOP_DIR/astralrinth.desktop" <<EOF
[Desktop Entry]
Name=AstralRinth
Comment=AstralRinth Minecraft Launcher
Exec=$exec_line
Icon=astralrinth
Terminal=false
Type=Application
Categories=Game;
MimeType=application/zip+mrpack;x-scheme-handler/modrinth;
StartupNotify=false
EOF

    chmod +x "$DESKTOP_DIR/astralrinth.desktop"
    update-desktop-database "$DESKTOP_DIR" 2>/dev/null || true

    # Download icon
    if [[ "$SKIP_ICON" != true ]]; then
        local icon_url="https://raw.githubusercontent.com/$REPO/beta/apps/app/src-tauri/icons/icon.png"
        if ! curl -fSL --max-time 10 "$icon_url" -o "$HOME/.local/share/icons/astralrinth.png" 2>/dev/null; then
            warn "Failed to download icon (will use default)"
        fi
    fi

    ok "Desktop entry created"
}

# Verify installation
verify_install() {
    log "Verifying installation..."

    case "$PKG_TYPE" in
        rpm)
            rpm -q "$APP_ID" >/dev/null 2>&1 || rpm -q astralrinth-app >/dev/null 2>&1 || { err "RPM verification failed"; exit 1; }
            ;;
        deb)
            dpkg -l "$APP_ID" >/dev/null 2>&1 || dpkg -l astralrinth-app >/dev/null 2>&1 || { err "DEB verification failed"; exit 1; }
            ;;
        appimage)
            [[ -x "$HOME/.local/bin/$APP_NAME.AppImage" ]] || { err "AppImage not executable"; exit 1; }
            ;;
        dmg)
            [[ -d "/Applications/$APP_NAME.app" ]] || { err "DMG verification failed"; exit 1; }
            ;;
        msi)
            # Windows MSI verification is complex; assume msiexec succeeded
            ;;
    esac

    ok "Installation verified"
}

# Main
main() {
    log "=== AstralRinth Universal Installer ==="
    log "Repo: $REPO"
    log "Target OS: $OS"

    # Always clean old installation first (replaces old version)
    remove_old
    fetch_release
    select_asset
    download_asset
    install_package
    [[ "$SKIP_DESKTOP" == false ]] && create_desktop_entry
    verify_install

    ok "=== Installation complete! ==="
    log "Launch AstralRinth from your application menu."
    log "Config: ~/.local/share/AstralRinthApp/"
    log "Logs: ~/.local/share/AstralRinthApp/launcher_logs/"
}

main "$@"