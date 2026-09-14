import { invoke } from "@tauri-apps/api/core";
import type { DeviceView, TargetIdentity } from "./types";

/** Reads the current state. Probes nothing, so it is safe to poll. */
export function listDevices(): Promise<DeviceView[]> {
  return invoke<DeviceView[]>("list_devices");
}

/**
 * Asks a board what it is.
 *
 * Has side effects — only call this from a user action that has already shown
 * the `sideEffect` text of the probe being run (R4.7).
 */
export function probeDevice(
  instanceId: string,
  family: string,
): Promise<TargetIdentity> {
  return invoke<TargetIdentity>("probe_device", { instanceId, family });
}
