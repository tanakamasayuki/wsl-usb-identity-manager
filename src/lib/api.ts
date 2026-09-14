import { invoke } from "@tauri-apps/api/core";
import type {
  DeviceView,
  Operation,
  Settings,
  StoredSettings,
  TargetIdentity,
} from "./types";

/**
 * Writes to the application log.
 *
 * Fire-and-forget: a failure to log must never replace the problem being
 * logged, and never throws back into the caller's error path.
 */
export function log(level: "info" | "error", message: string): void {
  void invoke("log_message", { level, message }).catch(() => {});
}

/** Reads the saved settings, and whether saving is working at all. */
export function readSettings(): Promise<StoredSettings> {
  return invoke<StoredSettings>("read_settings");
}

/** Saves the settings. */
export function writeSettings(settings: Settings): Promise<void> {
  return invoke("write_settings", { settings });
}

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

/**
 * Runs a usbipd operation and returns the state that followed it.
 *
 * `bind` and `unbind` raise a UAC prompt; the promise settles once the user has
 * answered it and the command has finished.
 */
export function runOperation(
  instanceId: string,
  operation: Operation,
): Promise<DeviceView[]> {
  return invoke<DeviceView[]>("run_operation", { instanceId, operation });
}
