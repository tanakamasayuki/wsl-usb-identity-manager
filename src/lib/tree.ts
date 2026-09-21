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

/** The path one hop up: `…#USB(2)#USB(1)` is plugged into `…#USB(2)`. */
function parentOf(path: string): string | null {
  const at = Math.max(path.lastIndexOf("#USB("), path.lastIndexOf("#USBMI("));
  return at > 0 ? path.slice(0, at) : null;
}

/** The port number a path ends on, when it ends on one. */
function portOf(path: string): number | null {
  const m = /#USB\((\d+)\)$/.exec(path);
  return m ? Number(m[1]) : null;
}

/**
 * Builds the ordered rows.
 *
 * **The hierarchy comes from the location paths, not from the hub enumeration.**
 * A hub that does not answer its IOCTLs — or one Windows lists in a way this
 * application did not expect — would otherwise take everything plugged into it
 * out of the tree and drop it, unindented and without a port number, into the
 * list of things that could not be placed. The paths are enough on their own;
 * what the hubs add is the empty sockets and the power controls, which is a
 * bonus rather than a prerequisite.
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
  const hubByPath = new Map(
    topology.hubs.filter((h) => h.locationPath).map((h) => [h.locationPath!, h] as const),
  );
  const hubByInstance = new Map(topology.hubs.map((h) => [h.instanceId, h] as const));

  // Every node that has a place, by path. Devices win over hub entries for the
  // same path: the device row carries the columns, the hub entry only the ports.
  const deviceByPath = new Map<string, DeviceView>();
  for (const device of devices) {
    if (device.locationPath) deviceByPath.set(device.locationPath, device);
  }

  const paths = new Set<string>([...deviceByPath.keys(), ...hubByPath.keys()]);
  const children = new Map<string, string[]>();
  const roots: string[] = [];
  for (const path of [...paths].sort()) {
    const parent = parentOf(path);
    if (parent && paths.has(parent)) {
      children.set(parent, [...(children.get(parent) ?? []), path]);
    } else {
      roots.push(path);
    }
  }

  const rows: TreeRow[] = [];
  const placed = new Set<string>();

  function emit(path: string, depth: number) {
    const device = deviceByPath.get(path) ?? null;
    const hub =
      hubByPath.get(path) ?? (device ? (hubByInstance.get(device.instanceId) ?? null) : null);
    const port = portOf(path);

    if (hub) {
      rows.push({ kind: "hub", depth, hub, device, port });
      if (device) placed.add(device.instanceId);
      emitChildren(hub, path, depth + 1);
      return;
    }

    if (device) {
      const parentHub = hubByPath.get(parentOf(path) ?? "") ?? null;
      rows.push({
        kind: "device",
        depth,
        device,
        hub: parentHub,
        port: parentHub?.ports.find((p) => p.port === port) ?? null,
      });
      placed.add(device.instanceId);
    }
    for (const child of children.get(path) ?? []) emit(child, depth + 1);
  }

  /**
   * A hub's children, in port order, with its empty sockets among them.
   *
   * Ordering by the hub's own ports rather than by path keeps port 2 between
   * port 1 and port 3 even when only some of them have anything in them.
   */
  function emitChildren(hub: HubView, path: string, depth: number) {
    const seen = new Set<string>();
    for (const port of hub.ports) {
      const childPath = `${path}#USB(${port.port})`;
      seen.add(childPath);
      if (paths.has(childPath)) {
        emit(childPath, depth);
      } else if (hub.ppps) {
        // Only where the power can be switched: on a 16-port root hub the empty
        // rows are noise, and on a switchable hub they are the thing being
        // switched.
        rows.push({ kind: "port", depth, hub, port });
      }
    }
    // Anything the hub did not account for — an interface node, or a port it
    // did not report — still belongs under it.
    for (const child of children.get(path) ?? []) {
      if (!seen.has(child)) emit(child, depth);
    }
  }

  for (const root of roots) emit(root, 0);

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
