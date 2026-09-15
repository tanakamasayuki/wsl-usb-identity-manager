/**
 * Translation, picked from the locale the OS reports.
 *
 * No i18n library: the catalogue is small and the lookup is a map read, so a
 * dependency would cost more than it saves. Messages that originate in Rust
 * arrive as keys (`probe.blocked.no_com_port`) rather than as English prose,
 * so re-wording the backend cannot silently break a translation.
 */

const en = {
  "toolbar.refresh": "Refresh",
  "toolbar.settings": "Settings",
  "toolbar.auto_on": "Auto-identify on",
  "toolbar.auto_on.hint":
    "A device with no serial number is identified as soon as it is plugged in, which restarts the board. Excluded VID:PID are left alone.",

  "filter.connected": "Connected",
  "filter.shared": "Shared",
  "filter.attached": "In WSL",
  "filter.absent": "Records only",
  "filter.all": "All",

  "col.state": "State",
  "col.connection": "Connection",
  "col.device": "Device",
  "col.vidpid": "VID:PID",
  "col.transport": "Transport",
  "col.target": "Target",

  "state.shared": "Shared",
  "state.attached": "In WSL",
  "state.absent": "Absent",

  "transport.none": "none",
  "transport.hint":
    "Serial number of the adapter or probe itself. It stays the same when the board behind it is swapped.",
  "transport.none.hint":
    "This adapter reports no serial number, so its own identity rests on which port it is in.",

  "target.hint": "Read from the board itself, so it follows the board between adapters and ports.",
  "target.unidentified": "not identified",
  "target.unidentified.hint": "Click to identify. The board restarts.",
  "target.identifying": "identifying…",
  "target.unavailable.hint": "Nothing here can be identified beyond the transport.",

  "settings.title": "Automatic identification",
  "settings.auto.label": "Identify a device right after it is plugged in",
  "settings.auto.note":
    "A probe restarts the board, so this only runs in the {seconds} seconds after a device arrives, when nothing is using it yet — never at startup, and never on a timer.",
  "settings.confirm.label": "Ask before identifying a device",
  "settings.confirm.note":
    "Off skips the warning and identifies straight away. The board still restarts; you just stop being told.",
  "settings.startup.label": "Start with Windows",
  "settings.startup.note":
    "Adds an entry under the current user's Run key. No administrator rights, and no effect on other accounts. There is no tray icon yet, so the window opens at every sign-in.",
  "settings.exclude.label": "Never identify these VID:PID (comma separated)",
  "settings.exclude.placeholder": "1a86:7523, 10c4:ea60",
  "settings.exclude.note":
    "Everything else that has no serial number is identified on arrival. List the hardware that must not be disturbed — a USB-serial adapter wired to equipment rather than to a dev board, for instance. Devices that report a serial number are never probed automatically: they need no probe.",
  "settings.close": "Close",
  "settings.unsaved":
    "The settings file could not be read, so nothing is being saved this session. See the log.",
  "error.dismiss": "Dismiss",
  "toolbar.identify_all": "Identify all",
  "toolbar.identify_all.hint":
    "Identifies every connected device that has not been identified yet. Each one restarts.",
  "toolbar.identify_all.none": "Nothing left to identify",
  "busy.identify_all": "Identifying {done} of {total}…",

  "identify_all.title": "Identify every unidentified device?",
  "identify_all.count": "{count} device(s) will be identified, one after another.",
  "identify_all.warning": "Every one of them restarts.",
  "identify_all.run": "Identify all",

  "empty.connected": "No USB devices are connected.",
  "empty.shared": "Nothing is shared with usbipd right now.",
  "empty.attached": "No device is attached to WSL.",
  "empty.absent": "No bind records for absent devices.",
  "empty.all": "usbipd reports no devices.",

  "detail.select": "Select a device.",
  "detail.port": "Port",
  "detail.location": "Port chain",
  "detail.location.hint":
    "The chain of hub ports the device hangs off, derived from the physical topology rather than from the hub number Windows assigns — that one changes.",
  "detail.problem": "Windows reports a problem with this device (code {code}).",
  "detail.driver": "Driver",
  "detail.vidpid": "VID:PID",
  "detail.vendor": "Vendor",
  "detail.usb_product": "USB product",
  "detail.from_usb_ids": "From the USB ID Repository (usb.ids), which does not list every vendor.",
  "detail.transport": "Transport",
  "detail.target": "Target",
  "detail.target.unknown": "Not identified yet. Identifying reads the board's own ID.",
  "detail.target.transport_only": "Identified down to the transport only.",

  "menu.bind": "Share with usbipd",
  "menu.unbind": "Stop sharing",
  "menu.attach": "Attach to WSL",
  "menu.detach": "Detach from WSL",
  "menu.admin_suffix": " (administrator)",
  "menu.busy": "Working…",
  "busy.bind": "Sharing with usbipd…",
  "busy.unbind": "Stopping sharing…",
  "busy.attach": "Attaching to WSL…",
  "busy.detach": "Detaching from WSL…",
  "busy.probe": "Identifying…",
  "busy.admin": "Waiting for the administrator prompt…",
  "menu.identify": "Identify…",
  "menu.copy_instance_id": "Copy instance ID",
  "menu.copy_identifier": "Copy identifier",
  "menu.copied": "Copied",
  "menu.nothing_to_copy": "No identifier to copy yet",

  "probe.title": "Identify this device?",
  "probe.device": "Device",
  "probe.port": "Port",
  "probe.method": "Method",
  "probe.warning": "Running this affects the device.",
  "probe.cancel": "Cancel",
  "probe.run": "Identify",

  "probe.esp32.side_effect":
    "Resets the board into its ROM bootloader and then back, so the running firmware restarts.",
  "probe.ch32.side_effect":
    "Halts the target core while the probe reads its UUID, then releases it. A program already running on the board is interrupted.",
  "probe.blocked.no_com_port": "No COM port, so there is no serial line to talk over.",
  "probe.blocked.not_a_serial_carrier":
    "A debug probe's own serial port, not a link to a board.",
  "probe.blocked.not_a_wchlink": "Not a WCH-Link.",
  "probe.blocked.wchlink_arm_mode":
    "The WCH-Link is in ARM mode. Switch it to RISC-V mode to identify CH32 parts.",
  "probe.blocked.not_connected": "The device is not connected.",
  "probe.blocked.attached": "Attached to WSL, so Windows cannot reach the device.",
  "probe.blocked.none": "No identification method covers this device.",
};

/**
 * English is the reference catalogue: every other language is typed against its
 * keys, so a message added to `en` fails the type check until it is translated
 * rather than silently falling back at runtime.
 */
type Key = keyof typeof en;

const ja: Record<Key, string> = {
  "toolbar.refresh": "更新",
  "toolbar.settings": "設定",
  "toolbar.auto_on": "自動識別 ON",
  "toolbar.auto_on.hint":
    "シリアル番号を持たないデバイスは接続された直後に自動で識別され、ボードが再起動します。除外した VID:PID には触れません。",

  "filter.connected": "接続中",
  "filter.shared": "共有可能",
  "filter.attached": "WSL 接続中",
  "filter.absent": "記録のみ",
  "filter.all": "すべて",

  "col.state": "状態",
  "col.connection": "接続",
  "col.device": "デバイス",
  "col.vidpid": "VID:PID",
  "col.transport": "Transport",
  "col.target": "Target",

  "state.shared": "共有可能",
  "state.attached": "WSL 接続中",
  "state.absent": "未接続",

  "transport.none": "なし",
  "transport.hint":
    "アダプタ／プローブ自身のシリアル番号です。その先のボードを載せ替えても変わりません。",
  "transport.none.hint":
    "このアダプタはシリアル番号を申告しないため、アダプタ自身もポート位置でしか特定できません。",

  "target.hint": "ボード自身から読み出した識別子です。アダプタやポートを変えても追随します。",
  "target.unidentified": "未識別",
  "target.unidentified.hint": "クリックすると識別します。ボードが再起動します。",
  "target.identifying": "識別中…",
  "target.unavailable.hint": "このデバイスは Transport までの識別に留まります。",

  "settings.title": "自動識別",
  "settings.auto.label": "接続された直後に自動で識別する",
  "settings.auto.note":
    "識別はボードを再起動させるため、接続イベントから {seconds} 秒以内、まだ誰も使っていないタイミングに限って実行します。起動時や定期実行では行いません。",
  "settings.confirm.label": "識別の前に確認する",
  "settings.confirm.note":
    "オフにすると警告を出さずに即座に識別します。ボードが再起動することは変わりません。知らされなくなるだけです。",
  "settings.startup.label": "Windows と一緒に起動する",
  "settings.startup.note":
    "現在のユーザーの Run キーに登録します。管理者権限は不要で、他のアカウントには影響しません。タスクトレイ常駐はまだ無いので、サインインのたびにウィンドウが開きます。",
  "settings.exclude.label": "識別しない VID:PID（カンマ区切り）",
  "settings.exclude.placeholder": "1a86:7523, 10c4:ea60",
  "settings.exclude.note":
    "ここに挙げたもの以外で、シリアル番号を持たないデバイスは接続時に識別されます。触られては困る機器を挙げてください（開発ボードではなく装置に繋がった USB シリアル変換など）。シリアル番号を持つデバイスは識別不要なので、自動識別の対象になりません。",
  "settings.close": "閉じる",
  "settings.unsaved":
    "設定ファイルを読めなかったため、今回は何も保存されません。ログを確認してください。",
  "error.dismiss": "閉じる",
  "toolbar.identify_all": "一括識別",
  "toolbar.identify_all.hint":
    "接続中で未識別のデバイスをすべて識別します。対象はいずれも再起動します。",
  "toolbar.identify_all.none": "識別するものがありません",
  "busy.identify_all": "識別しています（{done} / {total}）…",

  "identify_all.title": "未識別のデバイスをすべて識別しますか？",
  "identify_all.count": "{count} 台を順に識別します。",
  "identify_all.warning": "対象はいずれも再起動します。",
  "identify_all.run": "すべて識別する",

  "empty.connected": "接続中の USB デバイスがありません。",
  "empty.shared": "usbipd で共有中のデバイスはありません。",
  "empty.attached": "WSL に接続中のデバイスはありません。",
  "empty.absent": "未接続の共有記録はありません。",
  "empty.all": "usbipd がデバイスを報告していません。",

  "detail.select": "デバイスを選択してください。",
  "detail.port": "ポート",
  "detail.location": "ポート",
  "detail.location.hint":
    "デバイスがぶら下がっているハブポートの連鎖です。Windows が振るハブ番号ではなく物理トポロジから導いているので、再認識で変わりません。",
  "detail.problem": "Windows がこのデバイスに問題を報告しています（コード {code}）。",
  "detail.driver": "ドライバ",
  "detail.vidpid": "VID:PID",
  "detail.vendor": "ベンダー",
  "detail.usb_product": "USB 製品名",
  "detail.from_usb_ids": "USB ID Repository（usb.ids）の記載です。全ベンダーが登録しているわけではありません。",
  "detail.transport": "Transport",
  "detail.target": "Target",
  "detail.target.unknown": "未識別。識別するとボード自身の ID が判ります。",
  "detail.target.transport_only": "Transport までの識別に留まります。",

  "menu.bind": "usbipd で共有する",
  "menu.unbind": "共有をやめる",
  "menu.attach": "WSL に接続する",
  "menu.detach": "WSL から切り離す",
  "menu.admin_suffix": "（管理者）",
  "menu.busy": "実行中…",
  "busy.bind": "usbipd で共有しています…",
  "busy.unbind": "共有を解除しています…",
  "busy.attach": "WSL に接続しています…",
  "busy.detach": "WSL から切り離しています…",
  "busy.probe": "識別しています…",
  "busy.admin": "管理者の許可を待っています…",
  "menu.identify": "識別…",
  "menu.copy_instance_id": "インスタンス ID をコピー",
  "menu.copy_identifier": "識別子をコピー",
  "menu.copied": "コピーしました",
  "menu.nothing_to_copy": "コピーできる識別子がまだありません",

  "probe.title": "このデバイスを識別しますか？",
  "probe.device": "デバイス",
  "probe.port": "ポート",
  "probe.method": "方式",
  "probe.warning": "実行するとデバイスに影響があります。",
  "probe.cancel": "キャンセル",
  "probe.run": "識別する",

  "probe.esp32.side_effect":
    "ボードを ROM ブートローダに入れてから戻します。動作中のファームウェアが再起動します。",
  "probe.ch32.side_effect":
    "UUID を読み出す間、ターゲットのコアを停止させてから解放します。動作中のプログラムは中断されます。",
  "probe.blocked.no_com_port": "COM ポートが無いため、シリアル経由で話しかけられません。",
  "probe.blocked.not_a_serial_carrier":
    "デバッグプローブ自身のシリアルポートです。ボードへの経路ではありません。",
  "probe.blocked.not_a_wchlink": "WCH-Link ではありません。",
  "probe.blocked.wchlink_arm_mode":
    "WCH-Link が ARM モードです。CH32 を識別するには RISC-V モードに切り替えてください。",
  "probe.blocked.not_connected": "デバイスが接続されていません。",
  "probe.blocked.attached": "WSL に接続中のため、Windows からデバイスに触れません。",
  "probe.blocked.none": "このデバイスに対応する識別方式がありません。",
};

const CATALOGS: Record<string, Partial<Record<Key, string>>> = { en, ja };

/** Picks the first language the catalogue covers, matching on the base tag. */
function pickLocale(tags: readonly string[]): string {
  for (const tag of tags) {
    const base = tag.toLowerCase().split("-")[0];
    if (base in CATALOGS) return base;
  }
  return "en";
}

export const locale = pickLocale(
  navigator.languages?.length ? navigator.languages : [navigator.language],
);

/**
 * `key` is loosely typed because keys also arrive from the backend
 * (`probe.blocked.no_com_port`). An unknown key renders as itself, which keeps
 * a gap visible instead of blank.
 */
export function t(key: string, params?: Record<string, string | number>): string {
  const text = CATALOGS[locale]?.[key as Key] ?? en[key as Key] ?? key;
  if (!params) return text;
  return text.replace(/\{(\w+)\}/g, (_, name: string) =>
    name in params ? String(params[name]) : `{${name}}`,
  );
}
