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
  /** What the stored file says this device is, when it recognises it. */
  identity: Identity | null;
}

/** A recalled or freshly probed identity, with how much it is worth (R4.3). */
export interface Identity {
  identityKey: string;
  deviceType: string;
  deviceId: string;
  hardwareRevision: string | null;
  /**
   * `confirmed` rests on a serial number or a probe just now; `probable` only
   * on the device still sitting in the port it was last seen in; `ambiguous`
   * means the evidence fits more than one device and nothing is claimed.
   */
  confidence: "confirmed" | "probable" | "ambiguous";
}

export interface Settings {
  autoIdentify: boolean;
  /** `vid:pid` never probed automatically. */
  autoExclude: string[];
}

export interface StoredSettings {
  settings: Settings;
  /** False when the file on disk was refused; nothing is being saved. */
  writable: boolean;
  path: string;
}

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
