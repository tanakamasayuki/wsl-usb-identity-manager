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
    "Devices matching the allow list are identified as soon as they are plugged in, which restarts them.",

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
    "Off by default. A probe restarts the board, so this only runs in the {seconds} seconds after a device arrives, when nothing is using it yet — never at startup, and never on a timer.",
  "settings.allow.label": "Only these VID:PID (comma separated)",
  "settings.allow.placeholder": "1a86:7523, 1a86:55d3",
  "settings.allow.note":
    "Nothing is sent to a device that is not listed here. Devices that report a serial number are skipped: they need no probe.",
  "settings.close": "Close",
  "settings.unsaved": "Settings are not saved yet and reset when the app restarts.",

  "empty.connected": "No USB devices are connected.",
  "empty.shared": "Nothing is shared with usbipd right now.",
  "empty.attached": "No device is attached to WSL.",
  "empty.absent": "No bind records for absent devices.",
  "empty.all": "usbipd reports no devices.",

  "detail.select": "Select a device.",
  "detail.port": "Port",
  "detail.location": "Location",
  "detail.driver": "Driver",
  "detail.transport": "Transport",
  "detail.target": "Target",
  "detail.target.unknown": "Not identified yet. Identifying reads the board's own ID.",
  "detail.target.transport_only": "Identified down to the transport only.",

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
  "probe.running": "Identifying…",

  "probe.esp32.side_effect":
    "Resets the board into its ROM bootloader and then back, so the running firmware restarts.",
  "probe.blocked.no_com_port": "No COM port, so there is no serial line to talk over.",
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
    "対象 VID:PID のデバイスは接続された直後に自動で識別され、ボードが再起動します。",

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
    "既定は無効です。識別はボードを再起動させるため、接続イベントから {seconds} 秒以内、まだ誰も使っていないタイミングに限って実行します。起動時や定期実行では行いません。",
  "settings.allow.label": "対象とする VID:PID（カンマ区切り）",
  "settings.allow.placeholder": "1a86:7523, 1a86:55d3",
  "settings.allow.note":
    "ここに挙げていないデバイスへは何も送信しません。シリアル番号を持つデバイスは識別不要なので対象外です。",
  "settings.close": "閉じる",
  "settings.unsaved": "設定はまだ保存されません。再起動すると既定に戻ります。",

  "empty.connected": "接続中の USB デバイスがありません。",
  "empty.shared": "usbipd で共有中のデバイスはありません。",
  "empty.attached": "WSL に接続中のデバイスはありません。",
  "empty.absent": "未接続の共有記録はありません。",
  "empty.all": "usbipd がデバイスを報告していません。",

  "detail.select": "デバイスを選択してください。",
  "detail.port": "ポート",
  "detail.location": "物理位置",
  "detail.driver": "ドライバ",
  "detail.transport": "Transport",
  "detail.target": "Target",
  "detail.target.unknown": "未識別。識別するとボード自身の ID が判ります。",
  "detail.target.transport_only": "Transport までの識別に留まります。",

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
  "probe.running": "識別中…",

  "probe.esp32.side_effect":
    "ボードを ROM ブートローダに入れてから戻します。動作中のファームウェアが再起動します。",
  "probe.blocked.no_com_port": "COM ポートが無いため、シリアル経由で話しかけられません。",
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
