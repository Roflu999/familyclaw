# OpenClaw Shell

A family-friendly, safety-first desktop shell for [OpenClaw](https://github.com/openclaw/openclaw). Built with **Tauri v2** (Rust backend + React frontend).

## Download (No Build Required)

1. Go to [**Releases**](https://github.com/YOUR_USERNAME/openclaw-shell/releases)
2. Pick your flavor:
   - **`.msi`** — Standard install (adds to Start Menu, auto-installs WebView2 if missing)
   - **`.exe`** — Quick installer (same as MSI, smaller download)
   - **`-portable.zip`** — No install at all — unzip to Downloads and double-click
3. Launch "OpenClaw Shell"

> **No Node.js, no npm, no terminal commands.** The installer bundles everything.

## What This Does

OpenClaw Shell wraps the OpenClaw CLI in a GUI so non-developers can:

- **Install** Node.js and OpenClaw automatically (no PATH setup)
- **Control** the gateway with Start/Stop/Restart buttons
- **Manage** API keys with validation and direct links to providers
- **Configure** safety settings (family mode, approval modes) via toggles
- **Set up** messaging channels (Telegram/Discord/Slack) with guided flows
- **Review** skill permissions before enabling them
- **Debug** with one-click system diagnostics
- **Backup/restore** all settings as a zip file

Plus it **self-improves**: it learns from errors, remembers your preferences, and gets smarter over time.

## Build From Source

### Prerequisites

- [Node.js](https://nodejs.org/) 22+
- [Rust](https://rustup.rs/)
- Windows: [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) with "Desktop development with C++"

### Steps

```bash
# 1. Clone
git clone https://github.com/YOUR_USERNAME/openclaw-shell.git
cd openclaw-shell

# 2. Install dependencies
npm install

# 3. Generate icons (first time only)
python3 scripts/generate-icons.py

# 4. Dev mode (hot reload)
npm run tauri dev

# 5. Build release installer
npm run tauri build
```

Output in `src-tauri/target/release/bundle/`:
- `msi/*.msi` — Windows installer
- `nsis/*.exe` — NSIS installer
- `../../openclaw-shell.exe` — Raw executable (for portable zips)

## Project Structure

```
openclaw-shell/
├── src/                          # React frontend
│   ├── App.tsx                    # Main layout + sidebar
│   ├── components/
│   │   ├── Dashboard.tsx        # Gateway control + logs
│   │   ├── ApiKeys.tsx          # Key management
│   │   ├── Channels.tsx         # Channel setup guides
│   │   ├── Skills.tsx           # Skill safety review
│   │   ├── Safety.tsx           # Family mode + approvals
│   │   ├── DebugPanel.tsx       # System diagnostics
│   │   ├── SettingsPanel.tsx    # Backup/restore
│   │   └── SelfImprove.tsx      # Learning dashboard
│   └── main.tsx, styles.css, lib/utils.ts
├── src-tauri/
│   ├── src/                     # Rust backend
│   │   ├── main.rs              # Tauri commands + state
│   │   ├── installer.rs         # Auto-install Node + OpenClaw
│   │   ├── openclaw.rs          # Gateway lifecycle
│   │   ├── config.rs            # Safe JSON config read/write
│   │   ├── api_keys.rs          # Provider key validation
│   │   ├── safety.rs            # Safety settings
│   │   ├── debug.rs             # System info + doctor parser
│   │   ├── backup.rs            # Zip backup/restore
│   │   ├── channels.rs          # Channel setup guides
│   │   ├── skills.rs            # Skill permission review
│   │   └── self_improve.rs      # Learning engine
│   ├── icons/                   # App icons
│   └── tauri.conf.json        # App config
├── scripts/
│   └── generate-icons.py      # Icon generator
├── .github/
│   └── workflows/
│       └── release.yml          # Auto-build on GitHub
└── package.json, Cargo.toml, etc.
```

## How the Self-Improvement Works

Every time something fails — install error, gateway crash, bad API key — the shell captures a **lesson** with:
- What triggered it
- What went wrong  
- How it was fixed
- Whether it can be auto-solved next time

It also learns your **preferences**:
- Which AI provider you use most
- Whether you keep family mode on or off
- Your preferred gateway port

After 3 occurrences of the same error, the shell marks it **auto-solvable** and shows you an **insight** card with a one-click fix.

Lessons sync bidirectionally with OpenClaw's `.learnings/` directory, so both the shell and the CLI learn from each other.

## Release Process

1. Update `CHANGELOG.md`
2. Bump version in `package.json`, `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml`
3. Commit and push:
   ```bash
   git add .
   git commit -m "Release v0.1.0"
   git tag v0.1.0
   git push origin main --tags
   ```
4. GitHub Actions automatically builds the `.msi` and `.exe` and attaches them to a draft release
5. Review and publish the release on GitHub

## License

MIT
