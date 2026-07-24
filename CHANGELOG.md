# Changelog

All notable changes to OpenClaw Shell.

## [0.2.5] - 2025-07-24

### Fixed
- `main.rs`: tray icon no longer panics if default window icon is missing (returns clean error instead)
- `installer.rs`: macOS/Linux Node.js extraction now passes paths directly to `tar` without `to_str().unwrap()`, avoiding panic on non-UTF8 paths
- `self_improve.rs`: preference/insight sorting no longer panics on NaN confidence values
- `config.rs`: atomic temp file now includes process ID to prevent collision during concurrent writes
- `Dashboard.tsx`: added missing error handling around `runDoctor` to prevent unhandled promise rejections

## [0.2.4] - 2025-07-24

### Fixed
- `launch_tui` now uses the resolved OpenClaw binary path on all platforms instead of relying on PATH (Windows) or referencing an undefined variable (macOS/Linux)
- Settings updater no longer crashes with a broken Tauri v1 internal API call; instead it shows a friendly "please restart" message
- Removed dead code (unused `_target` in restore, unused `_lesson_text` in pattern matcher)

## [0.2.3] - 2025-07-20

### Fixed
- Synced package.json and Cargo.lock versions

## [0.2.2] - 2025-07-08

### Fixed
- Added frontend build step (`npm run build`) before `cargo check` in CI so Tauri's `frontendDist` validation passes
- Fixed all Rust compiler warnings (unused imports, unused variables, unicode escapes, visibility, trait imports, zip API)
- Bumped `dtolnay/rust-toolchain` action name in release workflow

## [0.2.1] - 2025-07-08

### Fixed
- Fixed `dtolnay/rust-action` -> `dtolnay/rust-toolchain` action reference in CI

## [0.2.0] - 2025-07-07

### Added
- System tray + background mode (window close hides to tray)
- Auto-start on boot toggle
- Light / Dark / Auto theme toggle
- Desktop notifications for gateway events
- Check for Updates button with auto-installer

## [0.1.0] - 2024-XX-XX

### Added
- Initial release
- Setup wizard for first-run installation (Node.js + OpenClaw auto-install)
- Gateway dashboard with start/stop/restart, live logs, and doctor
- API key manager with validation and deep links to providers
- Safety settings: family mode, approval modes, timeout, danger-tool blocking
- Channel guides: Telegram, Discord, Slack setup walkthroughs
- Skill safety review: permission parsing and risk scoring
- Debug panel: system info, doctor output, one-click folder open
- Backup/restore: zip export/import of config
- Self-improvement engine: auto-captures errors, learns preferences, generates insights
- Bidirectional sync with OpenClaw's learning store
- Auto port conflict detection and resolution
- PID ownership tracking to avoid interfering with CLI-started gateways
- Deep-merge config writes to preserve CLI changes
- ZipSlip protection and symlink traversal prevention
- PowerShell escape and --ignore-scripts for safe installs
- 5-minute download timeout with SHA256 verification

## [Unreleased]

- Auto-updater
- System tray minimization
- Dark mode toggle
