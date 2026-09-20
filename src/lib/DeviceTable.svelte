<script lang="ts">
  import { t } from "./i18n";
  import type { TreeRow } from "./tree";
  import type { DeviceView, HubView, Identity, PortView } from "./types";

  interface Props {
    devices: DeviceView[];
    /**
     * The same rows, ordered by where things are plugged in.
     *
     * One table either way: switching to the tree must not cost the state, the
     * connection, the serial number or the board, which is most of what anyone
     * came to the list for. What it adds is the order, an indent, and rows for
     * ports with nothing in them.
     */
    rows: TreeRow[] | null;
    selected: string | null;
    /** Shown in place of the table when there is nothing to list. */
    empty: string;
    /** Devices with a probe in flight. */
    probing: Set<string>;
    /** Devices being attached by a rule rather than by the user. */
    attaching: Set<string>;
    /** Whether the rules are being acted on at all, so the marker means something. */
    autoAttachOn: boolean;
    onselect: (instanceId: string) => void;
    onmenu: (instanceId: string, x: number, y: number) => void;
    onidentify: (instanceId: string) => void;
    onswitchport: (hub: string, port: number, on: boolean) => void;
    onswitchhub: (hub: string, on: boolean) => void;
  }

  let {
    devices,
    rows,
    selected,
    empty,
    probing,
    attaching,
    autoAttachOn,
    onselect,
    onmenu,
    onidentify,
    onswitchport,
    onswitchhub,
  }: Props = $props();

  /**
   * Transport and Target are separate columns because they are separate things
   * (requirements §3.1, R3.1). A CH343 reports a serial number and may still
   * have an unidentified board behind it; collapsing the two into one cell
   * would let the adapter's serial stand in for the board's identity, which is
   * the confusion this application exists to remove.
   */
  function transport(device: DeviceView) {
    return device.serial
      ? { text: device.serial, known: true, hint: t("transport.hint") }
      : { text: t("transport.none"), known: false, hint: t("transport.none.hint") };
  }

  /**
   * The tooltip for a remembered identification.
   *
   * The date carries the weight here: the value is kept across restarts, so
   * "what it was" is only useful next to "when". It goes in the tooltip rather
   * than the cell because the cell has one line and the identity key has first
   * claim on it.
   */
  function lastHint(device: DeviceView, key: string): string {
    return t(key, { at: device.lastIdentifiedAt ?? "—" });
  }

  /**
   * Why the board column says what it says.
   *
   * Worth distinguishing because the two answers cost different things: a
   * silicon read restarted the board, a descriptor read sent it nothing at all.
   */
  function targetHint(identity: Identity): string {
    return identity.idSource === "usb-serial" ? t("target.hint.usb_serial") : t("target.hint");
  }

  /** Whether a probe could answer for this device, so the cell offers one. */
  function identifiable(device: DeviceView): boolean {
    return device.probes.some((p) => p.available);
  }

  function connection(device: DeviceView): string {
    return [device.busId, device.comPort].filter(Boolean).join(" / ") || "—";
  }

  /**
   * Whether the power column is there at all.
   *
   * Only in tree order, and only when a hub on screen can actually switch: a
   * column of blank cells is worse than no column, and this one is wide enough
   * to be felt.
   */
  const showPower = $derived(
    rows !== null && rows.some((row) => "hub" in row && row.hub?.ppps === true),
  );

  function openMenu(event: MouseEvent, instanceId: string) {
    event.preventDefault();
    onselect(instanceId);
    onmenu(instanceId, event.clientX, event.clientY);
  }
</script>

{#if rows ? rows.length === 0 : devices.length === 0}
  <p class="empty">{empty}</p>
{:else}
  <table>
    <thead>
      <tr>
        <th class="state">{t("col.state")}</th>
        <th class="connection">{t("col.connection")}</th>
        <th class="device">{t("col.device")}</th>
        <th class="vidpid">{t("col.vidpid")}</th>
        <th class="transport">{t("col.transport")}</th>
        <th class="target">{t("col.target")}</th>
        <th class="auto">{t("col.auto")}</th>
        {#if showPower}
          <th class="power-col">{t("col.power")}</th>
        {/if}
      </tr>
    </thead>
    <tbody>
      {#snippet deviceRow(
        device: DeviceView,
        depth: number,
        portLabel: string | null,
        hub: HubView | null = null,
        port: PortView | null = null,
      )}
        {@const off = port?.switched === "off"}
        {@const tr = transport(device)}
        {@const identity = device.identity}
        <tr
          class:off
          class:selected={device.instanceId === selected}
          onclick={() => onselect(device.instanceId)}
          oncontextmenu={(e) => openMenu(e, device.instanceId)}
        >
          <td class="state">
            {#if attaching.has(device.instanceId)}
              <span class="id pending">{t("state.attaching")}</span>
            {:else if device.state !== "not shared"}
              <!-- "not shared" is the state almost every device is in, so it is
                   left blank: a column that repeats the same word down every row
                   carries no information. -->
              <span class="dot {device.state}"></span>
              {t(`state.${device.state}`)}
            {/if}
          </td>
          <td class="connection">{connection(device)}</td>
          <td class="device" title={device.instanceId} style="--depth: {depth}">
            {#if portLabel !== null}
              <span class="port">{portLabel}</span>
            {/if}
            <!-- The name Windows reports, unchanged. Identifying a board does
                 not rename the device it is plugged into. -->
            <span class="primary">{device.name}</span>
            <!-- Sharing a device to WSL re-enumerates it as the stub, and
                 Windows stops reporting what it was. The name from before is
                 kept beside it so the row is still recognisable. -->
            {#if device.lastName}
              <span class="was" title={t("name.last.hint")}>{device.lastName}</span>
            {/if}
            {#if identity}
              <span class="secondary">{identity.deviceType}</span>
            {/if}
          </td>
          <td class="vidpid">{device.vidPid ?? "—"}</td>
          <td class="transport">
            <span class="id" class:known={tr.known} title={tr.hint}>{tr.text}</span>
          </td>
          <td class="target">
            {#if identity}
              <span class="id known" title={targetHint(identity)}>{identity.identityKey}</span>
            {:else if probing.has(device.instanceId)}
              <span class="id pending">{t("target.identifying")}</span>
            {:else if device.lastIdentity && identifiable(device)}
              <!-- The last answer, greyed, doubling as the button that
                   confirms it: what the user wants to know is whether this is
                   still that board, and one click settles it. -->
              <button
                class="identify last"
                title={lastHint(device, "target.last.identify.hint")}
                onclick={(e) => {
                  e.stopPropagation();
                  onselect(device.instanceId);
                  onidentify(device.instanceId);
                }}
              >
                {device.lastIdentity.identityKey}
              </button>
            {:else if device.lastIdentity}
              <span class="id last" title={lastHint(device, "target.last.hint")}
                >{device.lastIdentity.identityKey}</span
              >
            {:else if identifiable(device)}
              <!-- The action sits where the answer will appear, so identifying
                   is one click from the question rather than a trip through a
                   detail pane. -->
              <button
                class="identify"
                title={t("target.unidentified.hint")}
                onclick={(e) => {
                  e.stopPropagation();
                  onselect(device.instanceId);
                  onidentify(device.instanceId);
                }}
              >
                {t("target.unidentified")}
              </button>
            {:else}
              <span class="id" title={t("target.unavailable.hint")}>—</span>
            {/if}
          </td>
          <!-- Which rows a rule will take away, before it takes them. Its own
               column rather than a mark beside the state, which it outgrew as
               soon as the state read "WSL 接続中". -->
          <td class="auto">
            {#if device.autoAttach.matched}
              {@const kind = t(`auto_attach.kind.${device.autoAttach.matched}`)}
              <span
                class="tag"
                class:idle={!autoAttachOn}
                title={autoAttachOn
                  ? t("auto_attach.marked", { kind })
                  : t("auto_attach.marked.off", { kind })}>&check;</span
              >
            {/if}
          </td>
          {#if showPower}
            <!-- The row that matters most: a board that has been identified is
                 exactly the one worth power-cycling, and hiding the control
                 behind an empty-port row put it everywhere except there. -->
            <td class="power-col">
              {#if hub?.ppps && port}
                {@render powerButtons(hub, port, off)}
              {/if}
            </td>
          {/if}
        </tr>
      {/snippet}

      {#snippet powerButtons(hub: HubView, port: PortView, off: boolean)}
        {#if off}
          <span class="off-mark" title={t("tree.power.off.hint")}>{t("tree.power.off")}</span>
        {/if}
        <!-- Two actions rather than a toggle: nothing reports the power state,
             so a toggle would sometimes need pressing twice (R10.34). -->
        <button
          class="power"
          title={t("tree.power.on.hint")}
          onclick={(e) => {
            e.stopPropagation();
            onswitchport(hub.instanceId, port.port, true);
          }}>{t("tree.power.on")}</button
        >
        <button
          class="power"
          title={t("tree.power.cut.hint")}
          onclick={(e) => {
            e.stopPropagation();
            onswitchport(hub.instanceId, port.port, false);
          }}>{t("tree.power.cut")}</button
        >
      {/snippet}

      <!-- A hub is where things hang rather than something to bind or attach,
           so its row carries the name, what it is, and the one control that
           belongs to it: switching every port at once. -->
      {#snippet hubRow(hub: HubView, depth: number, portLabel: number | null, device: DeviceView | null)}
        <tr class="hub" onclick={() => device && onselect(device.instanceId)}>
          <td class="state"></td>
          <td class="connection"></td>
          <td class="device" title={hub.instanceId} style="--depth: {depth}">
            {#if portLabel !== null}
              <span class="port">{portLabel}</span>
            {/if}
            <span class="hub-mark">▣</span>
            <span class="primary">{hub.name}</span>
          </td>
          <td class="vidpid">{device?.vidPid ?? ""}</td>
          <td class="transport"></td>
          <td class="target"></td>
          <td class="auto"></td>
          <td class="power-col">
            {#if hub.ppps}
              <!-- Two actions rather than a toggle. A toggle would have to know
                   which way the port is, and nothing on Windows reports that —
                   so with the state unknown one press would do nothing visible
                   and the user would have to go the long way round. -->
              <button
                class="power"
                title={t("tree.power.all_on.hint")}
                onclick={(e) => {
                  e.stopPropagation();
                  onswitchhub(hub.instanceId, true);
                }}>{t("tree.power.all_on")}</button
              >
              <button
                class="power"
                title={t("tree.power.all_off.hint")}
                onclick={(e) => {
                  e.stopPropagation();
                  onswitchhub(hub.instanceId, false);
                }}>{t("tree.power.all_off")}</button
              >
            {/if}
          </td>
        </tr>
      {/snippet}

      <!-- A port with nothing on it, or nothing this tab lets through. Drawn
           only for hubs whose power can be switched, so the control stays
           reachable whatever the tabs are set to. -->
      {#snippet emptyPort(hub: HubView, port: PortView, depth: number, hidden: DeviceView | null)}
        {@const off = port.switched === "off"}
        <tr class="port-row">
          <td class="state"></td>
          <td class="connection"></td>
          <td class="device" style="--depth: {depth}">
            <span class="port">{port.port}</span>
            <span class="vacant">
              {hidden || port.connected ? t("tree.filtered") : t("tree.empty")}
            </span>
          </td>
          <td class="vidpid">{hidden?.vidPid ?? ""}</td>
          <td class="transport"></td>
          <td class="target"></td>
          <td class="auto"></td>
          <td class="power-col">{@render powerButtons(hub, port, off)}</td>
        </tr>
      {/snippet}

      {#if rows}
        {#each rows as row, i (i)}
          {#if row.kind === "hub"}
            {@render hubRow(row.hub, row.depth, row.port, row.device)}
          {:else if row.kind === "device"}
            {@render deviceRow(
              row.device,
              row.depth,
              row.port ? String(row.port.port) : null,
              row.hub,
              row.port,
            )}
          {:else}
            {@render emptyPort(row.hub, row.port, row.depth, row.hidden)}
          {/if}
        {/each}
      {:else}
        {#each devices as device (device.instanceId)}
          {@render deviceRow(device, 0, null)}
        {/each}
      {/if}
    </tbody>
  </table>
{/if}

<style>
  table {
    width: 100%;
    border-collapse: collapse;
    font-size: 13px;
    table-layout: fixed;
  }

  th {
    text-align: left;
    font-weight: 600;
    color: var(--fg-muted);
    padding: 6px 10px;
    border-bottom: 1px solid var(--border);
    position: sticky;
    top: 0;
    background: var(--bg-header);
  }

  td {
    padding: 5px 10px;
    border-bottom: 1px solid var(--border-subtle);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  tbody tr {
    cursor: default;
  }

  tbody tr:hover {
    background: var(--bg-hover);
  }

  tbody tr.selected {
    background: var(--bg-selected);
  }

  .state {
    width: 106px;
  }
  .connection {
    width: 128px;
    font-variant-numeric: tabular-nums;
  }
  .vidpid {
    /* Wide enough for `1a86:7523` in the mono face; a truncated VID/PID is
       worse than no column at all. */
    width: 100px;
    font-family: var(--mono);
    color: var(--fg-muted);
  }
  .transport {
    width: 176px;
  }
  .target {
    /* Room for the longest identifier a probe produces, e.g.
       `ch32x035c8t6-1ff9abcd880ebc48`. */
    width: 224px;
  }
  .auto {
    width: 52px;
    text-align: center;
  }

  .primary {
    font-weight: 600;
  }

  /* The indent lives on the cell's padding rather than on a spacer element, so
     the text still truncates against the column edge. */
  .device {
    padding-left: calc(10px + var(--depth, 0) * 16px);
  }

  .port {
    display: inline-block;
    min-width: 18px;
    margin-right: 6px;
    text-align: right;
    font-variant-numeric: tabular-nums;
    font-size: 12px;
    color: var(--fg-faint);
  }

  .hub-mark {
    margin-right: 4px;
    color: var(--fg-muted);
  }

  tr.hub {
    background: var(--bg-header);
  }

  .vacant {
    color: var(--fg-faint);
  }

  /* Switched off by us: the device node is still there and Windows still says
     it is fine, which is exactly why the row has to say otherwise. */
  tr.off .primary,
  tr.off .secondary,
  tr.off .id {
    color: var(--fg-faint);
    text-decoration: line-through;
  }

  .off-mark {
    font-size: 11px;
    color: var(--warn);
    margin-right: 6px;
  }

  button.power {
    font-size: 11px;
    padding: 2px 8px;
  }

  .power-col {
    width: 118px;
    white-space: nowrap;
  }

  .secondary {
    color: var(--fg-muted);
    margin-left: 8px;
  }

  .id {
    font-size: 12px;
    color: var(--fg-faint);
  }

  /* A value the device or board actually reported, as opposed to a placeholder. */
  .id.known {
    font-family: var(--mono);
    color: var(--fg);
  }

  .target .id.known {
    font-weight: 600;
    color: var(--ok);
  }

  .id.pending {
    color: var(--accent);
  }

  /* Reference, not an answer: the same value the live one would carry, drawn
     so it cannot be mistaken for one. */
  .was,
  .id.last,
  .identify.last {
    color: var(--fg-faint);
    font-style: italic;
  }

  .was {
    font-size: 12px;
    margin-left: 8px;
  }

  .id.last,
  .identify.last {
    font-family: var(--mono);
  }

  .identify {
    all: unset;
    font-size: 12px;
    color: var(--fg-muted);
    border-bottom: 1px dashed var(--fg-faint);
    cursor: default;
  }

  .identify:hover {
    color: var(--accent);
    border-bottom-color: var(--accent);
  }

  .tag {
    color: var(--accent);
    font-weight: 600;
  }

  /* The rule matches, but the switch is off, so nothing will come of it. */
  .tag.idle {
    color: var(--fg-faint);
    font-weight: 400;
  }

  .dot {
    display: inline-block;
    width: 8px;
    height: 8px;
    border-radius: 50%;
    margin-right: 6px;
    vertical-align: 1px;
    background: var(--fg-faint);
  }
  .dot.shared {
    background: var(--accent);
  }
  .dot.attached {
    background: var(--ok);
  }
  .dot.absent {
    background: transparent;
    border: 1px solid var(--fg-faint);
  }

  .empty {
    color: var(--fg-muted);
    font-size: 13px;
    padding: 14px 10px;
    margin: 0;
  }
</style>
