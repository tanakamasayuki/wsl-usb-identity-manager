# リリース手順

*[English](release.md) | [日本語](release.ja.md)*

リリースはタグ push ではなく、**Actions タブから手動で起動する**。

## リリースする

1. Actions → **Release** → *Run workflow*
2. バージョンの上げ幅を選ぶ（`patch` / `minor` / `major`）
3. *dry run* はオフのまま。オンにするとビルドだけ行って止まる。
   バージョンの更新もタグもリリースも WinGet 提出も行われない。

ワークフローの動作:

- ルート `Cargo.toml` の `[workspace.package]` から現在のバージョンを読み、
  上げて書き戻し、読み直して検証する
- `CHANGELOG.md` の `## Unreleased` の内容を `## <version> - <日付>` へ閉じ、
  `## Unreleased` の見出しは空のまま残す
- バージョンとチェンジログの変更をコミットして push する
- ビルドして NSIS インストーラとポータブル ZIP を作る
- そのコミットに `v<version>` の GitHub Release を作り、両方を添付する。
  リリース本文は上で閉じたチェンジログの節をそのまま使う
- WinGet の設定が済んでいれば `microsoft/winget-pkgs` へ PR を出す

リリースに載せたい変更は、開発中に `CHANGELOG.md` の `## Unreleased` に書いておく。

### バージョンは 1 箇所にしかない

`src-tauri/tauri.conf.json` に `version` フィールドは**意図的に置いていない**。
Tauri は無ければ `Cargo.toml` のバージョンを使うので、上げる数字は 1 つで済み、
食い違いが原理的に起きない。**書き戻さないこと。**

## 公開されるもの

| ファイル | 内容 |
| --- | --- |
| `wsl-usb-identity-manager_<version>_x64_setup.exe` | NSIS インストーラ。per-user、管理者権限不要 |
| `wsl-usb-identity-manager_<version>_x64_portable.zip` | 実行ファイル + README（英日）+ ライセンス + `portable.txt` |

`portable.txt` があると、`devices.json` と `wuim.log` を `%APPDATA%` /
`%LOCALAPPDATA%` ではなく実行ファイルと同じ場所に置く。

**コード署名はしていない。** ダウンロードの実績が積み上がるまで、
初回起動時に Windows SmartScreen の警告が出る。

## WinGet

### 初回だけは手動で提出する

`winget-releaser` は**既にコミュニティリポジトリに存在するパッケージを更新する**
ものであり、新規作成はできない。したがって最初の 1 回だけ手動で登録する。

```console
winget install Microsoft.WingetCreate
wingetcreate new https://github.com/tanakamasayuki/wsl-usb-identity-manager/releases/download/v<version>/wsl-usb-identity-manager_<version>_x64_setup.exe
```

対話に答えて PR を提出させる。`InstallerType` は `nullsoft` を指定する。
WinGet は Nullsoft 指定時にサイレントインストールのスイッチを自動で補うため、
インストーラを MSI ではなく NSIS にしてある。

入力する値は以下。

```text
PackageIdentifier   tanakamasayuki.WSLUSBIdentityManager
Publisher           TANAKA Masayuki
PackageName         WSL USB Identity Manager
ShortDescription    Track which physical device is which when forwarding USB devices to WSL
```

**識別子は後から変更できない。** 変更すると別パッケージ扱いになるため、
最初に提出したものが、パッケージが存在する限り `WINGET_IDENTIFIER` と
完全一致していなければならない値になる。

`wingetcreate` はこれらを読み取らず、対話で尋ねる。MSI からはメタデータを
取り出せるが、EXE インストーラからはインストーラ種別・アーキテクチャ・
ハッシュしか判らない。Tauri の NSIS テンプレートはインストーラの
バージョン情報に `CompanyName` を書かないため、読む対象自体が無い。

installer manifest には `Scope: user` を入れる。
インストーラは per-user（`installMode: currentUser`）であり、
これが無いと winget はマシン全体へのインストールとみなし、更新の扱いを誤る。

PR には Microsoft の検証が走り、モデレータがマージする。所要時間はまちまちで、
こちら側から早める手段はない。

### 以降はワークフローに任せる

`winget-pkgs` に 1 版でも入ったら、リポジトリに以下を設定する。
これ以降のリリースは自動で提出される。

| 種別 | 名前 | 値 |
| --- | --- | --- |
| Variable | `WINGET_IDENTIFIER` | 初回提出で使った PackageIdentifier |
| Secret | `WINGET_TOKEN` | `public_repo` スコープを持つ**クラシック**の PAT |

**fine-grained のトークンは使えない。** `winget-releaser` はクラシック PAT を要求する。
このトークンの所有アカウントの下に `microsoft/winget-pkgs` をフォークし、
そこから PR を出す。

`WINGET_IDENTIFIER` が未設定の間、WinGet のステップは丸ごとスキップされる。
設定前でもリリースワークフローはそのまま使える。

WinGet に載せるのはインストーラだけ。ポータブル ZIP は Release ページには置くが
マニフェストには含めない。GUI アプリを WinGet の Links シムに置く形になり、
筋が悪いため。

## bump が push できない場合

ワークフローはバージョン更新を `GITHUB_TOKEN` でコミットする。`main` に
ブランチ保護をかけていると **この push が弾かれ、ビルド前にリリースが失敗する**。
保護を外す / トークンに bypass を許可する / bump を PR 経由に変える、
のいずれかを選ぶ。
