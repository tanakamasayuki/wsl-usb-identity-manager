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

## 手元でビルドする

リリースと同じものをローカルで作れる。リリース前の確認や、
Actions を通さずに配りたいときはこちら。

### 必要なもの

| | 版 | 備考 |
| --- | --- | --- |
| Rust | stable（1.98 以上） | ルート `Cargo.toml` の `rust-version`。edition 2024 |
| Node.js | 22 | CI と同じ |
| Visual Studio Build Tools | — | 「C++ によるデスクトップ開発」。MSVC ツールチェインが使う |
| WebView2 ランタイム | — | Windows 11 は同梱済み |

Windows 専用である。デバイス列挙も usbipd の呼び出しも昇格の経路も
他の OS に相当物が無いため、CI も `windows-latest` だけで回している。

### ビルド

```console
npm ci
npx tauri build
```

`tauri build` は先に `npm run build`（`svelte-check` + Vite の本番バンドル）を
走らせてから Rust を release プロファイルでビルドし、NSIS インストーラまで作る。
フロントエンドだけを直したときも、この 1 コマンドでよい。

`npm ci` が要るのは依存を入れ直すときだけである。
`node_modules` を丸ごと作り直すため、**`tauri dev` や `tauri build` が
動いている間は実行できない**。`npx tauri` はネイティブモジュールを
読み込んだままなので、それを消せずに落ちる。

```text
npm error code EPERM
npm error syscall unlink
npm error path ...\node_modules\@tauri-apps\cli-win32-x64-msvc\cli.win32-x64-msvc.node
```

先にそちらを終わらせてから実行する。

| 成果物 | 場所 |
| --- | --- |
| 実行ファイル | `target/release/wsl-usb-identity-manager.exe` |
| インストーラ | `target/release/bundle/nsis/WSL USB Identity Manager_<version>_x64-setup.exe` |

Release ページのファイル名と違うのは、Tauri が `productName` で命名するためである。
空白は GitHub が添付時に `.` へ変えてしまうので、ワークフロー側で
`wsl-usb-identity-manager_<version>_x64_setup.exe` に改名している。

ポータブル ZIP はワークフローが組み立てるもので、`tauri build` は作らない。
実行ファイルの隣に `portable.txt` を置けば同じ挙動になる（R7.2）。

**`アクセスが拒否されました (os error 5)`** で失敗したら、
ビルドしたアプリがまだ動いている。ウィンドウを閉じるだけでは終了しないので、
トレイアイコンから終了してから実行し直す。

### 開発中に動かす

```console
npx tauri dev
```

### コミット前に通すもの

```console
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
npm run check
```

CI（`.github/workflows/ci.yml`）がこれと同じものを走らせる。
CI は `tauri build` までは行わない。パッケージングに 7 分ほどかかる割に
壊れることが稀で、壊れたときもリリースワークフローの再実行 1 回で済むためである。

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
