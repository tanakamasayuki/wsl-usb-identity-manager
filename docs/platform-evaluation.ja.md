# 言語・フレームワーク・配布方式の評価

作成日: 2026-09-02
前提: [docs/research-findings.ja.md](research-findings.ja.md) の調査結果

---

## 1. 評価軸

要件から導出した評価軸を、重み順に並べる。

| # | 軸 | 由来 |
| --- | --- | --- |
| A1 | **プローブ実装を再発明しないこと** | 本計画で最も難しく、最も device-specific な部分（F3・F8） |
| A2 | **ネイティブ Windows API に素直に触れること** | CfgMgr32 直叩き必須（F7）、WinUSB 必須（F6） |
| A3 | **インストーラ + 自動更新 + ポータブル ZIP を出せること** | WinGet 配布要件 |
| A4 | **GitHub Actions でビルド・リリースを完全自動化できること** | OSS 公開要件 |
| A5 | 常駐アプリとして軽いこと（起動時間・メモリ・バイナリサイズ） | トレイ常駐でホットプラグ監視 |
| A6 | 保守者（1 名）が継続できること | 既存資産との整合 |

---

## 2. プローブ実装の所在（A1 の実態）

識別プローブは 2 系統しか要らない。それぞれ **既存ライブラリがどの言語にあるか** を確認した。

| プローブ対象 | 必要な処理 | Rust | Python | C# |
| --- | --- | --- | --- | --- |
| WCH-Link / LinkE + その先の CH32 | ベンダ USB プロトコル、チップ署名、UUID | **`ch32rv` の `crates/wchlink` `crates/usb` `crates/target`（自作・既存）**、`wlink` | board-identify（自作・既存） | なし（フルスクラッチ） |
| ESP32 系（CH340 等の裏） | ROM ブートローダ通信、eFuse MAC 読み出し | **`espflash` crate（esp-rs 公式）** | `esptool`（Espressif 公式） | なし（フルスクラッチ） |
| USB 生アクセス | WinUSB | **`nusb`（純 Rust、libusb 不要）** | `pyusb` + libusb-1.0.dll 同梱 | WinUSB P/Invoke（自作） |

### 重要な前提の訂正

当初は「board-identify（Python）にプローブ実装があるから Python が有利」と評価していたが、
`ch32rv` の存在によりこれは成立しない。

`ch32rv` は既に以下を **再利用可能なライブラリ crate として分離済み**である。

```text
crates/wchlink/src/probe.rs   26,728 bytes
crates/usb/src/device.rs       9,762 bytes
crates/usb/src/selector.rs     9,537 bytes
crates/target/src/lib.rs      13,002 bytes  + generated/skus.csv 他
docs/protocol/wch-link.ja.md  25,327 bytes  （プロトコル仕様書）
```

つまり **Rust を選ぶと、WCH 系プローブは「自作の既存 crate への依存」で済み、新規実装がゼロになる。**
ESP32 系は `espflash` crate（サードパーティだが esp-rs 公式）で賄える。

→ **Rust なら、プローブ実装の新規開発も二重保守も発生しない。**

C# を選ぶと `ch32rv` の成果が一切使えず、WCH プロトコルをフルスクラッチで書くことになる。
これは本計画で最も重い作業であり、C# は脱落する。

### board-identify（Python）との関係

board-identify は **WSL（Linux）側で動く別プロセス・別インストール**である。
Windows GUI と Python パッケージを共有しても、

- board-identify 側に Windows プラットフォームバックエンド（CfgMgr32）を足す必要がある
- Windows GUI のパッケージに Python ランタイムを同梱する必要がある（A3・A5 に直撃）

というコストが発生し、共有の利得を上回る。

→ **共有すべきなのはコードではなく「識別子フォーマット」という契約。**
`esp32-s3-7cdfa1123456` のような文字列仕様をドキュメントとテストで固定し、
実装は各 OS のツールが独立して持つ。
`ch32rv` が `docs/naming.ja.md` と `docs/contract/*.schema.json` で
既に同じやり方を採っているので、運用方法も揃う。

---

## 3. Windows API アクセス（A2 の実態）

| 用途 | 使う API | Rust での手段 |
| --- | --- | --- |
| デバイス列挙・プロパティ | `CM_Get_Device_ID_ListW`, `CM_Get_DevNode_PropertyW` | `windows` crate（Microsoft 公式バインディング） |
| ホットプラグ通知 | `CM_Register_Notification` または `WM_DEVICECHANGE` | 同上 |
| COM ポート特定 | レジストリ `Device Parameters\PortName` | `windows` crate / `winreg` |
| WinUSB I/O | WinUSB API | `nusb`（純 Rust、libusb DLL 不要） |
| シリアル I/O | `CreateFile` + DCB | `serialport` crate |
| 昇格実行 | `ShellExecuteW` verb=`runas` | `windows` crate |

### nusb の Windows 制約（実装方針に影響）

nusb のドキュメントに以下の制約がある。

> on Windows, serial numbers are only available for composite devices bound to the usbccgp driver,
> and will be empty if the entire device is bound to a specific driver.

CH340 は `CH341SER_A64` に丸ごとバインドされるため、**nusb からはシリアルが取れない。**

→ **役割分担を明確にする。**

- **列挙・識別情報の取得は CfgMgr32 が担当**（Instance ID・LocationPaths・ContainerId・Service）
- **nusb は WinUSB プローブの I/O にだけ使う**（WCH-Link の interface 0）

この分担なら nusb の制約は問題にならない。むしろ CfgMgr32 の方が情報量が多い（F3）。

---

## 4. 言語・フレームワーク候補の評価

| 軸 | Rust + Tauri 2 | Rust + egui/slint | Python + PySide6 | C# + WPF |
| --- | --- | --- | --- | --- |
| A1 プローブ再利用 | ◎ ch32rv + espflash | ◎ 同左 | △ board-identify に Windows 対応追加が必要 | ✗ フルスクラッチ |
| A2 Windows API | ◎ windows crate | ◎ 同左 | ○ ctypes（170ms 実証済み） | ◎ CsWin32 |
| A3 インストーラ/更新/ポータブル | ◎ NSIS+MSI+updater 内蔵 / ポータブルは要工夫 | ○ 単一 exe = ポータブル自明 / 他は自作 | △ PyInstaller、AV 誤検知が実務的な壁 | ○ Velopack が優秀 |
| A4 CI 自動化 | ◎ `tauri build` が NSIS まで生成 | ○ 素の cargo + gh release | △ OS 別ランナー + 署名必須 | ○ |
| A5 常駐時の軽さ | ○ ~10MB / WebView2 依存 | ◎ ~15MB 単一 exe / 依存なし | ✗ ~100MB / 起動 1 秒 | ○ ~70MB(self-contained) |
| A6 継続性 | ◎ ch32rv と同じ言語・同じ CI 作法 | ◎ 同左 | ○ board-identify と同じ言語 | △ 新規言語 + 新規実装 |
| UI 実装コスト（表・詳細ペイン・トレイ） | ◎ HTML/CSS、日本語フォントは OS 任せ | △ 表・ツリーは自作、CJK フォント同梱が必要 | ◎ Qt の TableView | ◎ |

### 評価

- **C# は A1 で脱落。** ch32rv の資産が使えず、最重量の作業を新規に抱える。
- **Python は A3・A5 で脱落。** WinGet + 自動更新 + ポータブルという要件に対し、
  PyInstaller ベースの配布は最も摩擦が大きい（サイズ、起動時間、AV 誤検知）。
  board-identify との言語一致という利点は、§2 の通り実利が薄い。
- **Rust の 2 案が残る。** 差は GUI レイヤと配布形態のみ。

---

## 5. 推奨

> **Rust + Tauri 2 を第一候補とする。**

理由の要約。

1. `ch32rv` の crate をそのまま依存にでき、WCH-Link プローブの新規実装・二重保守がゼロになる
2. ESP32 プローブも `espflash` crate で賄え、こちらも新規実装がゼロ
3. `windows` crate + `nusb` で必要な Windows API に過不足なく届く
4. `tauri build` が NSIS バンドルまで作るため、CI は素の GitHub Actions で完結する
5. デバイス一覧・詳細ペイン・設定画面という UI は HTML/CSS が最も安く、
   日本語表示は OS のフォントに任せられる
6. ch32rv と言語・ツールチェイン・CI 作法が揃うため、保守者 1 名でも運用が二重化しない

### 採用しない案とその理由

- **egui / slint**: 単一 exe でポータブル配布は自明だが、
  デバイス一覧の表・詳細ペイン・設定 UI を全て自作することになり、
  CJK フォントの同梱も必要。UI 実装コストが Tauri より明確に高い。
  ただし **WebView2 依存を避けたい場合の代替として残す**（§6 の論点）。

---

## 6. パッケージング・配布の検討材料

要件は「WinGet 配布」「ポータブル ZIP」「GitHub Actions で自動化」。
本節は判断の材料となる事実であり、**結論は §9 の決定事項にある。**

### 6.1 WinGet が受け付ける形式

`InstallerType` に指定できる主な値:
`msi` / `wix` / `nullsoft` / `inno` / `exe` / `msix` / `zip` / `portable` / `font`

- `zip` を使う場合は `NestedInstallerType` の指定が必須
- `NestedInstallerType: portable` のときだけ、アーカイブ内に複数ファイルを置ける
- MSI は `ProductCode` をマニフェストに書くと `winget upgrade` の検出精度が上がる
- Nullsoft / Inno を指定すると winget がサイレントスイッチを自動設定する
- **サイレントインストール対応は community repo の必須条件**

→ **NSIS(`nullsoft`) と ZIP ポータブルの 2 本立てが、要件に素直に対応する。**

### 6.2 Tauri が出せる成果物

| 形式 | Tauri での扱い |
| --- | --- |
| NSIS `-setup.exe` | ◎ 標準。**per-user インストール（管理者権限不要）** |
| MSI (WiX) | ◎ 標準。per-machine インストール |
| 自動更新 | ◎ `tauri-plugin-updater`。NSIS/MSI 両対応 |
| ポータブル ZIP | **△ 正式サポートなし。要工夫**（§6.4） |

### 6.3 自動更新の署名と、WinGet との衝突

Tauri updater の仕様:

- **署名は必須で無効化できない。** 秘密鍵を失うと以後の更新を配信できなくなる
  → GitHub Actions の Secrets に置き、鍵のバックアップ手順を運用に含める必要がある
- `latest.json` に `version` / `platforms.<OS-ARCH>.url` / `platforms.<OS-ARCH>.signature` が必要
- `.sig` は**ファイルの中身をそのまま埋める**（URL では不可）
- インストール実行時に **アプリが強制終了される**（Windows インストーラの制約）

**WinGet と自己更新の併存は設計上の論点になる。**
WinGet でインストールしたものをアプリが自己更新すると、
winget 側の記録とインストール実体がズレる。よくある折衷案は 3 つ。

| 案 | 内容 | 評価 |
| --- | --- | --- |
| a | インストール経路を検出し、WinGet 経由なら自己更新を無効化して通知のみ | 挙動が正しいが検出ロジックが必要 |
| b | 自己更新は行わず、常に「新版があります」の通知とリリースページ誘導のみ | 実装が最も軽い。WinGet と競合しない |
| c | per-user NSIS のみ配布し、自己更新を正とする | winget upgrade と二重管理になる |

→ **自動更新そのものを実装しないこと**で、この論点ごと消滅した（D3）。
更新は `winget upgrade` または ZIP の差し替えによる。

### 6.4 ポータブル ZIP の実現方法

Tauri はポータブル形式を正式サポートしていない。取りうる手段:

| 手段 | 内容 | 難点 |
| --- | --- | --- |
| 1 | ビルド成果の `app.exe` + リソースを ZIP に固める | WebView2 ランタイム依存が残る |
| 2 | ロジックを CLI バイナリに分離し、ポータブル版は CLI のみ | GUI がポータブルにならない |
| 3 | GUI を egui/slint にして単一 exe にする | Tauri を捨てる（§5 の代替案） |

**WebView2 の実態**: Windows 11 にはプリインストール済み。
Windows 10 でも Edge 更新経由でほぼ導入済み。
NSIS インストーラにはブートストラッパを埋め込めるが、**ポータブル版には埋め込めない**。

→ 手段 1 を採り、ポータブル版は「WebView2 が入っている環境向け」と割り切る（D2）。
起動時に検出し、無ければ導入先を案内して終了する（§9.4）。

### 6.5 設定の保存場所

ポータブル版を出すなら、設定ファイルの位置に方針が要る。

| 案 | 内容 |
| --- | --- |
| a | 常に `%APPDATA%\<app>\` | ポータブル版でも痕跡が残る |
| b | exe と同じディレクトリに `portable.txt` があればそこに保存、無ければ `%APPDATA%` | 一般的なポータブル規約 |

→ b を採用する（D5）。

### 6.6 コード署名

WinGet 配布そのものに署名は必須ではないが、無署名だと SmartScreen 警告が出る。
Tauri updater の署名（`.sig`）とは別物である点に注意。

- Authenticode 証明書の取得は有償かつ本人確認が必要
- 無署名で出し、SmartScreen 評価が溜まるのを待つ選択も現実的

→ 署名しない（D6）。

---

## 7. 権限設計（言語非依存だが配布形態に影響する）

調査結果 F5 より:

- `usbipd bind` / `unbind` / `policy` : **管理者権限が必要**
- `usbipd attach` / `detach` / `state` : 一般ユーザーで可

→ **GUI 本体は非昇格で起動する。**
bind が必要になった場面でのみ `ShellExecuteW(verb="runas")` で `usbipd.exe` を昇格実行し、
UAC ダイアログを出す。GUI 全体を管理者で走らせない。

さらに、初期設定で `usbipd policy add`（AutoBind）を一度だけ設定してもらえば、
日常運用では bind 操作そのものが不要になり、昇格の頻度をほぼゼロにできる。

これは **per-user NSIS インストール（管理者権限不要）と整合する**ため、
配布形態の選択にも効いてくる。

---

## 8. WebView2 の実際の普及状況（D1・D2 の判断材料）

Microsoft 公式ドキュメント（[Distribute your app and the WebView2 Runtime](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/distribution)）より。

| 環境 | 状況 |
| --- | --- |
| **Windows 11** | **OS に同梱**（"The Evergreen WebView2 Runtime will be included as part of the Windows 11 operating system."） |
| **Windows 10** | **「大多数の端末に導入済み」と Microsoft が明記**。2022 年 12 月から管理対象 Windows 10 へ配信 |
| 例外 | "A small number of Windows 10 devices don't have the WebView2 Runtime installed. We recommend that you handle this edge case." |

想定される例外環境: Windows Server、LTSC、Windows Update から切り離された端末、
企業のクリーンイメージ直後。

### 検出方法（実機で確認済み）

```text
HKLM\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}
  pv (REG_SZ)      per-machine インストール
HKCU\Software\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}
  pv (REG_SZ)      per-user インストール
```

いずれかが存在し `0.0.0.0` 以外であれば導入済み。
本調査環境での実測値: `name=Microsoft Edge WebView2 Runtime`, `pv=151.0.4129.107`（HKLM）。

### 導入手段

- **Evergreen Bootstrapper**: 約 2 MB。`MicrosoftEdgeWebview2Setup.exe /silent /install`
- Standalone Installer: オフライン環境向けのフルインストーラ
- Fixed Version: 250 MB 超。本アプリでは採用しない

→ **結論**: 欠落は実質的にレアケースであり、かつ **レジストリ 1 個で確実に検出できる**。
ポータブル版でも「起動時に検出し、無ければダウンロードページを案内する」で十分に扱える。
**WebView2 依存は Tauri を採用しない理由にはならない。**

---

## 9. 決定事項

「自動更新は無しでよい」「なるべくシンプルに公開したい」「ポータブル版は作る」
という方針を受けての決定。

| # | 論点 | 決定 |
| --- | --- | --- |
| D1 | GUI レイヤ | **Tauri 2**（§9.1） |
| D2 | ポータブル ZIP の WebView2 依存 | **許容する。**起動時に検出し、無ければ案内（§8） |
| D3 | 自動更新 | **実装しない。**更新は `winget upgrade` または ZIP 差し替え |
| D4 | インストーラ形態 | **NSIS（per-user、管理者権限不要）のみ。**MSI は出さない |
| D5 | 設定の保存場所 | exe と同じ場所に `portable.txt` があればそこ、無ければ `%APPDATA%` |
| D6 | コード署名 | **行わない。**無署名で公開し、必要になった時点で再検討 |

### 9.1 自動更新を外したことによる GUI レイヤの再評価

自動更新を実装しないため、Tauri の優位点だった `tauri-plugin-updater`
（署名鍵管理・`latest.json`・WinGet との衝突）は **論点ごと消滅した**。
これにより egui / slint との差が縮まったため、改めて比較した。

| | Tauri 2 | egui (eframe) |
| --- | --- | --- |
| ポータブル | ZIP 化は可。WebView2 依存あり | 単一 exe。依存なし |
| 一覧表・詳細ペイン・設定画面 | HTML/CSS。実装が最も安い | `egui_extras::TableBuilder` 等で自作 |
| 日本語表示 | OS のフォントを使用。追加作業なし | **CJK フォントの同梱が必要**（Alias/Memo に任意の漢字が入るため subset 不可、約 5〜8 MB） |
| 日本語 IME 入力 | **WebView2 = OS のテキスト入力そのもの。確実** | 実装済みで改善も続くが、[トラッキング issue #248](https://github.com/emilk/egui/issues/248) は長期オープン |
| ツールチェイン | Rust + Node（フロントエンドビルド） | Rust のみ |
| バンドル | `tauri build` が NSIS インストーラを生成 | `cargo build` の成果物をそのまま ZIP |

**Tauri を採用する。** 決め手は 2 点。

1. **UI 実装コスト**が本アプリの開発量の大半を占める（デバイス一覧・詳細・設定・トレイ）。
   HTML/CSS はここが最も安い。
2. **Alias / Memo への日本語入力は本アプリの主要機能**である。
   WebView2 は OS のテキスト入力機構そのものなので IME が確実に動く。
   egui の IME は動作するが、確実性で WebView2 に及ばない。

egui の利点（単一 exe・フォント不要・Rust のみ）は、
§8 の通り WebView2 の欠落がレアかつ検出可能であるため、決定打にならない。

### 9.2 配布物

リリースごとに **2 種類だけ**を出す。

```text
wsl-usb-identity-manager_<version>_x64_setup.exe    NSIS / per-user / 管理者権限不要
wsl-usb-identity-manager_<version>_x64_portable.zip ポータブル（portable.txt 同梱）
```

- **MSI は出さない。** per-machine と per-user が併存すると更新経路が二重化するため
- **WinGet に登録するのは NSIS のみ**（`InstallerType: nullsoft`）。
  WinGet は Nullsoft 指定時にサイレントスイッチを自動設定する
- ポータブル ZIP は GitHub Release で直接配布する。
  WinGet の `zip` + `NestedInstallerType: portable` でも登録可能だが、
  GUI アプリを WinGet の Links シムに置く形になり筋が悪いため見送る

### 9.3 CI / リリース自動化

**リリースはタグ push ではなく、GitHub Actions の画面から手動で起動する。**
起動時に bump レベル（major / minor / patch）を指定する。

```text
workflow_dispatch  （Actions 画面で bump: major|minor|patch を選択）
   ↓
Cargo.toml の version を bump して書き戻し、読み直して検証
   ↓
CHANGELOG.md の ## Unreleased を ## <version> - <date> へ閉じる
   ↓
bump とチェンジログをコミット & push
   ↓
npx tauri build            NSIS インストーラを生成
   ↓
成果物を配布名へ改名し、ポータブル ZIP を作る
   ↓
gh release create          タグを打ち、Release を作成（本文は上で閉じた節）
   ↓
vedantmgoyal9/winget-releaser   winget-pkgs へ PR を自動作成
```

自動更新を実装しないため、**署名鍵の管理も `latest.json` の生成も不要**。

#### バージョンの単一情報源

Tauri の公式ドキュメントより:

> `version`: App version. ... **If removed the version number from `Cargo.toml` is used.**

→ `tauri.conf.json` の `version` は **書かない**。
**`Cargo.toml` の `version` だけを唯一の情報源とする。**
bump 時に書き換えるファイルが 1 つで済み、食い違いが原理的に起きない。
`Cargo.lock` は追随するのでコミットに含める。

#### ワークフローの入力

```yaml
on:
  workflow_dispatch:
    inputs:
      bump:
        description: バージョンの上げ幅
        type: choice
        options: [patch, minor, major]
        default: patch
      dry_run:
        description: リリースを作らずビルドのみ
        type: boolean
        default: false

concurrency:
  group: release
  cancel-in-progress: false

permissions:
  contents: write
```

#### タグ push 方式との差分

**キックの場所だけではない部分**が 1 つある。

| | タグ push 方式 | workflow_dispatch 方式 |
| --- | --- | --- |
| バージョンを決める主体 | 人間（タグ名） | **ワークフロー**（現在値 + bump レベルから計算） |
| ファイルへの反映 | 人間が事前に `Cargo.toml` を編集 | **ワークフローが書き戻してコミット** |
| タグ | 人間が打つ | Release 作成時に GitHub が打つ（`releaseCommitish` の位置） |
| 食い違いの可能性 | タグとファイルがズレうる | **原理的にズレない** |

つまり「バージョン計算 + ファイル書き戻し + コミット」というステップが増えるが、
**人間がファイル更新を忘れる余地が消えるため、整合性はむしろ上がる。**

#### 実務上の注意点

- `permissions: contents: write` が必要（bump コミットの push と Release 作成の両方）
- **`main` にブランチ保護をかけている場合、`GITHUB_TOKEN` による push が弾かれる。**
  保護をかけない / PAT か GitHub App トークンで bypass する / bump を PR 経由にする、のいずれかを選ぶ
- `concurrency` で二重起動を防ぐ（同時実行するとバージョンが競合する）
- `winget-releaser` はタグ名からバージョンを取るため、`v` プレフィックスの有無を
  タグ・Release・マニフェストで統一する
- Release は `gh release create` で作る。タグが未作成なら、その時点で
  指定したコミットに対して GitHub 側が打つ

### 9.4 WebView2 未導入時の挙動

起動時に §8 のレジストリを確認する。

- 導入済み → 通常起動
- 未導入 → ダイアログを表示し、
  [WebView2 ダウンロードページ](https://developer.microsoft.com/microsoft-edge/webview2/) を案内して終了

インストーラ版では NSIS の `webviewInstallMode`（`downloadBootstrapper`）で
インストール時に解決させるため、この分岐は主にポータブル版のためのもの。
