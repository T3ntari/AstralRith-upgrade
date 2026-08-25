<p align="center">
  <img src="https://github.com/DIDIRUS4/AstralRinth/assets/77334306/43d4acb0-546c-4dff-834d-83fb2ba6ad6f" alt="AstralRinth Logo" width="200"/>
</p>

<h1 align="center">AstralRinth</h1>

<p align="center">
  <b>Empowering Your Minecraft Adventure</b><br>
  A fork of <a href="https://github.com/modrinth/code">Modrinth (Theseus)</a> with offline auth, CurseForge support, and more.
</p>

<p align="center">
  <a href="https://github.com/T3ntari/AstralRith-upgrade/releases"><img src="https://img.shields.io/github/v/release/T3ntari/AstralRith-upgrade?style=flat-square&color=blue" alt="Latest Release"></a>
  <a href="https://github.com/T3ntari/AstralRith-upgrade/blob/beta/LICENSE"><img src="https://img.shields.io/github/license/T3ntari/AstralRith-upgrade?style=flat-square&color=green" alt="License"></a>
  <a href="https://github.com/T3ntari/AstralRith-upgrade/releases"><img src="https://img.shields.io/github/downloads/T3ntari/AstralRith-upgrade/total?style=flat-square&color=orange" alt="Downloads"></a>
</p>

---

## Quick Start

### Universal Installer (All Platforms)

**One command works everywhere — Linux (Fedora/Debian/Mint/Arch/macOS/Windows):**

```bash
bash <(curl -fsSL https://raw.githubusercontent.com/T3ntari/AstralRith-upgrade/beta/setup-universal.sh)
```

This auto-detects your OS, downloads the correct package (`.rpm`, `.deb`, `.dmg`, `.msi`, `.AppImage`), installs via native package manager, creates a desktop entry with GPU fixes, and cleans up.

**Options:**
```bash
# Force reinstall (clean old first)
bash <(curl -fsSL https://raw.githubusercontent.com/T3ntari/AstralRith-upgrade/beta/setup-universal.sh) --force

# See what would happen without changes
bash <(curl -fsSL https://raw.githubusercontent.com/T3ntari/AstralRith-upgrade/beta/setup-universal.sh) --dry-run

# Verbose output
bash <(curl -fsSL https://raw.githubusercontent.com/T3ntari/AstralRith-upgrade/beta/setup-universal.sh) -v
```

### Linux — Fedora / RHEL / CentOS / Rocky / AlmaLinux
```bash
# Native RPM (recommended)
sudo dnf install -y https://github.com/T3ntari/AstralRith-upgrade/releases/latest/download/astralrinth-app-*.rpm

# Or use the universal installer above
```

### Linux — Debian / Ubuntu / Linux Mint / Pop!_OS / Zorin / Elementary
```bash
# Native DEB (recommended)
wget -qO /tmp/astralrinth.deb "https://github.com/T3ntari/AstralRith-upgrade/releases/latest/download/astralrinth-app_*_amd64.deb" && sudo apt install -y /tmp/astralrinth.deb && sudo apt --fix-broken install -y && rm /tmp/astralrinth.deb
```

### Linux — Arch / Manjaro / EndeavourOS
```bash
# AppImage (works everywhere)
curl -fSL https://github.com/T3ntari/AstralRith-upgrade/releases/latest/download/AstralRinth.AppImage -o ~/.local/bin/AstralRinth.AppImage && chmod +x ~/.local/bin/AstralRinth.AppImage
```

### macOS (Ventura / Sonoma / Sequoia)
```bash
# Native DMG
curl -fSL https://github.com/T3ntari/AstralRith-upgrade/releases/latest/download/AstralRinth.dmg -o /tmp/AstralRinth.dmg && hdiutil attach /tmp/AstralRinth.dmg -quiet && cp -R /Volumes/AstralRinth/AstralRinth.app /Applications/ && hdiutil detach /Volumes/AstralRinth -quiet && rm /tmp/AstralRinth.dmg
```

### Windows
Download the `.msi` from [Releases](https://github.com/T3ntari/AstralRith-upgrade/releases) and run the installer.

### Any Linux (AppImage Fallback)
```bash
chmod +x AstralRinth.AppImage
./AstralRinth.AppImage
```

---

## What is AstralRinth?

AstralRinth is a specialized fork of the [Modrinth App (Theseus)](https://github.com/modrinth/code) that adds:

| Feature | Description |
|---|---|
| **Offline auth** | Play without a license — login with a pirate account |
| **CurseForge integration** | Browse and install CurseForge modpacks directly |
| **No ads** | All Modrinth advertisements removed |
| **No telemetry** | Statistics collection hard-patched off |
| **Discord RPC** | Rich presence with random messages, play timer, AFK counter |
| **Auto-updates** | Fetches and installs updates from GitHub releases |
| **Custom SVG icons** | Personalized launcher appearance |
| **Launcher VSYNC** | True VSYNC for smooth launcher rendering (toggle in settings) |
| **GPU Selector** | Choose which GPU Minecraft uses (NVIDIA/Integrated/custom PCI) |
| **Efficient Mode** | Disable shadows/blurs/animations for low-end GPUs |
| **GPU JVM args** | G1GC tuned for smooth frame pacing |

---

## Available Downloads

| File | Platform |
|---|---|
| `astralrinth-app_*_amd64.deb` | Debian / Ubuntu / Linux Mint / Pop!_OS / Zorin |
| `astralrinth-app-*.rpm` | Fedora / RHEL / CentOS / Rocky / AlmaLinux |
| `AstralRinth_*.msi` | Windows |
| `AstralRinth_*.dmg` | macOS (Ventura / Sonoma / Sequoia) |
| `AstralRinth.AppImage` | Any Linux distro (Arch, NixOS, etc.) |
| `astralrinth-app_*.tar.gz` | Any Linux distro |

> **Warning:** Builds with prefixes `dev`, `nightly`, `dirty`, `dirty-dev`, `dirty-nightly` are unstable and not recommended for daily use.

Download from the **[Releases page](https://github.com/T3ntari/AstralRith-upgrade/releases)**.

---

## Uninstall

```bash
# Fedora / RHEL / CentOS
sudo rpm -e astralrinth-app

# Debian / Ubuntu / Mint
sudo apt remove astralrinth-app

# AppImage
rm ~/.local/bin/AstralRinth.AppImage ~/.local/share/applications/astralrinth.desktop ~/.local/share/icons/astralrinth.png

# macOS
rm -rf /Applications/AstralRinth.app

# Windows (PowerShell)
msiexec /x "{ProductCode}" /quiet
```

---

## Building from Source

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (latest stable)
- [Node.js](https://nodejs.org/) v20+
- [pnpm](https://pnpm.io/)
- [Turbo](https://turbo.build/)

### Build

```bash
git clone https://github.com/T3ntari/AstralRith-upgrade.git
cd AstralRith-upgrade

pnpm install
pnpm run app:build
cargo build --release -p theseus_gui
```

The built binary will be at `target/release/theseus_gui`.

### Build Packages

```bash
# Debian/Ubuntu/Mint
cargo tauri build --bundles deb

# Fedora/RHEL/CentOS
cargo tauri build --bundles rpm

# AppImage
cargo tauri build --bundles appimage

# All at once
cargo tauri build --bundles deb,rpm,appimage
```

---

## Disclaimer

AstralRinth is a project intended for experimentation and educational purposes only. It does not endorse or support piracy, and users are encouraged to obtain valid licenses for a fully-supported Minecraft experience. Users are reminded to respect licensing agreements and support the developers of Minecraft.

## License

This project is licensed under the **GNU General Public License v3.0** — see the [LICENSE](LICENSE) file for details.

## Support

- **BTC**: `14g6asNYzcUoaQtB8B2QGKabgEvn55wfLj`
- **USDT TRC20**: `TMSmv1D5Fdf4fipUpwBCdh16WevrV45vGr`
- **TONCOIN**: `UQAqUJ2_hVBI6k_gPyfp_jd-1K0OS61nIFPZuJWN9BwGAvKe`