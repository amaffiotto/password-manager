# How to Release a New Version

## Quick version

```bash
# 1. Bump version numbers
# 2. Commit and tag
# 3. Push — GitHub Actions builds everything automatically

git add -A
git commit -m "v1.x.x"
git tag v1.x.x
git push && git push --tags
```

## Full step-by-step

### 1. Bump the version number in two files

**`package.json`** — line 3:
```json
"version": "1.1.0",
```

**`src-tauri/tauri.conf.json`** — line 3:
```json
"version": "1.1.0",
```

Both must match.

### 2. Update the extension version (if you changed extension code)

**`extension/manifest.json`** — line 4:
```json
"version": "1.1.0",
```

### 3. Test locally

```bash
npm run tauri:dev
```

Make sure the app launches, vault works, settings save, etc.

### 4. Commit everything

```bash
git add -A
git commit -m "v1.1.0"
```

### 5. Tag the release

```bash
git tag v1.1.0
```

Tags must start with `v` (e.g. `v1.0.0`, `v1.1.0`, `v2.0.0`).
GitHub Actions only triggers on tags that match `v*`.

### 6. Push

```bash
git push && git push --tags
```

### 7. Wait

Go to https://github.com/amaffiotto/password-manager/actions and watch the build.
Takes about 10-15 minutes. Four machines build in parallel:
- macOS ARM (Apple Silicon .dmg)
- macOS Intel (.dmg)
- Linux (.AppImage + .deb)
- Windows (.exe installer)

### 8. Done

The Release page is automatically created at:
https://github.com/amaffiotto/password-manager/releases/tag/v1.1.0

All installers are attached. Users can download from there.

---

## Version numbering

Use semantic versioning: `MAJOR.MINOR.PATCH`

- **PATCH** (1.0.0 → 1.0.1): Bug fixes, small tweaks
- **MINOR** (1.0.0 → 1.1.0): New features, non-breaking changes
- **MAJOR** (1.0.0 → 2.0.0): Breaking changes, major redesigns

---

## If something goes wrong

### Build failed on GitHub Actions
- Go to Actions tab, click the failed run, read the error log
- Fix the issue, commit, delete the tag, re-tag, push:
  ```bash
  git tag -d v1.1.0
  git push origin --delete v1.1.0
  # fix the issue, commit
  git tag v1.1.0
  git push && git push --tags
  ```

### Need to update a release after publishing
- Go to the release page on GitHub
- Click "Edit release"
- You can change the description, upload new files, or delete old ones

### Want to test the build without releasing
- Use a pre-release tag: `v1.1.0-beta`
- Or build locally: `npm run tauri:build`

---

## Chrome Web Store updates

The extension is separate from the desktop app releases.

1. Bump `version` in `extension/manifest.json`
2. Zip the extension folder: `cd extension && zip -r ../extension.zip . && cd ..`
3. Upload to https://chrome.google.com/webstore/devconsole
4. Submit for review (takes 1-3 days)
