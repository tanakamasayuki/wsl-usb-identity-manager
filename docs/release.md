# Releasing

*[English](release.md) | [日本語](release.ja.md)*

Releases are started by hand from the **Actions** tab, not by pushing a tag.

## Cutting a release

1. Actions → **Release** → *Run workflow*.
2. Choose how much to raise the version (`patch`, `minor`, `major`).
3. Leave *dry run* off. With it on the workflow builds and stops — no version
   bump, no tag, no release, no WinGet submission.

The workflow then:

- reads the current version from `[workspace.package]` in the root `Cargo.toml`,
  raises it, writes it back and reads it again to check;
- closes the `## Unreleased` section of `CHANGELOG.md` into `## <version> - <date>`,
  leaving the `## Unreleased` heading in place and empty;
- commits the bump and the changelog, and pushes;
- builds, producing the NSIS installer and a portable ZIP;
- creates the GitHub release `v<version>` at that commit, attaches both, and uses
  the section it just closed as the release notes;
- opens a pull request against `microsoft/winget-pkgs`, if WinGet is set up.

Write what belongs in a release into `## Unreleased` in `CHANGELOG.md` as you go.

### The version lives in one place

`src-tauri/tauri.conf.json` deliberately has **no** `version` field. Tauri falls
back to the version in `Cargo.toml`, so there is one number to raise and nothing
to keep in sync. Do not add it back.

## What gets published

| File | What it is |
| --- | --- |
| `wsl-usb-identity-manager_<version>_x64_setup.exe` | NSIS installer, per-user, no administrator rights |
| `wsl-usb-identity-manager_<version>_x64_portable.zip` | The executable, both READMEs, the licence and `portable.txt` |

`portable.txt` is what makes the application keep `devices.json` and `wuim.log`
beside the executable rather than under `%APPDATA%` and `%LOCALAPPDATA%`.

Builds are **not code-signed**. Windows SmartScreen will warn on first run until
the download builds a reputation.

## Building it yourself

The same thing the release builds can be built locally — to check a change
before releasing it, or to hand someone a build without going through Actions.

### What you need

| | Version | Notes |
| --- | --- | --- |
| Rust | stable (1.98 or later) | `rust-version` in the root `Cargo.toml`; edition 2024 |
| Node.js | 22 | the same as CI |
| Visual Studio Build Tools | — | "Desktop development with C++", for the MSVC toolchain |
| WebView2 runtime | — | already present on Windows 11 |

Windows only. Device enumeration, the usbipd calls and the elevation path have no
counterpart anywhere else, which is why CI runs on `windows-latest` and nothing
else.

### Build

```console
npm ci
npx tauri build
```

`tauri build` runs `npm run build` first (`svelte-check` and the Vite production
bundle), then builds the Rust side in the release profile and packages the NSIS
installer. One command covers a frontend-only change too.

`npm ci` is only needed when the dependencies change. It rebuilds `node_modules`
from scratch, so **it cannot run while `tauri dev` or `tauri build` is running**:
`npx tauri` holds its native module open, and `npm ci` cannot delete it.

```text
npm error code EPERM
npm error syscall unlink
npm error path ...\node_modules\@tauri-apps\cli-win32-x64-msvc\cli.win32-x64-msvc.node
```

Let the build finish first.

| Artefact | Where |
| --- | --- |
| Executable | `target/release/wsl-usb-identity-manager.exe` |
| Installer | `target/release/bundle/nsis/WSL USB Identity Manager_<version>_x64-setup.exe` |

The name differs from the one on the release page because Tauri names the bundle
after `productName`. GitHub turns the spaces into dots when a file is attached to
a release, so the workflow renames it to
`wsl-usb-identity-manager_<version>_x64_setup.exe`.

The portable ZIP is assembled by the workflow; `tauri build` does not produce
one. Putting `portable.txt` beside the executable gives the same behaviour
(R7.2).

A build that fails with **`os error 5` (access denied)** means the application it
is trying to overwrite is still running. Closing the window does not quit it —
quit from the tray icon and run the build again.

### Running it while working on it

```console
npx tauri dev
```

### What to run before committing

```console
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
npm run check
```

CI (`.github/workflows/ci.yml`) runs the same set. It stops short of
`tauri build`: packaging takes about seven minutes and rarely breaks, and when it
does the cost is one re-run of the release workflow rather than a broken release.

## WinGet

### The first version has to be submitted by hand

`winget-releaser` updates a package that already exists in the community
repository; it cannot create one. So the first version goes in manually, once:

```console
winget install Microsoft.WingetCreate
wingetcreate new https://github.com/tanakamasayuki/wsl-usb-identity-manager/releases/download/v<version>/wsl-usb-identity-manager_<version>_x64_setup.exe
```

Answer its questions, then let it submit the pull request. Use
`InstallerType: nullsoft` — WinGet fills in the silent-install switches itself
for Nullsoft installers, which is why the installer is NSIS and not MSI.

These are the values to give it:

```text
PackageIdentifier   tanakamasayuki.WSLUSBIdentityManager
Publisher           TANAKA Masayuki
PackageName         WSL USB Identity Manager
ShortDescription    Track which physical device is which when forwarding USB devices to WSL
```

**The identifier can never change.** Changing it makes a different package, so
whatever is submitted first is what `WINGET_IDENTIFIER` has to match exactly,
for as long as the package exists.

`wingetcreate` asks for all of these rather than reading them: it can extract
metadata from an MSI, but from an EXE installer it only works out the installer
type, the architecture and the hash. Tauri's NSIS template writes no
`CompanyName` into the installer's version info, so there is nothing there for
it to find.

Set `Scope: user` in the installer manifest. The installer is per-user
(`installMode: currentUser`), and without it winget assumes a machine-wide
install and gets upgrades wrong.

Microsoft's validation runs on the pull request and a moderator merges it. That
takes as long as it takes; nothing on this side can hurry it.

### Then let the workflow do it

Once one version is in `winget-pkgs`, set these on the repository and every
later release submits itself:

| Kind | Name | Value |
| --- | --- | --- |
| Variable | `WINGET_IDENTIFIER` | the PackageIdentifier the first submission used |
| Secret | `WINGET_TOKEN` | a **classic** personal access token with `public_repo` scope |

A fine-grained token will not work — `winget-releaser` needs a classic one. The
token forks `microsoft/winget-pkgs` under the account that owns it and opens the
pull request from there.

The WinGet step is skipped while `WINGET_IDENTIFIER` is unset, so the release
workflow is usable before any of this is arranged.

Only the installer is offered through WinGet. The portable ZIP is on the release
page but not in the manifest: WinGet would have to expose a GUI application
through its shim directory, which is the wrong shape for it.

## When the bump cannot be pushed

The workflow commits the version bump with `GITHUB_TOKEN`. If `main` is
protected, that push is rejected and the release fails before building. Either
leave `main` unprotected, allow the token to bypass, or change the workflow to
raise the bump as a pull request.
