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
  | { kind: "hub"; depth: number; hub: HubView; device: DeviceView | null; port: number | null }
  | {
      kind: "device";
      depth: number;
      device: DeviceView;
      /** The hub and port it sits on, when it is on one. */
      hub: HubView | null;
      port: PortView | null;
    }
  | {
      kind: "port";
      depth: number;
      hub: HubView;
      port: PortView;
      /** Present when something is plugged in that the filter excluded. */
      hidden: DeviceView | null;
    };

/**
 * Builds the ordered rows.
 *
 * `devices` is every device, which is what decides whether a socket is occupied;
 * `shown` is what the filter tabs let through, which is what decides whether a
 * row is listed. Conflating the two would let a filter make an occupied port
 * look empty.
 */
export function buildTree(
  topology: TopologyView,
  devices: DeviceView[],
  shown: DeviceView[],
): TreeRow[] {
  const byPath = new Map(
    devices.filter((d) => d.locationPath).map((d) => [d.locationPath!, d] as const),
  );
  const hubByPath = new Map(
    topology.hubs.filter((h) => h.locationPath).map((h) => [h.locationPath!, h] as const),
  );
  const visible = new Set(shown.map((d) => d.instanceId));

  const rows: TreeRow[] = [];
  const placed = new Set<string>();

  function walk(hub: HubView, depth: number) {
    for (const port of hub.ports) {
      const path = `${hub.locationPath}#USB(${port.port})`;
      const device = byPath.get(path) ?? null;
      const nested = hubByPath.get(path);

      if (nested) {
        rows.push({ kind: "hub", depth, hub: nested, device, port: port.port });
        if (device) placed.add(device.instanceId);
        walk(nested, depth + 1);
        continue;
      }

      if (device && visible.has(device.instanceId)) {
        rows.push({ kind: "device", depth, device, hub, port });
        placed.add(device.instanceId);
        continue;
      }
      if (device) placed.add(device.instanceId);

      // A port with nothing the filter let through is worth a row only when its
      // power can be switched: on a 16-port root hub the empty rows are noise,
      // and on a switchable hub they are the thing being switched.
      if (hub.ppps) {
        rows.push({ kind: "port", depth, hub, port, hidden: device });
      }
    }
  }

  const roots = topology.hubs.filter(
    (h) => !h.locationPath || !h.locationPath.includes("#USB("),
  );
  for (const root of roots) {
    rows.push({ kind: "hub", depth: 0, hub: root, device: null, port: null });
    walk(root, 1);
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
