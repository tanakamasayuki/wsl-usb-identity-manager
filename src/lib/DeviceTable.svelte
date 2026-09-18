<script lang="ts">
  import { t } from "./i18n";
  import type { DeviceView, Identity } from "./types";

  interface Props {
    devices: DeviceView[];
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
  }

  let {
    devices,
    selected,
    empty,
    probing,
    attaching,
    autoAttachOn,
    onselect,
    onmenu,
    onidentify,
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

  function openMenu(event: MouseEvent, instanceId: string) {
    event.preventDefault();
    onselect(instanceId);
    onmenu(instanceId, event.clientX, event.clientY);
  }
</script>

{#if devices.length === 0}
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
      </tr>
    </thead>
    <tbody>
      {#each devices as device (device.instanceId)}
        {@const tr = transport(device)}
        {@const identity = device.identity}
        <tr
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
          <td class="device" title={device.instanceId}>
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
        </tr>
      {/each}
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
