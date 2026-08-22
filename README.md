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

### Linux (Debian / Ubuntu / Linux Mint / Pop!_OS / Zorin)

```bash
# Download the latest .deb
wget https://github.com/T3ntari/AstralRith-upgrade/releases/latest/download/astralrinth-app_*.deb

# Install it (creates a desktop shortcut automatically)
sudo apt install ./astralrinth-app_*.deb

# If you get dependency errors, fix them:
sudo apt --fix-broken install
```

That's it. **AstralRinth** will appear in your app menu as a desktop shortcut — just click to launch.

### Windows

Download the `.msi` from [Releases](https://github.com/T3ntari/AstralRith-upgrade/releases) and run the installer.

### macOS

Download the `.dmg` from [Releases](https://github.com/T3ntari/AstralRith-upgrade/releases), open it, and drag **AstralRinth** to your Applications folder.

### Any Linux (AppImage)

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

---

## Installing on Linux Mint (step by step)

```bash
# 1. Open a terminal

# 2. Download the .deb package
wget https://github.com/T3ntari/AstralRith-upgrade/releases/latest/download/astralrinth-app_*.deb

# 3. Install it (this also creates a desktop shortcut in your app menu)
sudo apt install ./astralrinth-app_*.deb

# 4. If step 3 fails with dependency errors, run:
sudo apt --fix-broken install

# 5. Launch from your desktop menu or run:
ModrinthApp
```

After install, you will find **AstralRinth App** in your desktop applications menu — no terminal needed after setup.

### To uninstall

```bash
sudo apt remove astralrinth-app
```

---

## Available Downloads

| File | Platform |
|---|---|
| `astralrinth-app_*_amd64.deb` | Debian / Ubuntu / Linux Mint / Pop!_OS / Zorin |
| `AstralRinth_*.msi` | Windows |
| `AstralRinth_*.dmg` | macOS (Ventura / Sonoma / Sequoia) |
| `AstralRinth.AppImage` | Any Linux distro |
| `astralrinth-app_*.tar.gz` | Any Linux distro |

> **Warning:** Builds with prefixes `dev`, `nightly`, `dirty`, `dirty-dev`, `dirty-nightly` are unstable and not recommended for daily use.

Download from the **[Releases page](https://github.com/T3ntari/AstralRith-upgrade/releases)**.

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

---

## Disclaimer

AstralRinth is a project intended for experimentation and educational purposes only. It does not endorse or support piracy, and users are encouraged to obtain valid licenses for a fully-supported Minecraft experience. Users are reminded to respect licensing agreements and support the developers of Minecraft.

## License

This project is licensed under the **GNU General Public License v3.0** — see the [LICENSE](LICENSE) file for details.

## Support

- **BTC**: `14g6asNYzcUoaQtB8B2QGKabgEvn55wfLj`
- **USDT TRC20**: `TMSmv1D5Fdf4fipUpwBCdh16WevrV45vGr`
- **TONCOIN**: `UQAqUJ2_hVBI6k_gPyfp_jd-1K0OS61nIFPZuJWN9BwGAvKe`
