# Releasing

*[English](release.md) | [日本語](release.ja.md)*

Releases are started by hand from the **Actions** tab, not by pushing a tag.

## Cutting a release

1. Actions → **Release** → *Run workflow*.
2. Choose how much to raise the version (`patch`, `minor`, `major`).
   The first release should be `minor`, which takes `0.0.0` to `0.1.0`.
3. Leave *dry run* off. With it on the workflow builds and stops — no version
   bump, no tag, no release, no WinGet submission.

The workflow then:

- reads the current version from `[workspace.package]` in the root `Cargo.toml`,
  raises it, writes it back and commits the change;
- builds, producing the NSIS installer and a portable ZIP;
- creates the GitHub release `v<version>` at that commit and attaches both;
- opens a pull request against `microsoft/winget-pkgs`, if WinGet is set up.

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
