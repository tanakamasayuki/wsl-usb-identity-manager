/**
 * Ordering the device list by where things are plugged in.
 *
 * Not a second view with its own columns — that loses the state, the connection,
 * the serial number and the board, which is most of what the list is for. This
 * produces **rows for the same table**: the same columns, in hub order, with a
 * depth to indent by and extra rows for ports that have nothing in them.
 */

import type { DeviceView, HubView, PortView, TopologyView } from "./types";

/** One row of the list when it is ordered by topology. */
export type TreeRow =
  | {
      kind: "hub";
      depth: number;
      hub: HubView;
      /** The hub's own device row, when the enumeration has one for it. */
      device: DeviceView | null;
      /** The port it is plugged into, when it is not a root. */
      port: number | null;
    }
  | {
      kind: "device";
      depth: number;
      device: DeviceView;
      /** The hub and port it sits on, when its hub could be asked about them. */
      hub: HubView | null;
      port: PortView | null;
    }
  /** A socket with nothing in it. Only hubs whose power can be switched get one. */
  | { kind: "port"; depth: number; hub: HubView; port: PortView };

/**
 * Instance ids are compared case-insensitively.
 *
 * Windows is not consistent about the case of its own: the same hub is
 * `4&945A1EC` as a device's parent and `4&945a1ec` as its own id. Comparing them
 * literally puts every device on that hub in the wrong place, and does it
 * silently.
 */
function key(instanceId: string | undefined | null): string {
  return (instanceId ?? "").toLowerCase();
}

/**
 * Builds the ordered rows.
 *
 * **The hierarchy comes from the device tree — each device's parent node and
 * port number — and not from location paths.** A location path is a formatted
 * string a device can stop publishing, which is what sharing one with `usbipd`
 * does: the device stays exactly where it is, its path goes, and anything built
 * on the path drops it out of the tree and into the list of things that could
 * not be placed. The parent and the port come from the tree itself and survive.
 *
 * **Anything on a port is listed, whatever the tabs say.** The tree is about
 * where things are plugged in, and a device hidden because the tab happens not
 * to cover its state takes the answer with it — the state is one of the columns,
 * so the row shows it rather than being removed for it. The tabs still govern
 * the devices that are *not* on a port, which are records and forwarded devices
 * rather than sockets.
 */
export function buildTree(
  topology: TopologyView,
  devices: DeviceView[],
  shown: DeviceView[],
): TreeRow[] {
  const hubs = new Map(topology.hubs.map((h) => [key(h.instanceId), h] as const));
  const hubDevice = new Map<string, DeviceView>();

  /**
   * What hangs off each node.
   *
   * Hubs are gathered as well as devices, and not only through them: `usbipd`
   * does not track hubs, so a hub usually has no device row at all and would be
   * invisible to a walk that only followed devices.
   */
  type Child = { device: DeviceView | null; hub: HubView | null; port: number | null };
  const children = new Map<string, Child[]>();
  const add = (parent: string, child: Child) =>
    children.set(parent, [...(children.get(parent) ?? []), child]);

  for (const device of devices) {
    if (hubs.has(key(device.instanceId))) hubDevice.set(key(device.instanceId), device);
    const parent = key(device.parentInstanceId);
    if (!parent) continue;
    const hub = hubs.get(key(device.instanceId)) ?? null;
    add(parent, { device, hub, port: device.portAddress ?? null });
  }
  for (const hub of topology.hubs) {
    // Only the ones no device row already introduced, so a hub `usbipd` does
    // happen to track is not listed twice.
    if (hubDevice.has(key(hub.instanceId))) continue;
    const parent = key(hub.parentInstanceId);
    if (!parent) continue;
    add(parent, { device: null, hub, port: hub.portAddress ?? null });
  }

  const rows: TreeRow[] = [];
  const placed = new Set<string>();

  function emitHub(hub: HubView, depth: number, onPort: number | null) {
    rows.push({
      kind: "hub",
      depth,
      hub,
      device: hubDevice.get(key(hub.instanceId)) ?? null,
      port: onPort,
    });
    const mine = children.get(key(hub.instanceId)) ?? [];
    const done = new Set<Child>();

    function emitChild(child: Child, port: PortView | null, at: number) {
      done.add(child);
      if (child.device) placed.add(child.device.instanceId);
      if (child.hub) emitHub(child.hub, at, child.port);
      else if (child.device) {
        rows.push({ kind: "device", depth: at, device: child.device, hub, port });
      }
    }

    for (const port of hub.ports) {
      const here = mine.filter((c) => c.port === port.port);
      for (const child of here) emitChild(child, port, depth + 1);
      // Only where the power can be switched: on a 16-port root hub the empty
      // rows are noise, and on a switchable hub they are the thing being
      // switched.
      if (here.length === 0 && hub.ppps) {
        rows.push({ kind: "port", depth: depth + 1, hub, port });
      }
    }

    // Anything the hub did not account for — a port it did not report, or a
    // child with no port number — still belongs under it rather than adrift.
    for (const child of mine) {
      if (!done.has(child)) emitChild(child, null, depth + 1);
    }
  }

  // A root is a hub whose own parent is not itself a hub we know about.
  for (const hub of topology.hubs) {
    if (!hubs.has(key(hub.parentInstanceId))) emitHub(hub, 0, null);
  }

  // Attached to WSL, or a sharing record with nothing plugged in: no port to sit
  // on, so they go at the end rather than hung off a guess.
  for (const device of shown) {
    if (!placed.has(device.instanceId)) {
      rows.push({ kind: "device", depth: 0, device, hub: null, port: null });
    }
  }
  return rows;
}

/** Whether every port of a hub is one this application switched off. */
export function allSwitchedOff(hub: HubView): boolean {
  return hub.ports.length > 0 && hub.ports.every((p) => p.switched === "off");
}
