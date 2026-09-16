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
  "toolbar.auto_attach": "Auto-attach",
  "toolbar.on": "ON",
  "toolbar.off": "OFF",
  "toolbar.auto_attach.hint":
    "Hands a device to WSL as soon as a rule matches it and it is shared. Click to turn on or off.",
  "toolbar.auto_attach.rules": "Rules",
  "toolbar.auto_attach.rules.hint": "Which devices are attached automatically.",
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
  "col.transport": "USB serial number",
  "col.target": "Board",
  "col.auto": "Auto",

  "state.shared": "Shared",
  "state.attached": "In WSL",
  "state.absent": "Absent",

  "transport.none": "none",
  "transport.hint":
    "The serial number in the USB descriptors — not the serial line. It belongs to the adapter or probe itself, and stays the same when the board behind it is swapped.",
  "transport.none.hint":
    "This adapter reports no serial number, so its own identity rests on which port it is in.",

  "target.hint": "Read from the board itself, so it follows the board between adapters and ports.",
  "target.unidentified": "not identified",
  "target.unidentified.hint": "Click to identify. The board restarts.",
  "target.identifying": "identifying…",
  "target.unavailable.hint": "Nothing here can be identified beyond the USB side.",

  "auto_attach.title": "Automatic attach",
  "auto_attach.enable": "Attach matching devices to WSL automatically",
  "auto_attach.enable.note":
    "A device is attached when a rule below names it, usbipd is already sharing it, and Windows can see it. Sharing still needs administrator rights, so a device that has never been shared is left alone.",
  "auto_attach.detached.note":
    "Detaching a device by hand stops it being attached again until it is unplugged and plugged back in. An automatic rule should not undo what you just did.",
  "auto_attach.rules": "Rules",
  "auto_attach.empty": "No rules yet, so nothing is attached automatically.",
  "auto_attach.col.kind": "Match on",
  "auto_attach.col.value": "Value",
  "auto_attach.col.matches": "Now",
  "auto_attach.matches": "{count} connected",
  "auto_attach.matches.none": "nothing connected",
  "auto_attach.add": "Add",
  "auto_attach.remove": "Remove",
  "auto_attach.close": "Close",
  "auto_attach.none": "None",

  "auto_attach.kind.identity": "Board ID",
  "auto_attach.kind.serial": "USB serial number",
  "auto_attach.kind.vid_pid": "VID:PID",
  "auto_attach.kind.bus_id": "BUSID",
  "auto_attach.kind.identity.hint":
    "The board's own ID. It follows the board between adapters and ports, but is only known once the board has been identified — so this rule waits for that, and never sets off an identification itself.",
  "auto_attach.kind.serial.hint":
    "The USB serial number. For an adapter or a debug probe this names the adapter, not the board behind it.",
  "auto_attach.kind.vid_pid.hint": "Every device of this kind, e.g. every CH340.",
  "auto_attach.kind.bus_id.hint":
    "Whatever is at this bus id. The bus id is not a stable name: plugging in a hub renumbers it, and the same bus id then means a different device.",
  "auto_attach.example.identity": "esp32-s3-3485188f6d7c",
  "auto_attach.example.serial": "5B5F090816",
  "auto_attach.example.vid_pid": "1a86:7523",
  "auto_attach.example.bus_id": "12-3",
  "auto_attach.marked": "Attached automatically, matched on {kind}",
  "auto_attach.marked.off":
    "Matches the {kind} rule, but automatic attach is switched off.",
  "auto_attach.removed_shared":
    "Removed the {kind} rule, which also covered {count} other device(s).",
  "state.attaching": "attaching…",

  "detail.auto_attach": "Auto-attach",
  "detail.auto_attach.off": "Auto-attach is off, so these rules are not acted on.",
  "detail.auto_attach.none": "Not attached automatically.",

  "app.name": "WSL USB Identity Manager",
  "tray.status": "Connected {connected} / shared {shared} / in WSL {attached}",
  "tray.open": "Open",
  "tray.quit": "Quit",
  "tray.notice.title": "Closing leaves this running",
  "tray.notice.body":
    "The window closes to the notification area. It keeps running there because automatic attach and automatic identification only work while it does. Quit from the tray icon when you want it stopped.",
  "tray.notice.ok": "Got it",

  "usbipd.not_installed": "usbipd-win is not installed.",
  "usbipd.not_installed.what":
    "This application drives usbipd-win; without it there is nothing to list or attach. Install it, then press Refresh.",
  "usbipd.not_answering": "usbipd is installed but did not answer.",
  "usbipd.not_answering.what":
    "Usually the usbipd service is stopped. Start it from Services, or reinstall usbipd-win.",
  "usbipd.get": "Get usbipd-win",
  "usbipd.old": "This usbipd is older than the one this application was written against.",
  "usbipd.old.what":
    "Found {version}. The state it reports may be shaped differently, so some devices can read wrongly. Updating usbipd-win is the fix.",

  "settings.files": "Files",
  "settings.log": "Log",
  "settings.settings_file": "Settings",
  "settings.open_folder": "Open folder",

  "settings.title": "Automatic identification",
  "settings.auto.label": "Identify a device right after it is plugged in",
  "settings.auto.note":
    "A probe restarts the board, so this only runs in the {seconds} seconds after a device arrives, when nothing is using it yet — never at startup, and never on a timer.",
  "settings.confirm.label": "Ask before identifying a device",
  "settings.confirm.note":
    "Off skips the warning and identifies straight away. The board still restarts; you just stop being told.",
  "settings.startup.label": "Start with Windows",
  "settings.startup.note":
    "Adds an entry under the current user's Run key. No administrator rights, and no effect on other accounts. Started this way it goes straight to the notification area, without opening a window.",
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
  "detail.transport": "USB serial number",
  "detail.target": "Board",
  "detail.target.unknown": "Not identified yet. Identifying reads the board's own ID.",
  "detail.target.transport_only": "Identified down to the USB side only.",

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
  "toolbar.auto_attach": "自動 Attach",
  "toolbar.on": "ON",
  "toolbar.off": "OFF",
  "toolbar.auto_attach.hint":
    "条件に一致し、共有可能になっているデバイスを自動で WSL に接続します。クリックで切り替わります。",
  "toolbar.auto_attach.rules": "条件",
  "toolbar.auto_attach.rules.hint": "どのデバイスを自動で接続するかの一覧です。",
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
  "col.transport": "USB シリアル番号",
  "col.target": "ボード",
  "col.auto": "自動",

  "state.shared": "共有可能",
  "state.attached": "WSL 接続中",
  "state.absent": "未接続",

  "transport.none": "なし",
  "transport.hint":
    "USB 記述子に入っているシリアル番号です（シリアル回線のことではありません）。アダプタ／プローブ自身のもので、その先のボードを載せ替えても変わりません。",
  "transport.none.hint":
    "このアダプタはシリアル番号を申告しないため、アダプタ自身もポート位置でしか特定できません。",

  "target.hint": "ボード自身から読み出した識別子です。アダプタやポートを変えても追随します。",
  "target.unidentified": "未識別",
  "target.unidentified.hint": "クリックすると識別します。ボードが再起動します。",
  "target.identifying": "識別中…",
  "target.unavailable.hint": "このデバイスは USB 側までの識別に留まります。",

  "auto_attach.title": "自動 Attach",
  "auto_attach.enable": "条件に一致したデバイスを自動で WSL に接続する",
  "auto_attach.enable.note":
    "下の条件に一致し、usbipd で共有済みで、Windows から見えているデバイスを接続します。共有には管理者権限が要るため、一度も共有していないデバイスには手を出しません。",
  "auto_attach.detached.note":
    "手動で切り離したデバイスは、抜き挿しするまで再接続しません。自分でやった操作を自動処理が取り消すべきではないためです。",
  "auto_attach.rules": "条件",
  "auto_attach.empty": "条件がありません。自動接続は行われません。",
  "auto_attach.col.kind": "種類",
  "auto_attach.col.value": "値",
  "auto_attach.col.matches": "現在",
  "auto_attach.matches": "接続中 {count} 台",
  "auto_attach.matches.none": "一致なし",
  "auto_attach.add": "追加",
  "auto_attach.remove": "削除",
  "auto_attach.close": "閉じる",
  "auto_attach.none": "なし",

  "auto_attach.kind.identity": "ボード ID",
  "auto_attach.kind.serial": "USB シリアル番号",
  "auto_attach.kind.vid_pid": "VID:PID",
  "auto_attach.kind.bus_id": "BUSID",
  "auto_attach.kind.identity.hint":
    "ボード自身の ID です。アダプタやポートを変えても追随しますが、識別済みのときしか判りません。この条件は識別されるまで待つだけで、識別を実行することはありません。",
  "auto_attach.kind.serial.hint":
    "USB のシリアル番号です。アダプタやデバッグプローブの場合は、その先のボードではなくアダプタ自身を指します。",
  "auto_attach.kind.vid_pid.hint": "同じ種類のデバイスすべてです（CH340 全部、など）。",
  "auto_attach.kind.bus_id.hint":
    "その BUSID にあるものを接続します。BUSID は安定した名前ではありません。ハブを挿すと番号が振り直され、同じ BUSID が別のデバイスを指すようになります。",
  "auto_attach.example.identity": "esp32-s3-3485188f6d7c",
  "auto_attach.example.serial": "5B5F090816",
  "auto_attach.example.vid_pid": "1a86:7523",
  "auto_attach.example.bus_id": "12-3",
  "auto_attach.marked": "{kind} の条件に一致し、自動で接続されます",
  "auto_attach.marked.off": "{kind} の条件に一致していますが、自動 Attach は OFF です",
  "auto_attach.removed_shared":
    "{kind} の条件を削除しました。他に {count} 台が一致していました。",
  "state.attaching": "接続中…",

  "detail.auto_attach": "自動 Attach",
  "detail.auto_attach.off": "自動 Attach が OFF のため、条件は実行されません。",
  "detail.auto_attach.none": "自動接続しません。",

  "app.name": "WSL USB Identity Manager",
  "tray.status": "接続中 {connected} / 共有 {shared} / WSL {attached}",
  "tray.open": "開く",
  "tray.quit": "終了",
  "tray.notice.title": "閉じても終了しません",
  "tray.notice.body":
    "ウィンドウは通知領域に収まります。自動 Attach と接続直後の自動識別は動いている間しか効かないため、そこで動き続けます。止めるときはトレイアイコンから終了してください。",
  "tray.notice.ok": "わかりました",

  "usbipd.not_installed": "usbipd-win がインストールされていません。",
  "usbipd.not_installed.what":
    "本アプリは usbipd-win を操作するツールです。これが無いと、一覧に出すものも接続するものもありません。インストールしてから「更新」を押してください。",
  "usbipd.not_answering": "usbipd はありますが、応答しません。",
  "usbipd.not_answering.what":
    "多くの場合 usbipd サービスが停止しています。サービスから開始するか、usbipd-win を再インストールしてください。",
  "usbipd.get": "usbipd-win を入手",
  "usbipd.old": "本アプリが想定しているより古い usbipd です。",
  "usbipd.old.what":
    "検出したのは {version} です。state の構造が異なる可能性があり、一部のデバイスが正しく読めないことがあります。usbipd-win の更新で解決します。",

  "settings.files": "ファイル",
  "settings.log": "ログ",
  "settings.settings_file": "設定",
  "settings.open_folder": "フォルダを開く",

  "settings.title": "自動識別",
  "settings.auto.label": "接続された直後に自動で識別する",
  "settings.auto.note":
    "識別はボードを再起動させるため、接続イベントから {seconds} 秒以内、まだ誰も使っていないタイミングに限って実行します。起動時や定期実行では行いません。",
  "settings.confirm.label": "識別の前に確認する",
  "settings.confirm.note":
    "オフにすると警告を出さずに即座に識別します。ボードが再起動することは変わりません。知らされなくなるだけです。",
  "settings.startup.label": "Windows と一緒に起動する",
  "settings.startup.note":
    "現在のユーザーの Run キーに登録します。管理者権限は不要で、他のアカウントには影響しません。この経路で起動したときはウィンドウを開かず、通知領域に入ります。",
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
  "detail.transport": "USB シリアル番号",
  "detail.target": "ボード",
  "detail.target.unknown": "未識別。識別するとボード自身の ID が判ります。",
  "detail.target.transport_only": "USB 側までの識別に留まります。",

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
