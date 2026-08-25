#!/usr/bin/env bash
# AstralRinth Setup Script
# Removes old installation and installs the latest version from GitHub releases.
# Usage: bash setup.sh [--remove-only | --install-only]
set -euo pipefail

REPO="T3ntari/AstralRith-upgrade"
APP_NAME="AstralRinth"
INSTALL_DIR="$HOME/.local/bin"
DATA_DIR="$HOME/.local/share/AstralRinthApp"
DESKTOP_DIR="$HOME/.local/share/applications"
ICON_DIR="$HOME/.local/share/icons"
DEB_URL=""
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

# ── Detect GPU setup ──────────────────────────────────────────────────────────
detect_nvidia() {
    if command -v nvidia-smi &>/dev/null && nvidia-smi &>/dev/null; then
        return 0
    fi
    return 1
}

# ── Remove old installation ───────────────────────────────────────────────────
remove_old() {
    log "Removing old AstralRinth installation..."

    # AppImage
    if [[ -f "$INSTALL_DIR/$APP_NAME.AppImage" ]]; then
        rm -f "$INSTALL_DIR/$APP_NAME.AppImage"
        ok "Removed old AppImage"
    fi

    # Old helper scripts
    for script in "$HOME/bin/minecraft-dgpu" "$HOME/bin/minecraft"; do
        if [[ -f "$script" ]]; then
            rm -f "$script"
            ok "Removed old script: $script"
        fi
    done

    # Desktop entries
    for entry in "$DESKTOP_DIR/astralrinth.desktop" "$DESKTOP_DIR/AstralRinth App.desktop" \
                 "$DESKTOP_DIR/astralrinth-app.desktop"; do
        if [[ -f "$entry" ]]; then
            rm -f "$entry"
            ok "Removed old desktop entry: $(basename "$entry")"
        fi
    done

    # Old .deb install
    if dpkg -l 2>/dev/null | grep -qi astral; then
        warn "Old .deb package detected — removing..."
        sudo dpkg --remove astral-rinth-app 2>/dev/null || true
        sudo dpkg --remove astralrinth 2>/dev/null || true
        sudo dpkg --remove AstralRinth 2>/dev/null || true
        ok "Removed old .deb package"
    fi

    # Icon
    if [[ -f "$ICON_DIR/astralrinth.png" ]]; then
        rm -f "$ICON_DIR/astralrinth.png"
        ok "Removed old icon"
    fi

    # IMPORTANT: Preserve instance data (Minecraft instances, saves, mods, etc.)
    if [[ -d "$DATA_DIR" ]]; then
        warn "Instance data preserved at: $DATA_DIR"
        warn "Your Minecraft instances will remain after reinstall."
    fi

    ok "Old installation removed (instances preserved)."
}

# ── Install new version ──────────────────────────────────────────────────────
install_new() {
    log "Installing latest AstralRinth from GitHub releases..."

    # Ensure directories exist
    mkdir -p "$INSTALL_DIR" "$DESKTOP_DIR" "$ICON_DIR"

    # Detect OS and select appropriate package
    local os_type="unknown"
    if [ -f /etc/os-release ]; then
        . /etc/os-release
        case "$ID" in
            fedora|rhel|centos|rocky|almalinux) os_type="fedora" ;;
            debian|ubuntu|pop|elementary|zorin) os_type="debian" ;;
            linuxmint|mint) os_type="mint" ;;
            arch|manjaro|endeavouros) os_type="arch" ;;
            *) os_type="unknown" ;;
        esac
    fi

    log "Detected OS type: $os_type"

    # Fetch latest release info
    log "Fetching latest release info..."
    local api_url="https://api.github.com/repos/$REPO/releases/latest"
    local release_json
    release_json=$(curl -fsSL "$api_url" || { err "Failed to fetch release info"; exit 1; })

    local pkg_url=""
    local pkg_type=""

    case "$os_type" in
        fedora)
            pkg_url=$(echo "$release_json" | grep -o '"browser_download_url": *"[^"]*\.rpm"' | head -1 | cut -d'"' -f4)
            pkg_type="rpm"
            ;;
        debian|mint)
            pkg_url=$(echo "$release_json" | grep -o '"browser_download_url": *"[^"]*\.deb"' | head -1 | cut -d'"' -f4)
            pkg_type="deb"
            ;;
        arch)
            # Arch uses AppImage
            pkg_url=$(echo "$release_json" | grep -o '"browser_download_url": *"[^"]*AppImage"' | head -1 | cut -d'"' -f4)
            pkg_type="appimage"
            ;;
        *)
            # Try .deb first, then AppImage
            pkg_url=$(echo "$release_json" | grep -o '"browser_download_url": *"[^"]*\.deb"' | head -1 | cut -d'"' -f4)
            pkg_type="deb"
            if [[ -z "$pkg_url" ]]; then
                pkg_url=$(echo "$release_json" | grep -o '"browser_download_url": *"[^"]*AppImage"' | head -1 | cut -d'"' -f4)
                pkg_type="appimage"
            fi
            ;;
    esac

    if [[ -z "$pkg_url" ]]; then
        err "No suitable package found for $os_type in latest release."
        err "Check: https://github.com/$REPO/releases"
        exit 1
    fi

    local filename
    filename=$(basename "$pkg_url")
    log "Downloading $pkg_type: $filename"
    curl -fSL "$pkg_url" -o "$TMP_DIR/$filename"

    # Install package
    case "$pkg_type" in
        rpm)
            log "Installing RPM via dnf..."
            sudo dnf install -y "$TMP_DIR/$filename"
            ;;
        deb)
            log "Installing DEB via dpkg/apt..."
            sudo dpkg -i "$TMP_DIR/$filename" 2>/dev/null || sudo apt-get install -f -y 2>/dev/null || true
            ;;
        appimage)
            cp "$TMP_DIR/$filename" "$INSTALL_DIR/$APP_NAME.AppImage"
            chmod +x "$INSTALL_DIR/$APP_NAME.AppImage"
            log "AppImage installed to $INSTALL_DIR"
            ;;
    esac

    # Find installed binary
    local bin_path=""
    for candidate in "/usr/bin/astralrinth-app" "/usr/bin/astralrinth" "/usr/local/bin/astralrinth" "$INSTALL_DIR/astralrinth" "$INSTALL_DIR/$APP_NAME.AppImage"; do
        if [[ -x "$candidate" ]]; then
            bin_path="$candidate"
            break
        fi
    done

    # Package manager fallbacks
    if [[ -z "$bin_path" ]]; then
        bin_path=$(dpkg -L astralrinth-app 2>/dev/null | grep -E '/bin/' | head -1 || true)
    fi
    if [[ -z "$bin_path" ]]; then
        bin_path=$(rpm -ql astralrinth-app 2>/dev/null | grep -E '/bin/' | head -1 || true)
    fi

    if [[ -z "$bin_path" ]]; then
        warn "Could not locate installed binary. Falling back to AppImage."
        install_appimage
        return
    fi

    ok "Installed binary: $bin_path"
    create_desktop_entry "$bin_path"

    if [[ -d "$DATA_DIR" ]]; then
        ok "Existing instances preserved at: $DATA_DIR"
    fi
}

# ── Fallback: AppImage install ───────────────────────────────────────────────
install_appimage() {
    log "Installing via AppImage..."
    local api_url="https://api.github.com/repos/$REPO/releases/latest"
    local appimage_url
    appimage_url=$(curl -fsSL "$api_url" | grep -o '"browser_download_url": *"[^"]*AppImage"' | head -1 | cut -d'"' -f4)

    if [[ -z "$appimage_url" ]]; then
        err "No AppImage found in latest release."
        exit 1
    fi

    curl -fSL "$appimage_url" -o "$INSTALL_DIR/$APP_NAME.AppImage"
    chmod +x "$INSTALL_DIR/$APP_NAME.AppImage"
    ok "AppImage installed: $INSTALL_DIR/$APP_NAME.AppImage"

    create_desktop_entry "$INSTALL_DIR/$APP_NAME.AppImage" true
}

# ── Create desktop entry ─────────────────────────────────────────────────────
create_desktop_entry() {
    local bin_path="$1"
    local is_appimage="${2:-false}"
    local exec_line

    # Detect GPU environment
    local gpu_env=""
    local has_nvidia=false
    if command -v nvidia-smi >/dev/null 2>&1 && nvidia-smi >/dev/null 2>&1; then
        has_nvidia=true
    fi

    if [[ "$has_nvidia" == true ]]; then
        gpu_env="env GDK_BACKEND=x11 __NV_PRIME_RENDER_OFFLOAD=0 __GLX_VENDOR_LIBRARY_NAME=mesa __GL_SYNC_TO_VBLANK=0"
    else
        gpu_env="env GDK_BACKEND=x11"
    fi

    if [[ "$is_appimage" == "true" ]]; then
        exec_line="$gpu_env $bin_path --appimage-extract-and-run"
    else
        exec_line="$gpu_env $bin_path"
    fi

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
    ok "Desktop entry created: $DESKTOP_DIR/astralrinth.desktop"
}

# ── Download icon ─────────────────────────────────────────────────────────────
install_icon() {
    if [[ ! -f "$ICON_DIR/astralrinth.png" ]]; then
        log "Downloading icon..."
        curl -fSL "https://raw.githubusercontent.com/$REPO/beta/apps/app/src-tauri/icons/icon.png" \
            -o "$ICON_DIR/astralrinth.png" 2>/dev/null || true
    fi
}

# ── Main ──────────────────────────────────────────────────────────────────────
main() {
    echo ""
    echo -e "${CYAN}╔══════════════════════════════════════╗${NC}"
    echo -e "${CYAN}║     AstralRinth Setup Script         ║${NC}"
    echo -e "${CYAN}╚══════════════════════════════════════╝${NC}"
    echo ""

    local mode="${1:-all}"

    case "$mode" in
        --remove-only)
            remove_old
            ;;
        --install-only)
            install_new
            install_icon
            ;;
        *)
            remove_old
            echo ""
            install_new
            install_icon
            echo ""
            ok "Setup complete! Launch AstralRinth from your application menu."
            echo ""
            ;;
    esac
}

main "$@"
