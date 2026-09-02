# 要件定義: WSL USB Identity Manager

版: 0.1（初版）
作成日: 2026-09-02

本書は本アプリケーションの要件を定義する。
根拠となる実測は [research-findings.ja.md](research-findings.ja.md)、
識別方式の設計は [identification-policy.ja.md](identification-policy.ja.md)、
技術選定は [platform-evaluation.ja.md](platform-evaluation.ja.md) に分離している。

---

## 1. 目的

Windows 上で `usbipd` を用いて WSL に USB デバイスを転送する運用において、
**「いま何が刺さっていて、それがどの物理デバイスなのか」を確実に把握できるようにする。**

単なる `usbipd` の GUI ラッパーではない。中核は **デバイスの継続的な同一性の追跡** である。

### 1.1 解決する問題

現行ツール（wsl-usb-gui）では以下が成立しない。

1. **シリアル番号を持たない同型デバイスを区別できない。**
   CH340 が 3 個あると、どれがどのボードか分からない。
2. **BUSID が信用できない。**
   BUSID は `Hub_#NNNN` + `Port_#MMMM` であり、`Hub_#NNNN` は Windows が
   ハブを認識した順に振る動的番号である（[F1](research-findings.ja.md#f1-usbipd-の-busid-は物理位置ではなく-windows-の動的ハブ番号)）。
   ハブの接続順や再起動で変化し、**同じ BUSID が別のデバイスを指すようになる。**
3. **COM 番号も `/dev/ttyUSB*` も接続状況で変わる。**
4. **デバッグプローブ（WCH-LinkE 等）は、プローブ自身が識別できても
   その先のボードが分からない。**

### 1.2 識別の対象範囲

USB シリアル変換アダプタやデバッグプローブは、**それ自体は識別できても、
その先に何が繋がっているかは分からない。**

- CH340 / CP210x / FTDI のシリアル番号は **アダプタ** を指すのであって、
  その先のボードを指さない。アダプタを別のボードに繋ぎ替えても番号は変わらない。
- WCH-LinkE のシリアル番号は **プローブ** を指すのであって、
  デバッグピンに繋がったボードを指さない。ボードを載せ替えても番号は変わらない。
- ESP32-S3 / C3 等のネイティブ USB 機種は、ボード自身が USB デバイスなので
  この問題が発生しない。

ユーザーが「このデバイス」と言うとき、多くの場合それは **アダプタやプローブではなく、
その先のボード** を指している。

> **本アプリケーションは、アダプタやプローブの先にあるボードまで識別することを
> 目的とする。**

#### 現在の対応範囲

先のボードまで識別できるのは、以下の 2 系統に限る。

| 系統 | 識別できるもの |
| --- | --- |
| **ESP32 系** | eFuse MAC によるボード個体 + チップ種別 |
| **CH32 RISC-V 系**（WCH-Link / WCH-LinkE の先） | 部品 UUID によるボード個体 + チップ種別 |

**要件 R1.1**: 上記以外のデバイスについては、アダプタ / プローブまでの識別に留める。
その場合、識別できているのがトランスポート層までである旨を UI 上で区別して示す。

**要件 R1.2**: 対応系統を後から追加できるよう、
系統ごとの識別処理を差し替え可能な単位に分離する。

### 1.3 中核となる設計思想

**「どこに接続されているか」と「それが何という物理デバイスなのか」を分離して扱う。**

| 分類 | 内容 | 永続化 |
| --- | --- | --- |
| Runtime Connection | BUSID / COM 番号 / `/dev/ttyUSB*` / Attach 状態 | **してはならない** |
| Identity | USB シリアル / Application Device ID / ユーザー定義名称 | する |

---

## 2. 責務分担

### 2.1 本アプリケーション（Windows 側）の責務

- Windows が認識している USB デバイスの列挙
- USB デバイスの識別情報の取得
- デバイスへの識別問い合わせ（プローブ）
- `usbipd` の状態取得と操作（Bind / Unbind / Attach / Detach）
- Attach 対象 WSL Distribution の管理
- USB デバイスとユーザー定義情報の関連付けと永続化
- 接続・切断状態の監視と表示更新
- Windows 側状態と WSL への接続状態の統合表示
- 操作結果およびエラーの表示

### 2.2 WSL / Linux 側の責務（本アプリの責務外）

- `/dev/ttyUSB*` `/dev/ttyACM*` 等のデバイスノード管理
- udev によるデバイスイベント処理
- デバイスノードの権限管理、Linux グループ設定
- 安定したデバイス名の生成、シンボリックリンクの作成・削除
- Attach / Detach に連動した Linux 側処理

既存の [board-identify](https://github.com/tanakamasayuki/board-identify) による
WSL 内部のシンボリックリンク管理の仕組みは維持する。
**本アプリケーションはシンボリックリンクを生成・変更しない。**
取得できる場合に参照・表示するのみとする。

### 2.3 初期リリースの非対象

- udev ルールの生成・管理
- WSL 内部のシンボリックリンクの生成・削除
- Linux のデバイス権限・グループ管理
- WSL 内アプリケーションの起動管理
- USB デバイス用 Linux ドライバの管理
- ファームウェア書き込み
- ESP32 等の開発環境管理

---

## 3. 状態モデル

### 3.1 3 層のアイデンティティ

§1.2 の通り、1 本のケーブルの先には最大 3 つの「もの」がある。
board-identify と同じ 3 層モデルで扱う。

| 層 | 内容 | 例 | 識別手段 |
| --- | --- | --- | --- |
| **Port** | 一時的な接続位置 | BUSID / COM8 / `/dev/ttyUSB2` | LocationPaths（永続化しない） |
| **Transport** | USB アダプタ / ブリッジ / プローブ | CH340, CH343, WCH-LinkE | USB Serial（無いこともある） |
| **Target** | その先のマイコンボード | ESP32-S3, CH32X035 | **プローブ**（§4.6） |

- ネイティブ USB のボード（ESP32-S3 等）では Transport と Target が同一になる。
- 生の USB シリアルアダプタに何も繋がっていない場合、Target は存在しない。
- **1 つの物理接続が Transport と Target の両方の識別子を同時に持ちうる**（R4.11）。

**要件 R3.1**: 一覧および詳細画面において、
表示している識別子が Transport のものか Target のものかを区別できること。

以降の §3.2〜§3.5 では、デバイスの状態を 4 つの情報群に分離して定義する。

### 3.2 USB Identity

物理 USB デバイスに由来する、Windows から取得できる情報。

| 項目 | 取得元 | 備考 |
| --- | --- | --- |
| VID / PID | Instance ID | |
| USB Serial Number | Instance ID 第 3 要素 | **無いデバイスがある**（CH340 等） |
| Manufacturer | `DEVPKEY_Device_Manufacturer` | |
| Product Name | `DEVPKEY_Device_BusReportedDeviceDesc` | USB の iProduct 文字列 |
| Windows Friendly Name | `DEVPKEY_Device_FriendlyName` | |
| Device Instance ID | `DEVPKEY_Device_InstanceId` | **usbipd との結合キー** |
| Container ID | `DEVPKEY_Device_ContainerId` | シリアル有りなら UUIDv5 で安定、無しなら UUIDv1 |
| バインド中ドライバ | `DEVPKEY_Device_Service` | WinUSB 可否の判定に使う |

### 3.3 Runtime Connection

現在の接続状態。**永続的な識別子として使用しない。**

| 項目 | 取得元 |
| --- | --- |
| 物理ポートパス | `DEVPKEY_Device_LocationPaths` |
| BUSID | `usbipd state` の `BusId`（未接続時は `null`） |
| Present / Missing | 列挙結果 |
| Shared / Not Shared | `usbipd state` の `PersistedGuid` |
| Attached / Detached | `usbipd state` の `StubInstanceId` / `ClientIPAddress` |
| Attach 先 Distribution | 本アプリが記録 |
| COM ポート | レジストリ `Device Parameters\PortName`（**表示専用**） |

### 3.4 Application Identity

USB デバイスの先に存在する機器が提供する情報。プローブで取得する。

```text
Device ID          例: ESP32 の eFuse MAC、CH32 の部品 UUID
Device Type        例: esp32-s3, ch32x035c8t6
Hardware Revision
Firmware Version
Role
```

### 3.5 User Metadata

ユーザーが独自に管理する情報。Windows や USB が提供する情報が変化しても維持される。

```text
Alias（表示名）      必須
Memo                必須
Tags
用途 / 設置場所 / 管理番号
Default WSL Distribution
Auto Attach 設定
```

---

## 4. デバイス識別

詳細は [identification-policy.ja.md](identification-policy.ja.md) を参照。本節は要件のみを記す。

### 4.1 識別の 3 経路

| # | 経路 | 条件 | プローブ |
| --- | --- | --- | --- |
| 1 | USB シリアルによる識別 | デバイスがシリアルを持つ | **不要** |
| 2 | キャッシュ照合 | ポート位置 + VID/PID で候補が一意 | **不要** |
| 3 | プローブによる識別 | 経路 1・2 で決まらない | 要 |

**要件 R4.1**: シリアルを持つデバイスは、前回の識別結果をキャッシュから復元し、
プローブを行わずに識別しなければならない。

**要件 R4.2**: シリアルを持たないデバイスは、ポート位置と VID/PID による
キャッシュ照合を先に試み、一意に定まる場合はプローブを行ってはならない。

### 4.2 確信度

各デバイスは常に確信度を持つ。

| confidence | 根拠 | 自動 Attach |
| --- | --- | --- |
| `confirmed` | Application Identity 一致、または USB シリアル一致 | 可 |
| `probable` | ポート位置 + VID/PID が一致し、競合候補なし | 設定次第 |
| `ambiguous` | 同一 VID/PID の候補が複数 / 位置が変わった | **不可** |
| `unknown` | 未登録 | 不可 |

**要件 R4.3**: 確信度を UI 上に明示しなければならない。
「たぶんこれ」を「これ」と表示してはならない。

**要件 R4.4**: `ambiguous` および `unknown` のデバイスに対して
自動 Attach を実行してはならない。

### 4.3 プローブの実行契機

プローブには副作用がある。`esptool` 系はボードを再起動させ、
WCH-Link の attach はターゲットのコアを一時停止させる。

**要件 R4.5**: プローブを実行してよいのは以下の 2 契機に限る。

1. **ユーザーが明示的に「識別」を指示したとき**
2. **接続直後の猶予時間内**（オプション、既定で無効）

**要件 R4.6**: 以下ではプローブを実行してはならない。

- 定期ポーリング
- 一覧画面の更新
- アプリ起動時の一括スキャン
- 自動 Attach の判定

**要件 R4.7**: プローブ実行前に、副作用（対象が再起動する等）をユーザーに提示する。

### 4.4 接続直後の自動識別（オプション機能）

**要件 R4.8**: 以下の設定を持つ自動識別機能を提供する。既定は無効。

| 設定項目 | 既定値 |
| --- | --- |
| 自動識別の有効化 | 無効 |
| 対象 | シリアルを持たないデバイスのみ |
| 猶予時間 | 接続イベントから 10 秒以内 |
| 対象 VID/PID | ユーザーが明示的に列挙 |
| 対象確信度 | `ambiguous` 以下のみ |

**要件 R4.9**: 対象 VID/PID に列挙されていないデバイスへ、
自動でデータを送信してはならない。

### 4.5 識別子フォーマット

board-identify と同一のフォーマットを用いる。

```text
<variant>-<unique-id>

esp32-s3-7cdfa1123456
ch32x035c8t6-1ff9abcd880ebc48
wch-link-fc928f068181
```

規則: 小文字 ASCII / 区切りは `-` / unique-id から記号を除去 /
6 文字未満の unique-id は不採用 / `/` と NUL を含まない。

**要件 R4.10**: 同一の物理デバイスに対して、本アプリと board-identify が
同じ識別子文字列を生成しなければならない。
実装は共有しないが、フォーマットはテストで固定する。

**要件 R4.11**: 1 つのデバイスが複数の識別子を同時に持ちうる。
デバッグプローブの場合、プローブ自身とその先のボードの両方を保持する。

### 4.6 プローブの対象と手段

Transport（アダプタ / プローブ）の識別は USB 記述子だけで済むが、
**Target（その先のボード）の識別にはプローブが要る。**

#### Target まで識別できる系統（現在の対応範囲）

| 系統 | 手段 | 得られる Target ID | 前提 |
| --- | --- | --- | --- |
| **ESP32 系**（CH340 / CP210x 等の裏） | シリアル経由の ROM ブートローダ通信、eFuse MAC 読み出し | MAC アドレス + チップ種別 | COM ポートが開けること |
| **CH32 RISC-V 系**（WCH-Link / WCH-LinkE の先） | WinUSB 経由のベンダプロトコル | 部品 UUID + チップ署名 | interface 0 に WinUSB がバインドされていること |

**要件 R4.12**: WCH-Link 系のプローブ実行前に `DEVPKEY_Device_Service` を確認し、
`WinUSB` でない場合はプローブを試行せず、WinUSB ドライバの割り当てが必要である旨を
ユーザーに案内する。**アプリがドライバを自動で差し替えてはならない。**

#### プローブが不要な場合

| 対象 | 理由 |
| --- | --- |
| ESP32-S3 / C3 等（ネイティブ USB） | Transport と Target が同一で、MAC が USB シリアルとして露出している |
| USB シリアルを持つアダプタ自体 | 記述子だけで Transport を識別できる |

#### 対応外の系統

上記以外（Arduino、RP2040、STM32、汎用の CH340 の先にある不明なボード等）は、
**Transport までの識別に留まる。**

**要件 R4.13**: 対応外の系統では Target の識別を試行しない。
未対応であることを UI 上で示し、ユーザーが Alias で補えるようにする。

**要件 R4.14**: 系統の追加が、識別処理の追加のみで完結する構造とする
（既存の系統の処理に手を入れる必要がないこと）。

---

## 5. usbipd 連携

### 5.1 結合キー

**要件 R5.1**: `usbipd` との対応付けには `usbipd state` が返す
`InstanceId` を用いる。BUSID を対応付けに使用してはならない。

**要件 R5.2**: `usbipd` の状態取得には `usbipd state`（JSON）のみを使用する。
`usbipd list` のテキスト出力をパースしてはならない。

**要件 R5.3**: Attach 中のデバイスは `StubInstanceId`
（`Vid_80EE&Pid_CAFE\<元の第3要素>`）で追跡する。

### 5.2 操作

GUI から以下を実行できること。

| 操作 | コマンド | 権限 |
| --- | --- | --- |
| Bind（共有） | `usbipd bind --busid <BUSID>` | **管理者** |
| Unbind | `usbipd unbind --busid <BUSID>` | **管理者** |
| Attach | `usbipd attach --busid <BUSID> --wsl <DISTRO>` | 一般 |
| Detach | `usbipd detach --busid <BUSID>` | 一般 |
| 状態更新 | `usbipd state` | 一般 |

**要件 R5.4**: BUSID は操作を発行する直前に `usbipd state` から取得した値を用いる。
記録した BUSID を再利用してはならない。

**要件 R5.5**: 操作対象のデバイスを、ユーザーが識別情報を確認した上で選べること。

### 5.3 権限設計

**要件 R5.6**: アプリケーション本体は非昇格で起動する。
管理者権限が必要な操作の実行時にのみ、
`ShellExecuteW(verb="runas")` で `usbipd.exe` を昇格実行する。

**要件 R5.7**: 初期設定として `usbipd policy add`（AutoBind）の設定を
支援する導線を提供する。これにより日常運用での昇格頻度をほぼゼロにできる。

### 5.4 環境依存の警告

**要件 R5.8**: `usbipd` が出力する警告（USB フィルタドライバとの非互換など）を
検出し、`bind --force` が必要な状況をユーザーに提示する。

**要件 R5.9**: `bind --force` は Windows からデバイスを取り上げるため、
プローブと両立しない。この制約を UI 上で明示する。

---

## 6. WSL 連携

**要件 R6.1**: 利用可能な WSL Distribution を列挙し、Attach 先として選択できること。

**要件 R6.2**: デバイスごとに既定の Distribution を設定できること。

**要件 R6.3**: WSL 側の状態を**参照**できること。参照のみで、変更は行わない。

参照する情報の例:

```text
対応する Linux デバイスノード   /dev/ttyUSB1
既存のシンボリックリンク         /dev/board-identify/by-id/co2-sensor
vhci ポートと BUSID の対応       usbip port
```

`usbipd-win` は Linux バイナリ `usbip` を同梱している
（`C:\Program Files\usbipd-win\WSL\usbip`）。
`wsl.exe` 経由でこれを実行することで vhci ポートの状態を取得できる。

**要件 R6.4**: WSL 側から取得した情報を、Windows 側の永続的な識別情報として
使用してはならない。表示のためだけに用いる。

---

## 7. データモデルと永続化

### 7.1 保存する情報

```text
Device（永続）
  identity_key            確定した識別子（§4.5 の形式）
  usb_serial              あれば。経路 1 の照合キー
  vid / pid
  application_identity    Device ID / Type / HW Rev / FW Version / Role
  user_metadata           Alias / Memo / Tags / Default WSL / Auto Attach

Hints（学習・更新される）
  last_location_path      最後に見た物理ポートパス
  last_instance_id        最後に見た Instance ID
  last_com_port           表示用の参考値
  last_seen_at
  probe_result_at         最後にプローブした日時
```

### 7.2 永続化してはならない情報

**要件 R7.1**: 以下を識別子として永続化してはならない。

- BUSID
- COM 番号（表示用の参考値としての保持は可）
- `/dev/ttyUSB*` 等の Linux デバイスノード
- `usbipd` の `PersistedGuid`（usbipd 内部の識別子であり、
  シリアル無しデバイスではポートに紐づくため）

### 7.3 保存場所

**要件 R7.2**: 実行ファイルと同じディレクトリに `portable.txt` が存在する場合は
そのディレクトリに、存在しない場合は `%APPDATA%` 配下に保存する。

---

## 8. デバイス監視

**要件 R8.1**: 以下の状態変化を検出し、表示を更新する。

- Windows への USB 接続 / 切断
- `usbipd` の Bind 状態変更
- WSL への Attach / Detach

**要件 R8.2**: デバイス列挙とプロパティ取得には `CfgMgr32` / `SetupAPI` を
ネイティブに呼び出す。PowerShell の PnP コマンドレットを使用してはならない
（実測で 700 倍以上の性能差がある。[F7](research-findings.ja.md#f7-windows-pnp-へのアクセスは-powershell-経由では実用にならない)）。

**要件 R8.3**: ホットプラグ通知には `CM_Register_Notification` または
`WM_DEVICECHANGE` を用いる。定期ポーリングを主たる検出手段としない。

---

## 9. 自動 Attach

**要件 R9.1**: 登録済みデバイスについて自動 Attach を設定できること。

```text
Auto Attach:  Enabled
Target:       Ubuntu-24.04
```

**要件 R9.2**: 自動 Attach の判定に BUSID のみを用いてはならない。

**要件 R9.3**: 自動 Attach を実行してよい確信度の下限をユーザーが設定できること。
既定は `confirmed` のみ。

**要件 R9.4**: 自動 Attach の判定のためにプローブを実行してはならない（R4.6）。

---

## 10. ユーザーインタフェース

### 10.1 メイン画面（デバイス一覧）

Windows に接続されている USB デバイスと、登録済みデバイスを一覧表示する。

| 列 | 内容 |
| --- | --- |
| Name | ユーザー定義名称（Alias） |
| Confidence | 確信度（§4.2） |
| Device | Windows / USB の認識名 |
| VID:PID | |
| Serial | USB Serial Number（無い場合は空欄と明示） |
| BUSID | 現在の BUSID（一時的な値である旨を示す） |
| Shared | usbipd 共有状態 |
| WSL | Attach 状態 |
| Distribution | Attach 先 |
| Memo | ユーザーメモ |

**要件 R10.1**: 現在接続されていない登録済みデバイスも一覧に表示し、
接続中のものと区別できること。

**要件 R10.2**: `unknown` / `ambiguous` のデバイスでは「識別」操作への導線を強調する。

### 10.2 詳細画面

§3.2〜§3.5 の 4 情報群を分けて表示する。

```text
USB Identity        VID / PID / Serial / Manufacturer / Product /
                    Instance ID / Location Path / Driver Service
Application Identity Device ID / Device Type / HW Revision / FW Version / Role
User Metadata       Alias / Memo / Tags / Default WSL / Auto Attach
Runtime Connection  BUSID / COM / Shared / Attached / Distribution
WSL State           デバイスノード / シンボリックリンク（参照のみ）
```

### 10.3 設定画面

- 接続直後の自動識別（§4.4 の各項目）
- 自動 Attach の確信度下限
- 既定の WSL Distribution
- 表示言語（§10.4）

### 10.4 多言語対応

**要件 R10.3**: 起動時に OS の表示言語を取得し、
対応言語があれば自動的にその言語で表示する。

**要件 R10.4**: 設定画面で表示言語を明示的に選択できること。
選択肢は「システムに従う（既定）」と、対応する各言語。

**要件 R10.5**: UI 文字列の原本は英語とする。
翻訳リソースは言語ごとのファイルに分離し、
未翻訳の項目は英語にフォールバックする。

**要件 R10.6**: 初期対応言語は英語と日本語とする。
言語の追加が翻訳ファイルの追加のみで済む構造にする。

---

## 11. 技術スタック

決定の根拠は [platform-evaluation.ja.md](platform-evaluation.ja.md) を参照。

| レイヤ | 採用 |
| --- | --- |
| 言語 | Rust |
| GUI | Tauri 2（WebView2） |
| デバイス列挙・プロパティ | `windows` crate → `CfgMgr32` / `SetupAPI` |
| USB 生アクセス（WinUSB） | `nusb` |
| シリアル | `serialport` |
| WCH-Link プローブ | [ch32rv](https://github.com/ch32-riscv-ug/ch32rv) の crate |
| ESP32 プローブ | `espflash` crate |

**要件 R11.1**: デバイスの列挙と識別情報の取得は `CfgMgr32` が担当する。
`nusb` は WinUSB プローブの I/O にのみ用いる。
（`nusb` は Windows において、デバイス全体が特定ドライバにバインドされている場合に
シリアル番号を取得できないため。CH340 が該当する）

**要件 R11.2**: 起動時に WebView2 Runtime の有無を確認する。
未導入の場合はダウンロード先を案内して終了する。

```text
HKLM\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}\pv
HKCU\Software\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}\pv
```

---

## 12. 配布とリリース

### 12.1 配布物

リリースごとに 2 種類を提供する。

```text
wsl-usb-identity-manager_<version>_x64-setup.exe     NSIS / per-user / 管理者権限不要
wsl-usb-identity-manager_<version>_x64_portable.zip  ポータブル（portable.txt 同梱）
```

**要件 R12.1**: MSI は提供しない（per-machine と per-user の併存を避けるため）。

**要件 R12.2**: 自動更新機構は実装しない。
更新は `winget upgrade` または ZIP の差し替えによる。

**要件 R12.3**: WinGet には NSIS インストーラのみを登録する
（`InstallerType: nullsoft`）。ポータブル ZIP は GitHub Release で直接配布する。

### 12.2 バージョン管理

**要件 R12.4**: バージョンの唯一の情報源は `Cargo.toml` の `version` とする。
`tauri.conf.json` に `version` を記述しない（Tauri は省略時に `Cargo.toml` を参照する）。

### 12.3 リリース手順

**要件 R12.5**: リリースは GitHub Actions の画面から `workflow_dispatch` で
手動起動する。タグ push によるリリースは行わない。

起動時に bump レベル（`major` / `minor` / `patch`）を指定する。

```text
workflow_dispatch (bump: major|minor|patch)
  1. Cargo.toml の version を bump
  2. CHANGELOG.md の ## Unreleased の内容を ## <version> セクションへ移動
     （## Unreleased の見出しは空のまま残す）
  3. 変更をコミットして既定ブランチへ push
  4. ビルド（NSIS インストーラ + ポータブル ZIP）
  5. CHANGELOG.md の ## <version> セクションを抽出してリリース本文を生成
  6. タグを作成し GitHub Release を公開
  7. winget-pkgs へマニフェストの PR を作成
```

**要件 R12.6**: 変更点は開発時に `CHANGELOG.md` の `## Unreleased` に記載する。
リリース処理がバージョン番号を差し込み、その内容をリリース本文に転記する。

**要件 R12.7**: 同時実行によるバージョン競合を防ぐため、
ワークフローに `concurrency` を設定する。

**要件 R12.8**: bump コミットの push が必要なため、
既定ブランチの保護設定とワークフローの権限を整合させる。

---

## 13. ドキュメント規則

**要件 R13.1**: 意思決定に関する文書は日本語のみとし、`*.ja.md` の名前で作成する。

**要件 R13.2**: 利用者が読む文書（README 等）は英語を原本とし、
`*.ja.md` で日本語版を作成して相互にリンクする。

**要件 R13.3**: ソースコード中のコメントと識別子は英語とする。

**要件 R13.4**: `CHANGELOG.md` は単一ファイルとし、
各項目を `(EN)` と `(JA)` の対で記述する。

---

## 14. 初期リリースの範囲

### 14.1 必須

- USB デバイス一覧表示（接続中 / 登録済みの両方）
- VID / PID / USB Serial / Windows 認識名 / BUSID の表示
- `usbipd` 状態表示（Shared / Attached / Distribution）
- Bind / Unbind / Attach / Detach
- Attach 先 WSL Distribution の選択
- ユーザー定義名称（Alias）とメモ
- デバイス設定の永続化
- USB 接続・切断時の一覧更新
- **確信度の表示**
- **Transport / Target の区別表示**（§3.1、R3.1）
- **明示的な「識別」操作による Target のプローブ。
  対応系統は ESP32 系と CH32 RISC-V 系の 2 つのみ**（§4.6）
- 表示言語の自動判定と手動選択

### 14.2 拡張として後続

- 接続直後の自動識別（§4.4）
- 自動 Attach（§9）
- WSL 内シンボリックリンクの表示（§6）
- **Target 識別の対応系統の追加**（Arduino / RP2040 / STM32 等）
- ユーザー定義の問い合わせプロトコル
- Tags / 設置場所 / 管理番号などの拡張メタデータ

### 14.3 構造上の要求

**要件 R14.1**: 14.2 の機能を後から追加できる構造とする。
特に、識別経路（§4.1）とプローブ実装を差し替え可能な形で分離する。

---

## 15. 想定する利用フロー

```text
USB デバイスを Windows へ接続
        ↓
本アプリが接続イベントを検出
        ↓
USB Identity を取得（CfgMgr32）
        ↓
経路 1・2 で登録済みデバイスと照合
        ↓
  ┌─ 一意に定まる ──→ Alias / Memo を復元、確信度を提示
  │
  └─ 定まらない ────→ 「未確認」として表示
                        ユーザーが「識別」を押す → プローブ
                        （または接続直後の自動識別が有効なら実行）
        ↓
usbipd 状態を取得（usbipd state）
        ↓
ユーザー操作または自動処理で Attach
        ↓
WSL 側で USB デバイスを認識
        ↓
WSL 内部の既存処理（udev / board-identify）が動作
        ↓
シンボリックリンクが生成される
        ↓
必要に応じて本アプリから WSL 側の状態を参照・表示
```

---

## 16. 参照

- [research-findings.ja.md](research-findings.ja.md) — 実測による事前調査結果
- [identification-policy.ja.md](identification-policy.ja.md) — 識別ポリシーの設計
- [platform-evaluation.ja.md](platform-evaluation.ja.md) — 言語・フレームワーク・配布方式の評価
- [board-identify](https://github.com/tanakamasayuki/board-identify) — WSL 側の識別ツール
- [ch32rv](https://github.com/ch32-riscv-ug/ch32rv) — WCH-Link プローブの実装
- [usbipd-win](https://github.com/dorssel/usbipd-win)
