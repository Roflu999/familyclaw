# Changelog

All notable changes to OpenClaw Shell.

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
