/** Mirrors `DeviceView` in src-tauri/src/view.rs. Keep the two in step. */
export interface DeviceView {
  instanceId: string;
  name: string;
  vidPid: string | null;
  /** Vendor name from the USB ID Repository, when it lists one. */
  vendor: string | null;
  /** Product name from the same source. Listed less often than the vendor. */
  usbProduct: string | null;

  // Runtime connection. Displayed, never treated as an identity (R7.1).
  busId: string | null;
  comPort: string | null;
  locationPath: string | null;
  /** The same path as a short hub-port chain, `1-3-3`. */
  portChain: string | null;

  state: "not shared" | "shared" | "attached" | "absent";
  /** usbipd still reports a bus id. True even while attached. */
  present: boolean;
  /** Windows currently exposes a device node. False while attached. */
  reachable: boolean;
  shared: boolean;
  attached: boolean;
  clientAddress: string | null;

  /** The serial the device reports, when it reports one. */
  serial: string | null;
  identityBasis: "serial" | "port only";
  needsProbe: boolean;

  driver: string | null;
  driverVersion: string | null;
  /** Device revision from the descriptor, `2.64`. */
  revision: string | null;
  /** Windows' problem code, present only when there is one. */
  problemCode: number | null;
  probes: ProbeOption[];
  actions: Actions;
  /** What an auto-attach rule could name this device by, and what names it. */
  autoAttach: AutoAttach;
  /** What a probe found this session, if one has run. */
  identity: Identity | null;
}

/**
 * What an auto-attach rule matches on. Mirrors `RuleKind` in
 * crates/wuim-core/src/autoattach.rs, most specific first.
 */
export type RuleKind = "identity" | "serial" | "vid_pid" | "bus_id";

/** One thing about a device that a rule could name. */
export interface Candidate {
  kind: RuleKind;
  value: string;
}

export interface AutoAttach {
  /**
   * Most specific first, and only what the device actually offers: no serial
   * number means no serial candidate, and no probe has run means no identity
   * candidate.
   */
  candidates: Candidate[];
  /** The kind of rule matching it right now, if any. */
  matched: RuleKind | null;
}

/** One auto-attach rule. Mirrors `Rule` in crates/wuim-core/src/autoattach.rs. */
export interface AutoAttachRule {
  kind: RuleKind;
  value: string;
}

/**
 * What a probe found.
 *
 * Present only while the device it came from stays plugged in. Nothing in USB
 * can vouch that the same board is still on the other end of a cable after it
 * has been unplugged, so the answer is dropped rather than remembered.
 */
export interface Identity {
  identityKey: string;
  deviceType: string;
  deviceId: string;
  hardwareRevision: string | null;
}

export interface Settings {
  autoIdentify: boolean;
  /** `vid:pid` never probed automatically. */
  autoExclude: string[];
  /** Ask before probing, stating what it does to the board (R4.7). */
  confirmBeforeIdentify: boolean;
  /** Start with Windows, through the per-user Run key. */
  startWithWindows: boolean;
  /** Attach matching devices to WSL without being asked (§9). */
  autoAttach: boolean;
  /** What to attach automatically. Empty attaches nothing. */
  autoAttachRules: AutoAttachRule[];
  /** Whether closing to the tray has been explained once. */
  toldAboutTray: boolean;
}

/** What goes in the tray menu, already translated. Mirrors `TrayView` in src-tauri/src/tray.rs. */
export interface TrayView {
  tooltip: string;
  status: string;
  autoAttachLabel: string;
  autoAttachOn: boolean;
  identifyAllLabel: string;
  identifyAllEnabled: boolean;
  openLabel: string;
  settingsLabel: string;
  quitLabel: string;
}

export interface StoredSettings {
  settings: Settings;
  /** False when the file on disk was refused; nothing is being saved. */
  writable: boolean;
  path: string;
  /** Where the log is. A release build has no console to print it to. */
  logPath: string;
}

/**
 * Whether usbipd is installed and answering. Mirrors `Availability` in
 * crates/wuim-core/src/usbipd.rs.
 */
export type Availability =
  | { status: "ok"; version: string; supported: boolean }
  | { status: "not_installed" }
  | { status: "not_answering"; detail: string };

/** Somewhere the backend can point Windows at. */
export type OpenTarget =
  | "log_folder"
  | "settings_folder"
  | "usbipd_releases"
  | "webview2_download";

/** Which usbipd operations make sense for a device as it stands. */
export interface Actions {
  /** Share it with usbipd. Needs administrator rights. */
  bind: boolean;
  /** Stop sharing it. Needs administrator rights. */
  unbind: boolean;
  attach: boolean;
  detach: boolean;
}

export type Operation = keyof Actions;

export interface ProbeOption {
  family: string;
  /** Translation key for the side effects, shown before the probe runs (R4.7). */
  sideEffect: string;
  available: boolean;
  /** Translation key for why it is unavailable. */
  reason: string | null;
}

/** Mirrors `TargetIdentity` in crates/wuim-probe/src/lib.rs. */
export interface TargetIdentity {
  family: string;
  identity_key: string;
  device_id: string;
  device_type: string;
  hardware_revision: string | null;
  details: Record<string, string>;
}
