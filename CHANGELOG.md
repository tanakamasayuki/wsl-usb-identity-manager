# Changelog

## Unreleased

## 1.1.2 - 2026-09-18

- (EN) **Identifier change for the original ESP32.** It is now named the way esptool and board-identify name it — `esp32-d0wd-v3-<mac>` rather than `esp32-<mac>` — by reading the package eFuse that espflash does not expose. One board had two names across the two tools, which is the confusion this application exists to remove. An automatic-attach rule that matched an original ESP32 by board ID has to be added again; ESP32-S2 and later are unaffected, because there the series already was the chip name.
- (JA) **初代 ESP32 の識別子が変わります。** esptool / board-identify と同じ名前になります——`esp32-<mac>` ではなく `esp32-d0wd-v3-<mac>`。espflash が返さないパッケージの eFuse を追加で 1 ワード読んでいます。同じボードが 2 つのツールで別の名前になっていたためです。ボード ID で初代 ESP32 を指定していた自動 Attach の条件は登録し直しが必要です。ESP32-S2 以降は元々シリーズ名がチップ名なので変わりません。

## 1.1.1 - 2026-09-18

- (EN) Names a board from its USB descriptors alone, where the VID:PID is one a vendor programmed for that board and the device reports a serial number. Nothing is sent to the board, so this needs no confirmation and no grace window; it runs on every refresh, and answers for a device already attached to WSL. The table comes from board-identify, so both tools give one board one name. Stock USB-UART bridge IDs are refused: a CH340 names the cable, not what is on the end of it.
- (JA) VID:PID がそのボードのために発行されたもので、かつシリアル番号を持つデバイスを、USB ディスクリプタだけで識別する。ボードには何も送らないため確認も猟時間も不要で、毎回の更新で評価され、WSL へ Attach 中のデバイスにも答えられる。対応表は board-identify から取り込むので、同じボードを両ツールが同じ名前で呼ぶ。素の USB-UART ブリッジの ID は拒否する——CH340 が指すのはケーブルであって、その先ではない。
- (EN) CH32V006 can be identified. `ch32rv` 0.7 did not know the V00x line's `AttachChip` family byte, so the read never started, and its device database had no row for the CH32V006K8U6 even if it had. Both are fixed in 0.8.
- (JA) CH32V006 を識別できるようになった。`ch32rv` 0.7 は V00x 系の `AttachChip` ファミリバイトを知らず読み出しが始まらず、デバイス DB にも CH32V006K8U6 の行が無かった。0.8 で両方解決している。
- (EN) Fix: the CH32 silicon revision was read from the wrong nibble. It is bits 4..7 — the ones the device database masks off before matching, and stores zeroed in every SKU — not the top nibble, which varies between families rather than between revisions of a part. Display only; no identity key changes.
- (JA) 修正: CH32 のシリコンリビジョンを誤ったニブルから読んでいた。正しくは bit 4..7——デバイス DB が照合前にマスクし、全 SKU で 0 として保持しているニブルである。上位ニブルはリビジョンではなくファミリを分ける値だった。表示のみの修正で、識別子は変わらない。

## 1.1.0 - 2026-09-18

- (EN) Shows what a device was last identified as, greyed and dated, where there is no current answer, and keeps it across a restart — a device already attached to WSL cannot be probed at all, so without it a machine that starts with its boards forwarded can say nothing about them. A confirmed identity is still dropped on disconnect and still never saved as one, and the greyed value is never matched by an automatic-attach rule. In the list it doubles as the button that re-confirms it.
- (JA) 識別結果が無いデバイスに、最後に識別できた内容を日時付きのグレーで表示し、再起動しても残す。Attach 中のデバイスはそもそも識別できないため、これが無いとボードを Attach したまま起動した PC は何も答えられない。確定した識別結果を切断で破棄することも、確定情報としては保存しないことも変わらない。グレーの値は自動 Attach の条件には一致しない。一覧ではそのまま再識別のボタンを兼ねる。
- (EN) Keeps the name a device had before it was handed to WSL. An attached device is re-enumerated as the forwarding stub, so the name Windows had for it is gone; the previous one is shown beside whatever is left.
- (JA) WSL へ渡す前のデバイス名を保持する。Attach 中のデバイスは転送用スタブとして再列挙されるため Windows 側の名前が失われる。残っている名前の横に、それまでの名前を併記する。
- (EN) Says in the settings how many devices are remembered, and clears them on request. This is the one part of the file that can be wrong without anything having gone wrong: move a board to another port and its old entry stays behind. The oldest is dropped past 200 devices.
- (JA) 記憶しているデバイスの件数を設定画面に示し、まとめて消去できるようにした。ここだけは何も壊れていなくても間違いうる——ボードを別のポートへ移せば、前の記録はその場に残る。200 件を超えたら古いものから捨てる。

## 1.0.1 - 2026-09-16

- (EN) Names the publisher in the installer. Without it Tauri falls back to the second element of the bundle identifier, so the installed application reported its publisher as `github` in the uninstall list and to anything reading the installer's metadata.
- (JA) インストーラに発行元を明記した。未指定だと Tauri は identifier の 2 番目の要素を使うため、インストール済みアプリの発行元が `github` と表示されていた。

## 1.0.0 - 2026-09-16

- (EN) Lists what Windows enumerates and what `usbipd` has a record of in one list, filtered by state, and runs bind, unbind, attach and detach from it. The administrator prompt appears only for the operations that need one; the application itself runs unelevated.
- (JA) Windows が列挙しているデバイスと `usbipd` が記録しているデバイスを 1 つのリストに表示し、状態で絞り込み、bind / unbind / attach / detach をそこから実行する。管理者の確認は必要な操作のときだけ出て、アプリ自身は昇格せずに動く。
- (EN) Identifies the board behind an adapter or a debug probe: an ESP32 by its eFuse MAC, a CH32 RISC-V part by its factory UUID through a WCH-Link. An identity lasts while the device stays plugged in and is never stored — nothing in USB says the board behind an adapter is still the same one after it has been unplugged.
- (JA) アダプタやデバッグプローブの先にあるボードを識別する。ESP32 は eFuse MAC、CH32 RISC-V は WCH-Link 経由で読んだ工場出荷時 UUID による。識別結果は接続されている間だけ保持し、保存しない。アダプタの先のボードが抜き挿しを跨いで同じものかどうかは、USB からは判らないため。
- (EN) Identifies a device automatically in the seconds after it is plugged in, before anything has opened it, with an exclusion list for hardware that must not be disturbed. Identify-all covers what that leaves out.
- (JA) 接続直後のまだ誰も使っていない数秒の間に自動で識別する。触られては困る機器は VID:PID で除外できる。対象から外れたものは一括識別で拾える。
- (EN) Attaches devices to WSL by rule, naming them by board ID, USB serial number, VID:PID or bus id. It never shares a device by itself, and never identifies one in order to decide whether to attach it.
- (JA) 条件に一致したデバイスを自動で WSL に接続する。条件はボード ID / USB シリアル番号 / VID:PID / BUSID のいずれか。自動で共有(bind)することはなく、接続の可否を決めるために識別を実行することもない。
- (EN) Stays in the notification area. Closing the window does not quit, because automatic attach and identification on arrival only work while the application is running; the tray menu carries the counts, an auto-attach switch, identify-all, the settings, and quit. Set to start with Windows, it goes straight to the tray without opening a window.
- (JA) 通知領域に常駐する。自動 Attach と接続直後の自動識別は動いている間しか効かないため、ウィンドウを閉じても終了しない。トレイメニューには台数、自動 Attach の切り替え、一括識別、設定、終了を置く。Windows と一緒に起動する設定にすると、ウィンドウを開かずトレイに入る。
- (EN) Shows which build it is, next to the path to the log — a problem report needs both, and the log's first line carries the version too.
- (JA) どのビルドかを、ログの出力先と同じ場所に表示する。不具合の報告には両方が要るため。ログの 1 行目にもバージョンを記録する。
- (EN) Says what is missing rather than failing operation by operation: the WebView2 runtime is checked before a window is built, and a missing or stopped usbipd is reported above the list with a way to get it. The log and the settings file can be opened from the settings screen.
- (JA) 足りないものを、操作ごとの失敗ではなくそれとして伝える。WebView2 ランタイムはウィンドウを作る前に確認し、usbipd の未インストールや停止は一覧の上に入手先とともに示す。ログと設定ファイルは設定画面から開ける。
- (EN) Follows the OS display language, in English and Japanese. Vendor names come from the `usb.ids` that `usbipd-win` already installs, which is read rather than shipped.
- (JA) OS の表示言語に従い、英語と日本語で表示する。ベンダ名は `usbipd-win` が同梱している `usb.ids` を読んで表示する(同梱はしない)。
- (EN) Documentation is kept in English and Japanese as a pair, and holds what was decided with the facts behind it — the measurements the design rests on, the identification policy, the requirements and the release procedure.
- (JA) ドキュメントは英語版と日本語版の対で維持し、決まったこととその根拠の事実を書く。設計の根拠となる実測、識別ポリシー、要件定義、リリース手順。

## 0.2.0 - 2026-09-16

- (EN) Experimental pre-release: automatic attach by rule, refusing to run a second copy, and a set of interface corrections.
- (JA) 実験リリース。条件による自動 Attach、多重起動の防止、UI の修正。

## 0.1.2 - 2026-09-15

- (EN) Experimental pre-release: the device list, the `usbipd` operations, and target identification for ESP32 and CH32 RISC-V.
- (JA) 実験リリース。デバイス一覧、`usbipd` の操作、ESP32 と CH32 RISC-V の識別。
