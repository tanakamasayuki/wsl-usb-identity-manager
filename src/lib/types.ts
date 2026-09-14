/** Mirrors `DeviceView` in src-tauri/src/view.rs. Keep the two in step. */
export interface DeviceView {
  instanceId: string;
  name: string;
  vidPid: string | null;

  // Runtime connection. Displayed, never treated as an identity (R7.1).
  busId: string | null;
  comPort: string | null;
  locationPath: string | null;

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
  probes: ProbeOption[];
}

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
