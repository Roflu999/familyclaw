# OpenClaw Shell

A family-friendly, safety-first desktop app for [OpenClaw](https://github.com/openclaw/openclaw). Built with **Tauri v2** (Rust backend + React frontend).

---

## Download & Install (Windows)

**No coding. No terminal. No manual setup.**

1. **Download** the latest `.msi` installer  
   → [github.com/Roflu999/familyclaw/releases](https://github.com/Roflu999/familyclaw/releases)

2. **Double-click** the downloaded file and click **Next** a few times

3. **Launch** "OpenClaw Shell" from your Start Menu or Desktop

4. The built-in **Setup Wizard** installs everything else automatically:
   - Downloads Node.js (the runtime OpenClaw needs)
   - Installs the OpenClaw CLI
   - Sets family-safe defaults
   - Asks for your first API key

> **That's it.** The whole thing takes 2–3 minutes on first launch.

### Alternative Flavors

| File | Best For |
|------|----------|
| **`.msi`** | Standard install — adds to Start Menu, auto-updates WebView2 if missing |
| **`.exe`** | Quick install — smaller download, same end result |
| **`-portable.zip`** | No install at all — unzip to Downloads and double-click `OpenClaw-Shell.exe` |

---

## What Is This?

OpenClaw Shell wraps the OpenClaw AI CLI in a friendly GUI so anyone can use it:

- **One-click** gateway start/stop/restart
- **Safe** family mode with command approvals (on by default)
- **Easy** API key management with built-in provider links
- **Guided** channel setup for Telegram, Discord, Slack
- **Smart** skill permission review before enabling anything risky
- **Self-improving** — learns from errors and gets smarter over time
- **Backup/restore** all settings to a single zip file

---

## First-Run FAQ

**Q: Do I need Node.js installed?**  
A: No. The app downloads and manages its own Node.js automatically.

**Q: Do I need to know how to code?**  
A: No. If you can install Spotify or Discord, you can install this.

**Q: What if I already have OpenClaw installed?**  
A: The app will detect it and use your existing setup. Nothing breaks.

**Q: Where are my settings saved?**  
A: In your home folder at `~/.openclaw/` — the same place the CLI uses, so both stay in sync.

**Q: Is it safe for kids / shared computers?**  
A: Yes. Family mode is on by default. The AI can't run commands without approval.

---

## Build From Source

Only needed if you want to hack on the code.

### Prerequisites

- [Node.js](https://nodejs.org/) 22+
- [Rust](https://rustup.rs/)
- Windows: [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) with "Desktop development with C++"

### Steps

```bash
git clone https://github.com/Roflu999/familyclaw.git
cd familyclaw
npm install
npm run tauri dev        # hot-reload dev mode
npm run tauri build      # produce .msi + .exe
```

Output appears in `src-tauri/target/release/bundle/`.

---

## Project Structure

```
src/                          # React frontend
  App.tsx                     # Main layout + setup wizard
  components/
    Dashboard.tsx             # Gateway control + logs
    ApiKeys.tsx               # Key management
    Channels.tsx              # Channel setup guides
    Skills.tsx                # Skill safety review
    Safety.tsx                # Family mode + approvals
    DebugPanel.tsx            # System diagnostics
    SettingsPanel.tsx         # Backup/restore
    SelfImprove.tsx           # Learning dashboard
src-tauri/
  src/                        # Rust backend
    main.rs                   # Tauri commands + state
    installer.rs              # Auto-install Node + OpenClaw
    openclaw.rs               # Gateway lifecycle
    config.rs                 # Safe JSON config read/write
    api_keys.rs               # Provider key validation
    safety.rs                 # Safety settings
    debug.rs                  # System info + doctor parser
    backup.rs                 # Zip backup/restore
    channels.rs               # Channel setup guides
    skills.rs                 # Skill permission review
    self_improve.rs           # Learning engine
  icons/                      # App icons
  tauri.conf.json             # App config
.github/workflows/release.yml # Auto-build on GitHub
```

---

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

---

## Releasing a New Version

See [RELEASE.md](RELEASE.md) for the one-command release process.

---

## License

MIT
