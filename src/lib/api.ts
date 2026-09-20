import { invoke } from "@tauri-apps/api/core";
import { getVersion } from "@tauri-apps/api/app";
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
  TopologyView,
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

/**
 * The version of this build.
 *
 * Comes from `Cargo.toml`, which is the only place it is written (R12.4), by
 * way of Tauri's own `app` command — covered by the core permission set, so it
 * needs no command of ours.
 */
export function appVersion(): Promise<string> {
  return getVersion();
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

/** Brings the window back, for when something has to be asked of the user. */
export function showWindow(): Promise<void> {
  return invoke("show_window");
}

/** The hub tree, with what has been asked of each port. */
export function readTopology(): Promise<TopologyView> {
  return invoke("read_topology");
}

/**
 * Switches a hub port's power.
 *
 * What it did cannot be read back, so what comes back is only that `vhfilter`
 * accepted the request.
 */
export function switchPort(hub: string, port: number, on: boolean): Promise<void> {
  return invoke("switch_port", { hub, port, on });
}

/**
 * Writes the script that fetches vhfilter and installs its filter driver, and
 * opens the folder it went into. Returns where it was written.
 */
export function writeVhfilterSetup(): Promise<string> {
  return invoke("write_vhfilter_setup");
}

/** Switches every port of one hub. */
export function switchHub(hub: string, on: boolean): Promise<void> {
  return invoke("switch_hub", { hub, on });
}

/** Drops every remembered name and identification, returning how many there were. */
export function clearRemembered(): Promise<number> {
  return invoke("clear_remembered");
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

/** Runs `handler` when identify-all was chosen from the tray menu. */
export function onIdentifyAll(handler: () => void): Promise<() => void> {
  return listen("identify-all", handler);
}

/** Runs `handler` when the settings panel was asked for from the tray menu. */
export function onOpenSettings(handler: () => void): Promise<() => void> {
  return listen("open-settings", handler);
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
