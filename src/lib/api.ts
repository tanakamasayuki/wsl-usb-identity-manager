import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type {
  Availability,
  DeviceView,
  OpenTarget,
  Operation,
  Settings,
  StoredSettings,
  TargetIdentity,
  TrayView,
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

/**
 * Whether usbipd is installed and answering (R13.1).
 *
 * Runs two processes, so it is called at startup and after a listing fails —
 * not on the polling timer.
 */
export function checkUsbipd(): Promise<Availability> {
  return invoke<Availability>("check_usbipd");
}

/**
 * Asks Windows to open a folder or a page.
 *
 * The target is a name, not a path: what each one resolves to is decided in the
 * backend, so nothing here can point the shell somewhere else.
 */
export function openTarget(target: OpenTarget): Promise<void> {
  return invoke("open_target", { target });
}

/** Puts the current labels and counts into the tray menu. */
export function setTray(view: TrayView): Promise<void> {
  return invoke("set_tray", { view });
}

/** Hides the window, leaving the application running in the tray. */
export function hideWindow(): Promise<void> {
  return invoke("hide_window");
}

/**
 * Runs `handler` when the backend asks the window to close.
 *
 * The backend stops the close and asks, rather than hiding by itself, so the
 * first time it happens the user can be told that closing is not quitting.
 */
export function onCloseRequested(handler: () => void): Promise<() => void> {
  return listen("close-requested", handler);
}

/** Runs `handler` when the settings changed outside the window — from the tray. */
export function onSettingsChanged(handler: () => void): Promise<() => void> {
  return listen("settings-changed", handler);
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
