#!/usr/bin/env bash
set -e
# Fix Fedora desktop shortcuts for AstralRinth - erases old entries, creates correct Desktop shortcut
DESKTOP_DIR="$(xdg-user-dir DESKTOP 2>/dev/null || echo "$HOME/Desktop")"
BIN_CANDIDATES=(
  "$HOME/.local/bin/ModrinthApp"
  "/usr/bin/ModrinthApp"
  "$HOME/.local/bin/AstralRinth.AppImage"
)
BIN=""
for c in "${BIN_CANDIDATES[@]}"; do if [ -x "$c" ]; then BIN="$c"; break; fi; done
[ -z "$BIN" ] && BIN="$HOME/.local/bin/ModrinthApp"
ICON_CANDIDATES=(
  "$HOME/.local/share/icons/hicolor/128x128/apps/ModrinthApp.png"
  "/usr/share/icons/hicolor/128x128/apps/ModrinthApp.png"
  "$HOME/.local/share/icons/hicolor/128x128/apps/astralrinth.png"
)
ICON="ModrinthApp"
for ic in "${ICON_CANDIDATES[@]}"; do if [ -f "$ic" ]; then ICON="$ic"; break; fi; done

echo "[*] Using BIN=$BIN ICON=$ICON DESKTOP_DIR=$DESKTOP_DIR"

# 1. Erase old desktop entries (case-insensitive astral/modrinth)
echo "[*] Removing old desktop entries..."
shopt -s nullglob nocaseglob
OLD_SYSTEM=(
  /usr/share/applications/*astral*.desktop
  /usr/share/applications/*Astral*.desktop
  /usr/share/applications/*modrinth*.desktop
)
OLD_USER=(
  "$HOME/.local/share/applications/*astral*.desktop"
  "$HOME/.local/share/applications/*Astral*.desktop"
  "$HOME/.local/share/applications/AstralRinth*.desktop"
  "$HOME/.local/share/applications/astralrinth.desktop"
  "$HOME/.local/share/applications/modrinth*.desktop"
)
OLD_DESKTOP=(
  "$DESKTOP_DIR/*astral*.desktop"
  "$DESKTOP_DIR/*Astral*.desktop"
  "$DESKTOP_DIR/astralrinth.desktop"
)
# try system with sudo if available, else warn
for pat in "${OLD_SYSTEM[@]}"; do for f in $pat; do [ -e "$f" ] || continue; echo "  rm $f"; if sudo rm -f "$f" 2>/dev/null; then echo "    -> removed (sudo)"; else echo "    -> need sudo: run 'sudo rm -f \"$f\"' manually"; rm -f "$f" 2>/dev/null || true; fi; done; done
# user files (no sudo needed)
for pat in "${OLD_USER[@]}"; do for f in $pat; do [ -e "$f" ] || continue; echo "  rm $f"; rm -f "$f" || true; done; done
for pat in "${OLD_DESKTOP[@]}"; do for f in $pat; do [ -e "$f" ] || continue; echo "  rm $f"; rm -f "$f" || true; done; done
# also exact known paths with spaces
rm -f "/usr/share/applications/AstralRinth App.desktop" 2>/dev/null || sudo rm -f "/usr/share/applications/AstralRinth App.desktop" 2>/dev/null || true
rm -f "$HOME/.local/share/applications/AstralRinth App.desktop" 2>/dev/null || true
rm -f "$DESKTOP_DIR/AstralRinth App.desktop" 2>/dev/null || true
rm -f "$DESKTOP_DIR/astralrinth.desktop" 2>/dev/null || true

# 2. Ensure binary + icon are in place
mkdir -p "$HOME/.local/bin" "$HOME/.local/share/applications" "$DESKTOP_DIR"
# if BIN is AppImage, add extract flag
EXEC_LINE="$BIN"
if [[ "$BIN" == *.AppImage ]]; then EXEC_LINE="$BIN --appimage-extract-and-run"; fi
# Launcher is 2D UI: render on the integrated GPU with DMABUF hardware
# compositing (fast, smooth). Do NOT offload to the discrete GPU - NVIDIA's
# GBM path fails under XWayland (white window / "Failed to create GBM buffer"),
# and WEBKIT_DISABLE_DMABUF_RENDERER=1 falls back to software SHM (15 FPS).
EXEC_LINE="env GDK_BACKEND=x11 __NV_PRIME_RENDER_OFFLOAD=0 __GLX_VENDOR_LIBRARY_NAME=mesa __GL_SYNC_TO_VBLANK=0 $EXEC_LINE"
# ensure ModrinthApp binary exists from build
if [ ! -x "$HOME/.local/bin/ModrinthApp" ] && [ -f "/home/tentari/rlm-opencode/AstralRinth/target/release/ModrinthApp" ]; then
  echo "[*] Installing fresh binary to ~/.local/bin/ModrinthApp"
  cp "/home/tentari/rlm-opencode/AstralRinth/target/release/ModrinthApp" "$HOME/.local/bin/ModrinthApp"
  chmod +x "$HOME/.local/bin/ModrinthApp"
  BIN="$HOME/.local/bin/ModrinthApp"
  EXEC_LINE="$BIN"
fi

# ensure icon exists
if [ ! -f "$HOME/.local/share/icons/hicolor/128x128/apps/ModrinthApp.png" ] && [ -f "/usr/share/icons/hicolor/128x128/apps/ModrinthApp.png" ]; then
  mkdir -p "$HOME/.local/share/icons/hicolor/128x128/apps" "$HOME/.local/share/icons/hicolor/256x256@2/apps"
  cp "/usr/share/icons/hicolor/128x128/apps/ModrinthApp.png" "$HOME/.local/share/icons/hicolor/128x128/apps/" 2>/dev/null || true
  cp "/usr/share/icons/hicolor/256x256@2/apps/ModrinthApp.png" "$HOME/.local/share/icons/hicolor/256x256@2/apps/" 2>/dev/null || true
  ICON="$HOME/.local/share/icons/hicolor/128x128/apps/ModrinthApp.png"
fi

# 3. Create correct desktop entry
DESKTOP_FILE_CONTENT="[Desktop Entry]
Name=AstralRinth App
Comment=AstralRinth - Modrinth App (fixed)
Exec=${EXEC_LINE}
Icon=${ICON}
Terminal=false
Type=Application
Categories=Game;
StartupNotify=true
StartupWMClass=AstralRinth App
MimeType=application/zip+mrpack;x-scheme-handler/modrinth
Actions=
Keywords=minecraft;modrinth;astralrinth;
"

echo "[*] Creating $HOME/.local/share/applications/AstralRinth App.desktop"
printf "%s" "$DESKTOP_FILE_CONTENT" > "$HOME/.local/share/applications/AstralRinth App.desktop"
chmod +x "$HOME/.local/share/applications/AstralRinth App.desktop"

echo "[*] Creating $DESKTOP_DIR/AstralRinth App.desktop (DESKTOP shortcut)"
printf "%s" "$DESKTOP_FILE_CONTENT" > "$DESKTOP_DIR/AstralRinth App.desktop"
chmod +x "$DESKTOP_DIR/AstralRinth App.desktop"

# 4. Trust the desktop file (GNOME)
if command -v gio >/dev/null 2>&1; then
  echo "[*] Trusting desktop file via gio..."
  gio set "$DESKTOP_DIR/AstralRinth App.desktop" metadata::trusted true 2>/dev/null || true
  gio set "$HOME/.local/share/applications/AstralRinth App.desktop" metadata::trusted true 2>/dev/null || true
fi

# 5. Update caches
echo "[*] Updating desktop databases..."
update-desktop-database "$HOME/.local/share/applications" 2>/dev/null || true
if command -v kbuildsycoca5 >/dev/null 2>&1; then kbuildsycoca5 2>/dev/null || true; fi
if [ -d "$HOME/.local/share/icons/hicolor" ]; then gtk-update-icon-cache -f -t "$HOME/.local/share/icons/hicolor" 2>/dev/null || true; fi
if command -v update-desktop-database >/dev/null 2>&1 && [ -w /usr/share/applications ]; then
  sudo update-desktop-database /usr/share/applications 2>/dev/null || true
fi

echo "[*] Done. Remaining entries:"
ls -la "$HOME/.local/share/applications/"*Astral* 2>&1 || echo "  (none)"
ls -la "$DESKTOP_DIR/"*Astral* 2>&1 || echo "  (none)"
ls -la /usr/share/applications/*Astral* 2>&1 || echo "  system Astral: none (good)"
echo "[*] Validate: desktop-file-validate"
desktop-file-validate "$HOME/.local/share/applications/AstralRinth App.desktop" && echo "  -> valid" || echo "  -> INVALID"
desktop-file-validate "$DESKTOP_DIR/AstralRinth App.desktop" && echo "  -> valid" || echo "  -> INVALID"
echo "[*] If old icon still shows, log out/in or run: killall gnome-shell  (or Alt+F2 -> r)"
