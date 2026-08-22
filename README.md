# Navigation in this README
- [Install instructions](#install-instructions)
- [Features](#features)
- [Getting started](#getting-started)
- [Disclaimer](#disclaimer)
- [Donate](#support-our-project-crypto-wallets)
- [App logo](#logo)

# About Project

## AstralRinth - Empowering Your Minecraft Adventure
Welcome to AR - Fork of Modrinth, the ultimate game launcher designed to enhance your Minecraft experience through the Modrinth platform and their API. Whether you're a graphical interface enthusiast, or a developer integrating Modrinth projects, Theseus core is your gateway to a new level of Minecraft gaming.

## About Software
Introducing AstralRinth, a specialized variant of Theseus dedicated to implementing offline authorization for an even more flexible and user-centric Minecraft Modrinth experience. Roam the Minecraft realms without the constraints of online authentication, thanks to AstralRinth.

## AR - Unlocking Minecraft's Boundless Horizon
Dive into the extraordinary world of AstralRinth, a fork of the original project with a unique focus on providing a free trial experience for Minecraft, all without the need for a license. Currently boasting:

# Install instructions

## Quick Start (Debian / Ubuntu)
```bash
# 1. Download the .deb from https://github.com/T3ntari/AstralRith-upgrade/releases
# 2. Install it:
sudo apt install ./astralrinth-app_*.deb

# That's it. Fix any broken deps with:
sudo apt --fix-broken install
```

## All platforms
Download the correct file for your OS from [releases](https://github.com/T3ntari/AstralRith-upgrade/releases) or [dev builds](https://github.com/T3ntari/AstralRith-upgrade/releases/tag/nightly).

## Linux (Debian / Ubuntu / Linux Mint / Pop!_OS / Zorin)

### .deb package (recommended)
```bash
sudo apt install ./astralrinth-app_*.deb

# Fix broken deps if needed:
sudo apt --fix-broken install

# Uninstall:
sudo apt remove astralrinth-app
```

### AppImage (works on any distro)
```bash
chmod +x AstralRinth.AppImage
./AstralRinth.AppImage

# Move somewhere permanent:
mkdir -p ~/.local/bin
mv AstralRinth.AppImage ~/.local/bin/
```

### .tar.gz
```bash
tar -xzf astralrinth-app_*.tar.gz
cd AstralRinth
./AstralRinth.AppImage
```

## Windows
Download the `.msi` installer and run it.

## macOS
Download the `.dmg` file, open it, and drag AstralRinth to Applications.
Works on Ventura / Sonoma / Sequoia.

### Downloadable file extensions
- `.msi` for Windows
- `.dmg` for macOS
- `.deb` for Debian-based Linux
- `.AppImage` for any Linux
- `.tar.gz` for any Linux

### Installation subjects
- Builds signed with the following prefixes are not recommended for installation:
  - `dev`, `nightly`, `dirty`, `dirty-dev`, `dirty-nightly`, `dirty_dev`, `dirty_nightly`
- Auto-updating works through parsing special versions from releases

# Features

### Featured enhancement in AR
- AstralRinth offers a range of authorization options, giving users the flexibility to log in with valid licenses or even a pirate account without auth credentials breaks. Experience Minecraft on your terms, breaking free from traditional licensing constraints.

### Easy to use
- Using the launcher is intuitive, any user can figure it out.

### Update notifies
- We have implemented notifications about the release of new updates on our Github. The launcher can also download them for you and try to install them.

### Enhancements
- Custom .SVG vectors for a personalized touch.
- Improved compatibility for both pirate and licensed accounts.
- Beautiful Discord RPC with random messages while playing, along with an in-game timer and AFK counter.
- Forced disabling of statistics collection (modrinch metrics) with a hard patch from AstralRinth, ensuring it remains deactivated regardless of the configuration setting.
- Removal of advertisements from all launcher views.
- Optimization of packages (archives).
- Integrated update fetching feature

# Getting Started
To begin your AstralRinth adventure, follow these steps:
1. **Download Your OS Version**: Head over to our [releases page](https://github.com/T3ntari/AstralRith-upgrade/releases/latest) to find the right file for your operating system.
   - **Choosing the Correct File**: Ensure you select the file that matches your OS requirements.
   - [**How select file**](#downloadable-file-extensions)
   - [**How select release**](#installation-subjects)
2. **Authentication**: Log in with a valid license or, for testing, try using a pirate account to see AstralRinth in action.
3. **Launch Minecraft**: Start your journey by launching Minecraft through AstralRinth and enjoy the adventures that await.
   - **Choosing java installation**: The launcher will try to automatically detect the recommended JVM version for running the game, but you can configure everything in the launcher settings.

# Building from source

### Prerequisites
- [Rust](https://www.rust-lang.org/tools/install) (latest stable)
- [Node.js](https://nodejs.org/) v20+
- [pnpm](https://pnpm.io/)
- [Turbo](https://turbo.build/) (installed via pnpm)

### Build the launcher
```bash
git clone https://github.com/T3ntari/AstralRith-upgrade.git
cd AstralRinth

# Install JS dependencies
pnpm install

# Build the frontend
pnpm run app:build

# Build the Rust backend
cargo build --release -p theseus_gui
```

The built binary will be at `target/release/theseus_gui`.

# Disclaimer
- AstralRinth is a project intended for experimentation and educational purposes only. It does not endorse or support piracy, and users are encouraged to obtain valid licenses for a fully-supported Minecraft experience.
- Users are reminded to respect licensing agreements and support the developers of Minecraft.

# License
This project is licensed under the **GNU General Public License v3.0** - see the [LICENSE](LICENSE) file for details.

# Support our Project (Crypto Wallets)
- BTC (Telegram): 14g6asNYzcUoaQtB8B2QGKabgEvn55wfLj
- USDT TRC20 (Telegram): TMSmv1D5Fdf4fipUpwBCdh16WevrV45vGr
- TONCOIN (Telegram): UQAqUJ2_hVBI6k_gPyfp_jd-1K0OS61nIFPZuJWN9BwGAvKe

# Logo
![ar_logo](https://github.com/DIDIRUS4/AstralRinth/assets/77334306/43d4acb0-546c-4dff-834d-83fb2ba6ad6f)
