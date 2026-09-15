# 事前調査結果（ゼロベース再調査）

調査日: 2026-09-02
調査環境: Windows 11 Pro 26200.9168 / usbipd-win 5.3.0 / WSL 2.7.12.0 (Ubuntu-24.04)

本書は仕様策定の前提となる「実測で確認した事実」をまとめたものである。
推測と実測を区別し、実測したものは根拠を併記する。

---

## F1. usbipd の BUSID は物理位置ではなく Windows の動的ハブ番号

### 実測

同一 PC で、ドック未接続時と接続後に `usbipd list` を実行した結果。

ドック未接続:

```text
2-5    30c9:0050  Integrated Camera ...
2-7    27c6:6594  Goodix MOC Fingerprint
2-10   8087:0033  インテル(R) ワイヤレス Bluetooth(R)
```

ドック接続後（同一セッション）:

```text
2-5,  2-7,  2-10   （上記に加えて）
13-1   1d5c:7102  Generic Billboard Device
14-4   045e:0039  Microsoft USB IntelliMouse Optical
15-2   05e3:0749  USB 大容量記憶装置
15-4   0b95:1790  ASIX USB to Gigabit Ethernet
16-1   1a86:7523  USB-SERIAL CH340 (COM10)
16-2   1a86:7523  USB-SERIAL CH340 (COM11)
16-3   1a86:55d3  USB-Enhanced-SERIAL CH343 (COM7)
16-4   1a86:7523  USB-SERIAL CH340 (COM3)
```

同じデバイスの Windows PnP プロパティ `DEVPKEY_Device_LocationInfo` と突き合わせると、
BUSID が完全に一致する。

| デバイス | LocationInfo | usbipd BUSID |
| --- | --- | --- |
| Bluetooth | `Port_#0010.Hub_#0002` | `2-10` |
| Goodix | `Port_#0007.Hub_#0002` | `2-7` |
| Mouse | `Port_#0004.Hub_#0014` | `14-4` |
| Card Reader | `Port_#0002.Hub_#0015` | `15-2` |
| Billboard | `Port_#0001.Hub_#0013` | `13-1` |
| CH340 #1 | `Port_#0001.Hub_#0016` | `16-1` |

### 結論

**BUSID = `Hub_#NNNN` ハイフン `Port_#MMMM`。**
`Hub_#NNNN` は Windows がハブを認識した順に振る通し番号であり、物理トポロジではない。
ハブの接続順・接続タイミング・再起動で変化する。

→ **BUSID は「操作を発行するその瞬間だけ有効な一時ハンドル」として扱う。永続化してはならない。**

これはユーザーの「USBIPD もまちがって伝えたりするのであてにできない」という経験則の
技術的裏付けである。BUSID 自体が嘘なのではなく、
**同じ BUSID が時間とともに別のデバイスを指すようになる**ため、
記録した BUSID で操作すると別デバイスを掴む。

---

## F2. 物理ポートの安定識別子は DEVPKEY_Device_LocationPaths

### 実測

```text
CH340 #1  PCIROOT(0)#PCI(1400)#USBROOT(0)#USB(1)#USB(3)#USB(1)#USB(1)
CH340 #2  PCIROOT(0)#PCI(1400)#USBROOT(0)#USB(1)#USB(3)#USB(1)#USB(2)
CH343     PCIROOT(0)#PCI(1400)#USBROOT(0)#USB(1)#USB(3)#USB(1)#USB(3)
CH340 #3  PCIROOT(0)#PCI(1400)#USBROOT(0)#USB(1)#USB(3)#USB(1)#USB(4)
```

PCI ルート → ホストコントローラ → ハブポートの連鎖で表現されており、
`Hub_#NNNN` のような動的番号を含まない。

### 結論

同じ物理配線である限り安定。**「ポートの識別子」としてはこれを使う。**
ただしこれは *ポート* の識別であって *デバイス* の識別ではない（F3 参照）。

---

## F3. シリアル番号を持たないデバイスは Windows 側だけでは原理的に個体識別できない

### 実測: Device Instance ID

```text
USB\VID_1A86&PID_7523\8&7CC2A31&0&1        CH340（シリアル無し）
USB\VID_1A86&PID_7523\8&7CC2A31&0&2        CH340（シリアル無し）
USB\VID_1A86&PID_7523\8&7CC2A31&0&4        CH340（シリアル無し）
USB\VID_1A86&PID_55D3\5B5F090816           CH343（シリアル有り）
USB\VID_303A&PID_1001\70:04:1D:DA:86:F0    ESP32-S3 native USB（MAC がシリアル）
USB\VID_2341&PID_0043\8573531333335160D1C2 Arduino Uno
```

Instance ID の第 3 要素は、**シリアルがあればシリアル、無ければ
`親ハブ由来ハッシュ & 0 & ポート番号`** になる。後者はポートの識別子である。

### 実測: ContainerId

```text
CH340 #1 (シリアル無)  6aa62d84-4e6a-11f1-b04a-c0a5e85bd89d   UUID v1（時刻ベース）
CH340 #2 (シリアル無)  6aa62d6a-4e6a-11f1-b04a-c0a5e85bd89d   UUID v1
CH340 #3 (シリアル無)  fb0cb475-9ab0-11f1-b07d-c0a5e85bd89d   UUID v1
CH343    (シリアル有)  24309343-175d-5c84-a14b-bb9cc186c126   UUID v5（名前ベース）
Card Rdr (シリアル有)  1b62dd7d-6c9a-57f3-a424-5979ac2abcaa   UUID v5
AX88179  (シリアル有)  e17ce95f-2c07-5b26-ae60-762714168e78   UUID v5
```

**バージョンフィールドがきれいに割れている。**

- シリアル有り → **UUID v5**（VID/PID/Serial からの決定論的ハッシュ）。
  ポートを差し替えても同じ値になる。
- シリアル無し → **UUID v1**（初回接続時にその場で生成）。
  Instance ID（＝ポート）に紐づいて保存されるだけ。

### 結論

ContainerId は一見「デバイス固有 ID」に見えるが、
**シリアル無しデバイスではポート識別子に退化する。**
シリアル持ちデバイスでは複合デバイスの各インタフェースを束ねる優秀なキーになる。

> **Windows が提供する情報だけでは、シリアルを持たない CH340 が複数ある場合、
> 「どのポートに刺さっているか」以上のことは絶対に分からない。**
> これは実装の巧拙ではなく、USB 記述子に区別する情報が存在しないという物理的事実。

→ 個体識別には **その先の機器に問い合わせる（プローブする）以外の方法がない**。
これは board-identify が Linux 側で到達したのと同じ結論である。

---

## F4. usbipd との結合キーは InstanceId（BUSID ではない）

`usbipd state` が返す JSON:

```json
{
  "BusId": "16-1",
  "ClientIPAddress": null,
  "Description": "USB-SERIAL CH340 (COM10)",
  "InstanceId": "USB\\VID_1A86&PID_7523\\8&7CC2A31&0&1",
  "IsForced": false,
  "PersistedGuid": "05d75b37-ae96-4b2a-9d0d-5cded25f52eb",
  "StubInstanceId": null
}
```

- `InstanceId` は Windows PnP の Device Instance ID と完全一致する。
  → **自前のデバイスモデルと usbipd を突き合わせる唯一の正しいキー。**
- `BusId` は未接続時 `null`。接続中のみ有効。
- `PersistedGuid` は bind（共有）した時点で usbipd が発行する GUID。未接続でも保持される。
  ただし usbipd 内部の識別子であり、シリアル無しデバイスでは結局ポートに紐づく。

### attach 中の追跡

レジストリ `HKLM\SYSTEM\CurrentControlSet\Enum\USB` に以下が残っていた。

```text
Vid_80EE&Pid_CAFE\58FA041040      VirtualBox USB Driver (VBoxUSB)
Vid_80EE&Pid_CAFE\5B5F090816      VirtualBox USB Driver (VBoxUSB)
Vid_80EE&Pid_CAFE\8&7cc2a31&0&1   VirtualBox USB Driver (VBoxUSB)
Vid_80EE&Pid_CAFE\8&7cc2a31&0&2   VirtualBox USB Driver (VBoxUSB)
```

attach すると Windows 上ではデバイスが `VID_80EE&PID_CAFE`（VBoxUSB スタブ）として
再列挙されるが、**Instance ID の第 3 要素（シリアル or ポートパス）はそのまま引き継がれる**。
`usbipd state` の `StubInstanceId` がこれを返す。

→ attach 中でも同一性の追跡は可能。
ただし **Windows からデバイス本体には触れない**（プローブ不可）。

---

## F5. usbipd 5.3 の自動化機能は「個体」を区別できない

```text
usbipd bind    --busid <BUSID> | --hardware-id <VID:PID>  [--force]
usbipd attach  --busid <BUSID> | --hardware-id <VID:PID>  --wsl [DISTRO]
               [--auto-attach] [--unplugged] [--host-ip <IP>]
usbipd policy  add | list | remove     AutoBind ルール。BUSID または VID:PID 単位
usbipd state                            JSON。唯一の機械可読インタフェース
```

- 指定できるのは **BUSID か VID:PID のみ**。
- CH340 が 3 個ある環境では `--hardware-id 1a86:7523` は 3 個全部にマッチする。
- `--auto-attach` は接続順に attach するため `/dev/ttyUSB*` の割り当てが不定になる。

→ **usbipd 単体では本件の問題は解決できない。ここが本アプリの存在理由。**

### 権限

- `bind` / `unbind` / `policy` : 管理者権限が必要
- `attach` / `detach` / `state` / `list` : 一般ユーザーで可
- `usbipd` サービスは自動起動で常駐している

### 環境固有の注意

```text
usbipd: warning: Unknown USB filter 'CsDeviceControl' may be incompatible ...
usbipd: warning: USB filter 'USBPcap' is known to be incompatible ...
                 'bind --force' will be required.
```

USBPcap が入っているため、この環境では `bind --force` が必要になるケースがある。
`--force` は Windows 側からデバイスを取り上げるため **プローブ（F6）と両立しない**。
GUI 上で明示的に区別して扱う必要がある。

### 同梱バイナリ

```text
C:\Program Files\usbipd-win\usbipd.exe
C:\Program Files\usbipd-win\Usbipd.PowerShell.dll
C:\Program Files\usbipd-win\WSL\usbip              Linux ELF
C:\Program Files\usbipd-win\WSL\usbip-auto-attach  Linux ELF
C:\Program Files\usbipd-win\Drivers\VBoxUSB.sys
```

`WSL\usbip` は Linux バイナリ。`wsl.exe` 経由で `usbip port` を実行すれば、
WSL 側の vhci ポートと Windows 側 BUSID の対応が取得できる。

---

## F6. WCH-LinkE への Windows アプリからのアクセスは可能（実機で確認）

### 実測: インストール済みドライバ

```text
Published Name: oem24.inf
Original Name:  wch-link_(interface_0).inf
Provider Name:  libwdi
Class Name:     USBDevice
Class GUID:     {88bae032-5a81-49f0-bc3d-a4ff138216d6}   WinUSB デバイスクラス

Published Name: oem79.inf
Original Name:  wchlinkwdm.inf
Provider Name:  wch.cn
Class Name:     WCH
Class GUID:     {77989adf-06db-4025-92e8-40d902c03b0a}   WCH 純正 WDM ドライバ

Published Name: oem110.inf
Original Name:  winusb_generic_device.inf
Provider Name:  libwdi
Signer Name:    USB\MS_COMP_WINUSB (libwdi autogenerated)
```

`oem24.inf` は **Zadig / libwdi によって WCH-Link の interface 0 に WinUSB を
割り当てたもの**であり、この PC には既に導入済み。

### 結論

- WCH-LinkE のベンダインタフェース（interface 0）に **WinUSB がバインドされていれば、
  通常の Windows アプリから管理者権限なしでオープンできる。**
  WinUSB はユーザーモード API であり、libusb / nusb / pyusb すべて同じ経路を使う。
- ただし **WCH 純正の `wchlinkwdm.inf` がバインドされていると開けない。**
  両方インストールされているため、どちらが当たっているかは
  デバイス側の `DEVPKEY_Device_Service` で判定できる（値が `WinUSB` か否か）。
- CDC インタフェース（interface 1）は通常の COM ポートとして見える。

### 追測: WinUSB は不要だった（実機で確認）

上記の結論は **WinUSB がバインドされている場合の話**であり、
「WinUSB でなければ開けない」は誤りだった。同じ PC で追加測定した結果を示す。

```text
USB\VID_1A86&PID_8010\<serial>            usbccgp       複合デバイスの親
USB\VID_1A86&PID_8010&MI_00\...           WCHLink_A64   ベンダインタフェース（WCH 純正）
USB\VID_1A86&PID_8010&MI_01\...           usbser        CDC（COM ポート）
```

この PC に接続したことのある WCH-Link は **全数が `WCHLink_A64`**、
すなわち WCH 純正ドライバであった（`oem24.inf` の WinUSB が当たっていたのは
Zadig を通した個体のみ）。

そして `ch32rv-usb` は、**nusb でインタフェースを claim できない場合に
CH375 系の `DeviceIoControl` 経路へ自動でフォールバックする。**
純正ドライバのままの WCH-Link に対して `ch32rv probe list` が
プローブ情報（`WCH-Link(CH549)` firmware 2.6）を返すことを確認した。

### 訂正後の結論

- **WinUSB の割り当ては前提条件ではない。**
  純正ドライバのままプローブできる。
- したがって「Zadig で WinUSB を割り当ててください」という案内は行わない。
  ドライバの差し替えを求めれば、利用者に余計な操作を強い、
  かつ WCH-Link の COM ポート機能を失わせることになる。
- `DEVPKEY_Device_Service` の値は**表示のための情報**であり、
  プローブ可否の判定には用いない。

---

## F7. Windows PnP へのアクセスは PowerShell 経由では実用にならない

### 実測（同一 PC、USB デバイス 20 個）

| 手段 | 所要時間 |
| --- | --- |
| PowerShell `Get-PnpDevice` + `Get-PnpDeviceProperty` | **120 秒超でタイムアウト** |
| ctypes 経由の `CfgMgr32` 直叩き（`CM_Get_Device_ID_ListW` + `CM_Get_DevNode_PropertyW`） | **170 ms** |

700 倍以上の差。ホットプラグのたびに再列挙する常駐アプリでは PowerShell は使えない。

→ **仕様要件**: デバイス列挙・プロパティ取得は `CfgMgr32` / `SetupAPI` をネイティブに呼ぶ。
`usbipd.exe` の呼び出しだけはプロセス起動で許容する（頻度が低いため）。

参考: レジストリ `HKLM\SYSTEM\CurrentControlSet\Enum\USB` の直読みも高速で、
**過去に接続したことのあるデバイス**（現在未接続を含む）と、
`Device Parameters\PortName` による COM 番号が取得できる。

---

## F8. board-identify（既存の WSL 側ツール）の設計

Python + uv 製。3 層のアイデンティティモデルを採用している。

| 層 | 内容 | 例 |
| --- | --- | --- |
| Port | 一時的なカーネルノード | `/dev/ttyUSB2` |
| Transport | USB アダプタ / ブリッジ | CH340, FTDI, WCH-Link |
| Target | その先のマイコン基板 | ESP32-S3, CH32X035 |

識別子フォーマット: `<variant>-<unique-id>`

```text
esp32-s3-7cdfa1123456
ch32x035c8t6-1ff9abcd880ebc48
wch-link-fc928f068181
```

1 ポートが複数の識別子を同時に持ちうる（プローブ自身と、その先の基板）。

WSL + USB/IP 環境で標準機構が壊れる理由も README に整理されている。

- Windows 側 BUSID は Linux 内に存在しないため udev ルールに書けない
- 転送されたデバイスは仮想ホストコントローラ上に現れるため、
  `ID_PATH` / `/dev/serial/by-path/` は `platform-vhci_hcd.0-usb-0:1:1.0` のような形になり、
  ポート番号は **attach 順**に由来する
- `/dev/serial/by-id/` はシリアルを持つアダプタでのみ有効

### プローブの副作用（重要）

README が明記している通り、プローブは **対象を乱す**。

- `esptool` は DTR/RTS を操作してブートローダに落とすため、**ファームウェアが再起動する**
- WCH-Link の attach は **ターゲットのコアを一時停止させる**
- プローブ中のシリアル出力は失われる

→ **仕様要件**: GUI が定期ポーリングでプローブしてはならない。
プローブは明示的な操作か、初回登録時の 1 回に限定する。
これは要件 R4.9「対象 VID/PID に列挙されていないデバイスへ自動でデータを送信してはならない」
に対する技術的裏付けである。

---

## F9. wsl-usb-gui（現在使用中のツール）

- Python + tkinter、PyOxidizer で単一 exe 化、MSI 配布
- `wsl_usb_gui/usb_monitor.py`, `wsl_usb_gui/win_usb_inspect/winusbclasses.py`
- 自動 attach プロファイルは **VID/PID ベース**

→ CH340 が複数ある場合に個体を区別できないのは、ツールの実装不足ではなく
**VID/PID しか見ていない設計に起因する**。
F3 の通り、Windows 側の情報だけでは（シリアル無しデバイスでは）
VID/PID より細かい区別がそもそも不可能であるため、
プローブを導入しない限りこの問題は解決しない。

---

## 調査から導かれる設計上の必須要件

1. **BUSID・COM 番号・`/dev/ttyUSB*` は一切永続化しない**（F1）
2. **usbipd との結合キーは InstanceId**（F4）
3. **物理ポートの識別には LocationPaths を使う**（F2）
4. **シリアル無しデバイスの個体識別はプローブでしか達成できない**（F3）
5. **プローブは破壊的なので、常時実行してはならない**（F8）
   → 「確信度」の概念を導入し、プローブ無しで済む経路を主動線にする
6. **デバイス列挙はネイティブ API で行う**（F7）
7. **WCH-Link は WCH 純正ドライバのままアクセス可能。ドライバの差し替えを求めない**（F6）
8. **attach 中は Windows からデバイスに触れない**（F4）
   → プローブのタイミングは attach 前に限られる
9. **bind には管理者権限が必要、attach には不要**（F5）
   → GUI 全体を昇格させない設計にする

---

## 参考資料

- [board-identify](https://github.com/tanakamasayuki/board-identify)
- [usbipd-win](https://github.com/dorssel/usbipd-win) / [Automation wiki](https://github.com/dorssel/usbipd-win/wiki/Automation)
- [wsl-usb-gui](https://gitlab.com/alelec/wsl-usb-gui)
- [WinUSB Device (Microsoft Learn)](https://learn.microsoft.com/en-us/windows-hardware/drivers/usbcon/automatic-installation-of-winusb)
- [WCID Devices (libwdi wiki)](https://github.com/pbatard/libwdi/wiki/WCID-Devices)
- [wlink (ch32-rs)](https://github.com/ch32-rs/wlink)
