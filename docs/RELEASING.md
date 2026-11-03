# Releasing

The workflow builds, the maintainer publishes. This keeps release creation out of
GitHub Actions (no release-token permissions to fight) and makes the published
assets explicit.

## 1. Prepare

```bash
git checkout master && git pull
# bump the version in package.json, src-tauri/Cargo.toml, src-tauri/tauri.conf.json
# update CHANGELOG.md
bun install
cargo test --all-targets --manifest-path src-tauri/Cargo.toml
cargo clippy --all-targets --manifest-path src-tauri/Cargo.toml -- -D warnings
npx tsc --noEmit && bun run build
git add -A && git commit -m "chore(release): vX.Y.Z"
git push origin master
```

## 2. Build on CI

```bash
git tag -a vX.Y.Z -m "Kensaku vX.Y.Z"
git push origin vX.Y.Z
gh run watch   # or: gh run list --workflow=release.yml
```

`.github/workflows/release.yml` produces artifacts:

| Artifact | Contents |
|---|---|
| `kensaku-linux` | `.deb`, `.rpm`, `.AppImage` |
| `kensaku-macos` | universal `.dmg` / `.app` |
| `kensaku-windows` | `.msi`, NSIS `.exe` |
| `kensaku-cli-{linux,macos,windows}` | `kensaku-cli` binary |

Builds are **unsigned**. See `INSTALL.md` for the user-facing consequences.

## 3. Review and publish the draft

The tag push builds every platform and assembles a **draft** GitHub release with
all bundles, the CLI binaries, and `SHA256SUMS`.

```bash
# wait for the release build
gh run list --workflow=release.yml --limit 1

# review the draft
gh release view vX.Y.Z
gh release download vX.Y.Z -D /tmp/kensaku-release
(cd /tmp/kensaku-release && sha256sum -c SHA256SUMS)
```

Check that the expected assets are present:

| Asset | Platform |
|---|---|
| `kensaku_*_amd64.deb`, `kensaku-*.rpm`, `kensaku_*.AppImage` | Linux |
| `kensaku_*_universal.dmg` | macOS |
| `kensaku_*_x64-setup.exe`, `kensaku_*_x64_en-US.msi` | Windows |
| `kensaku_*_arm64-setup.exe`, `kensaku_*_arm64_en-US.msi`, `kensaku-portable-*-{x64,arm64}.zip` | Windows (x64 and ARM64) |
| `kensaku-cli-linux-*`, `kensaku-cli-macos-*`, `kensaku-cli-windows-*` | Agent CLI |
| `SHA256SUMS` | All assets |

Then publish the draft (or edit the notes first):

```bash
gh release edit vX.Y.Z --draft=false --latest
```

If the workflow could not create the draft, the manual fallback is:

```bash
RUN=$(gh run list --workflow=release.yml --limit 1 --json databaseId --jq '.[0].databaseId')
rm -rf /tmp/kensaku-release && mkdir -p /tmp/kensaku-release
gh run download "$RUN" -D /tmp/kensaku-release
find /tmp/kensaku-release -type f -exec sha256sum {} + > /tmp/kensaku-release/SHA256SUMS
gh release create vX.Y.Z --title "Kensaku vX.Y.Z" --notes-file /tmp/release-notes.md \
  $(find /tmp/kensaku-release -type f)
```

## 4. Verify

- The release page lists all bundles, the CLI binaries, and `SHA256SUMS`.
- `gh release view vX.Y.Z`
- Download one bundle, verify its checksum, install it, and launch.
- Check `kensaku-cli state --json` on a machine with an index.

## Signing / notarization (not yet)

Add the secrets and env vars described in the comment block in
`release.yml`, then re-run the bundle job. macOS notarization needs
`APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD`, `APPLE_SIGNING_IDENTITY`,
`APPLE_ID`, `APPLE_PASSWORD`, `APPLE_TEAM_ID`. Windows needs
`WINDOWS_CERTIFICATE` + password (or a `signtool` step).
