# Releasing a New Version

This project builds automatically on GitHub Actions. You don't need a local Rust/Node setup to publish a release.

## One-Command Release

```bash
git tag v0.1.1
git push origin main --tags
```

That's it. GitHub Actions will:
1. Type-check the frontend
2. Check the Rust code compiles
3. Build the `.msi` installer
4. Build the `.exe` NSIS installer
5. Build and zip the portable version
6. Attach everything to a **draft** release on GitHub

## Publish the Release

1. Go to https://github.com/Roflu999/familyclaw/releases
2. Find the draft release that was just created
3. Review the changelog and assets
4. Click **Publish release**

## Before You Tag

- Update `CHANGELOG.md`
- Bump version in:
  - `package.json`
  - `src-tauri/tauri.conf.json`
  - `src-tauri/Cargo.toml`
- Commit: `git commit -am "Bump version to v0.1.1"`

## Version Numbers

Use [SemVer](https://semver.org/): `vMAJOR.MINOR.PATCH`

Examples:
- `v0.1.0` — first release
- `v0.1.1` — bugfix
- `v0.2.0` — new feature
- `v1.0.0` — stable
