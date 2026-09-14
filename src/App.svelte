<script lang="ts">
  import ContextMenu from "./lib/ContextMenu.svelte";
  import DeviceTable from "./lib/DeviceTable.svelte";
  import ProbeDialog from "./lib/ProbeDialog.svelte";
  import SettingsPanel from "./lib/SettingsPanel.svelte";
  import { listDevices, probeDevice } from "./lib/api";
  import { t } from "./lib/i18n";
  import type { DeviceView, ProbeOption, TargetIdentity } from "./lib/types";

  /**
   * One list with tabs, rather than a table per state.
   *
   * A device attached to WSL disappears from the Windows device tree, so
   * splitting the view by "does Windows see it" would make rows jump between
   * tables exactly when the user is watching them. With one list, a row stays
   * where it is and only its state changes.
   */
  type Filter = "connected" | "shared" | "attached" | "absent" | "all";

  const FILTERS: { id: Filter; match: (d: DeviceView) => boolean }[] = [
    { id: "connected", match: (d) => d.present },
    { id: "shared", match: (d) => d.shared && d.present },
    { id: "attached", match: (d) => d.attached },
    { id: "absent", match: (d) => !d.present },
    { id: "all", match: () => true },
  ];

  let devices = $state<DeviceView[]>([]);
  let error = $state<string | null>(null);
  let selectedId = $state<string | null>(null);
  let filter = $state<Filter>("connected");
  let toast = $state<string | null>(null);

  /**
   * Probe results, keyed by instance id.
   *
   * In memory only for now: the persistence layer of requirements §7 is not
   * built yet, so these are lost on restart.
   */
  let identities = $state<Record<string, TargetIdentity>>({});
  let probeTarget = $state<{ device: DeviceView; probe: ProbeOption } | null>(null);
  let probing = $state(false);
  let menu = $state<{ device: DeviceView; x: number; y: number } | null>(null);
  let probingIds = $state(new Set<string>());
  let settingsOpen = $state(false);

  /**
   * Automatic identification right after a device is plugged in (R4.8).
   *
   * On by default, and gated by an exclusion list rather than the allow list
   * requirement R4.9 describes. Listing carriers up front does not work: the
   * VID:PID of a CH340 says nothing about whether a dev board or a router
   * console is on the other end of it, so an allow list ends up naming every
   * common bridge anyway. The exclusion list lets the user rule out the
   * specific hardware that must not be disturbed.
   *
   * Still deliberately narrow: a probe restarts the board, so it only runs in
   * the moment a device arrives, before anything has opened it. It never runs
   * at startup — the devices already plugged in have been running for a while
   * and something may well be talking to them (R4.6).
   */
  let autoIdentify = $state(true);
  /** VID:PID never probed automatically. Empty means nothing is excluded. */
  let autoExcludeList = $state("");

  const GRACE_SECONDS = 10;

  /** Present devices from the previous poll, to spot arrivals. */
  let seen = new Set<string>();
  /** The first poll seeds `seen` without probing anything. */
  let seeded = false;
  /** Arrivals waiting for a probe, with the moment they appeared. */
  let pending: { instanceId: string; arrivedAt: number }[] = [];
  let draining = false;

  const selected = $derived(devices.find((d) => d.instanceId === selectedId) ?? null);
  const counts = $derived(
    Object.fromEntries(FILTERS.map((f) => [f.id, devices.filter(f.match).length])) as Record<
      Filter,
      number
    >,
  );
  const shown = $derived(devices.filter(FILTERS.find((f) => f.id === filter)!.match));

  async function refresh() {
    try {
      const next = await listDevices();
      // Publish first: the queue looks arrivals up in `devices`, and an arrival
      // is by definition absent from the previous list.
      devices = next;
      noteArrivals(next);
      error = null;
    } catch (e) {
      error = String(e);
    }
  }

  /**
   * Diffs reachable devices against the previous poll and queues arrivals.
   *
   * Reachability, not `present`: usbipd keeps reporting a bus id throughout an
   * attach, so a device handed to WSL and taken back never looks absent by that
   * measure. What changes is whether Windows has a device node — which is also
   * exactly what decides whether a probe could run at all.
   */
  function noteArrivals(next: DeviceView[]) {
    const reachable = next.filter((d) => d.reachable);
    const arrivals = seeded
      ? reachable.filter((d) => !seen.has(d.instanceId))
      : [];
    seen = new Set(reachable.map((d) => d.instanceId));
    seeded = true;

    if (autoIdentify) {
      const now = Date.now();
      for (const device of arrivals) {
        // Only the filters that cannot change while the device stays plugged
        // in are applied here. Whether a probe can run yet is decided later:
        // see drain().
        if (device.serial) continue;
        if (device.vidPid && excluded().has(device.vidPid)) continue;
        pending.push({ instanceId: device.instanceId, arrivedAt: now });
      }
    }
    // Always drain, not only when something arrived: an entry queued a moment
    // ago may only now have become probeable.
    void drain();
  }

  /** The exclusion list, normalised to lowercase `vvvv:pppp`. */
  function excluded(): Set<string> {
    return new Set(
      autoExcludeList
        .split(/[\s,]+/)
        .map((s) => s.trim().toLowerCase())
        .filter(Boolean),
    );
  }

  function autoEligible(device: DeviceView): boolean {
    // A device that reports a serial needs no probe (R4.8).
    if (device.serial) return false;
    if (identities[device.instanceId]) return false;
    if (device.vidPid && excluded().has(device.vidPid)) return false;
    return device.probes.some((p) => p.available);
  }

  /**
   * Runs queued probes one at a time.
   *
   * Serial, because two probes would fight for the same adapter.
   *
   * An entry is kept and retried rather than dropped when it is not ready: a
   * device becomes visible to Windows a moment before its COM port is assigned,
   * and a probe needs the COM port. Checking once, at the instant of arrival,
   * made automatic identification a race it usually lost. The grace window is
   * what bounds the retrying — never the single chance it happened to get.
   */
  async function drain() {
    if (draining) return;
    draining = true;
    try {
      for (;;) {
        const cutoff = Date.now() - GRACE_SECONDS * 1000;
        pending = pending.filter((p) => p.arrivedAt > cutoff);

        const ready = pending.findIndex((p) => {
          const device = devices.find((d) => d.instanceId === p.instanceId);
          return !!device && autoIdentify && autoEligible(device);
        });
        // Nothing is ready right now; the next poll looks again.
        if (ready < 0) break;

        const [item] = pending.splice(ready, 1);
        const device = devices.find((d) => d.instanceId === item.instanceId)!;
        const probe = device.probes.find((p) => p.available)!;
        await identify(item.instanceId, probe.family);
      }
    } finally {
      draining = false;
    }
  }

  /** Runs one probe and records the result. Shared by manual and automatic paths. */
  async function identify(instanceId: string, family: string) {
    probingIds = new Set(probingIds).add(instanceId);
    try {
      identities[instanceId] = await probeDevice(instanceId, family);
      error = null;
    } catch (e) {
      error = String(e);
    } finally {
      const next = new Set(probingIds);
      next.delete(instanceId);
      probingIds = next;
      await refresh();
    }
  }

  $effect(() => {
    refresh();
    // Polling stands in for the device-change notifications of requirement
    // R8.1. It is safe because listing never probes (R4.6).
    const timer = setInterval(refresh, 2000);
    return () => clearInterval(timer);
  });

  /** The probe that could run against a device, or why none can. */
  function probeFor(device: DeviceView): { probe: ProbeOption } | { reason: string } {
    const probe = device.probes.find((p) => p.available);
    if (probe) return { probe };
    const blocked = device.probes.find((p) => p.reason);
    return { reason: t(blocked?.reason ?? "probe.blocked.none") };
  }

  function startProbe(device: DeviceView) {
    const outcome = probeFor(device);
    if ("probe" in outcome) probeTarget = { device, probe: outcome.probe };
  }

  async function runProbe() {
    if (!probeTarget) return;
    const { device, probe } = probeTarget;
    probing = true;
    try {
      await identify(device.instanceId, probe.family);
    } finally {
      probing = false;
      probeTarget = null;
    }
  }

  async function copy(text: string) {
    try {
      await navigator.clipboard.writeText(text);
      toast = t("menu.copied");
      setTimeout(() => (toast = null), 1400);
    } catch (e) {
      error = String(e);
    }
  }

  const menuItems = $derived.by(() => {
    if (!menu) return [];
    const device = menu.device;
    const outcome = probeFor(device);
    const identity = identities[device.instanceId];
    return [
      {
        label: t("menu.identify"),
        disabledReason: "reason" in outcome ? outcome.reason : null,
        action: () => startProbe(device),
      },
      {
        label: t("menu.copy_instance_id"),
        action: () => copy(device.instanceId),
      },
      {
        label: t("menu.copy_identifier"),
        disabledReason: identity || device.serial ? null : t("menu.nothing_to_copy"),
        action: () => copy(identity?.identity_key ?? device.serial ?? ""),
      },
    ];
  });
</script>

<main>
  <nav>
    <div class="tabs">
      {#each FILTERS as f (f.id)}
        <button class="tab" class:active={filter === f.id} onclick={() => (filter = f.id)}>
          {t(`filter.${f.id}`)}
          <span class="count">{counts[f.id]}</span>
        </button>
      {/each}
    </div>
    <div class="actions">
      {#if autoIdentify}
        <!-- A setting that restarts boards on its own should be visible without
             opening a dialog to check — and so should the case where it is
             switched on but cannot match anything. -->
        <span class="armed" title={t("toolbar.auto_on.hint")}>{t("toolbar.auto_on")}</span>
      {/if}
      <button onclick={() => (settingsOpen = true)}>
        {t("toolbar.settings")}
      </button>
      <button onclick={refresh}>{t("toolbar.refresh")}</button>
    </div>
  </nav>

  <div class="scroll">
    <DeviceTable
      devices={shown}
      selected={selectedId}
      {identities}
      probing={probingIds}
      empty={t(`empty.${filter}`)}
      onselect={(id) => (selectedId = id)}
      onidentify={(id) => {
        const device = devices.find((d) => d.instanceId === id);
        if (device) startProbe(device);
      }}
      onmenu={(id, x, y) => {
        const device = devices.find((d) => d.instanceId === id);
        if (device) menu = { device, x, y };
      }}
    />
  </div>

  <!-- Fixed height: selecting a device must not resize the list above it. -->
  <footer>
    {#if selected}
      {@const identity = identities[selected.instanceId]}
      {@const outcome = probeFor(selected)}
      <div class="detail">
        <div class="detail-head">
          <strong>{selected.name}</strong>
          {#if identity}
            <span class="type">{identity.device_type}</span>
          {/if}
          <code>{selected.instanceId}</code>
        </div>

        <dl>
          <dt>{t("detail.port")}</dt>
          <dd>{selected.busId ?? "—"} / {selected.comPort ?? "—"}</dd>
          <dt>{t("detail.location")}</dt>
          <dd><code>{selected.locationPath ?? "—"}</code></dd>
          <dt>{t("detail.driver")}</dt>
          <dd>{selected.driver ?? "—"}</dd>
          <dt>{t("detail.transport")}</dt>
          <dd>
            {#if selected.serial}
              <code>{selected.serial}</code>
            {:else}
              <span class="note" title={t("transport.none.hint")}>{t("transport.none")}</span>
            {/if}
          </dd>
          <dt>{t("detail.target")}</dt>
          <dd>
            {#if identity}
              <code class="key">{identity.identity_key}</code>
              <span class="note">{identity.device_type} / {identity.device_id}</span>
            {:else if selected.needsProbe}
              <span class="note">{t("detail.target.unknown")}</span>
            {:else}
              <span class="note">{t("detail.target.transport_only")}</span>
            {/if}
          </dd>
        </dl>
      </div>

      <div class="detail-actions">
        {#if "reason" in outcome}
          <span class="note wrap">{outcome.reason}</span>
        {:else}
          <button class="primary" onclick={() => startProbe(selected)}>
            {t("menu.identify")}
          </button>
        {/if}
      </div>
    {:else}
      <p class="note">{t("detail.select")}</p>
    {/if}
  </footer>

  {#if error}
    <p class="error">{error}</p>
  {/if}
</main>

{#if toast}
  <div class="toast">{toast}</div>
{/if}

{#if menu}
  <ContextMenu x={menu.x} y={menu.y} items={menuItems} onclose={() => (menu = null)} />
{/if}

{#if settingsOpen}
  <SettingsPanel
    enabled={autoIdentify}
    excludeList={autoExcludeList}
    graceSeconds={GRACE_SECONDS}
    onchange={(next) => {
      autoIdentify = next.enabled;
      autoExcludeList = next.excludeList;
    }}
    onclose={() => (settingsOpen = false)}
  />
{/if}

{#if probeTarget}
  <ProbeDialog
    device={probeTarget.device}
    probe={probeTarget.probe}
    running={probing}
    onconfirm={runProbe}
    oncancel={() => (probeTarget = null)}
  />
{/if}

<style>
  main {
    display: flex;
    flex-direction: column;
    height: 100vh;
    overflow: hidden;
  }

  nav {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 8px 14px;
    border-bottom: 1px solid var(--border);
    background: var(--bg-header);
  }

  nav .actions {
    display: flex;
    gap: 6px;
  }

  nav .armed {
    display: flex;
    align-items: center;
    padding: 0 10px;
    border-radius: 5px;
    font-size: 12px;
    color: var(--warn);
    border: 1px solid color-mix(in srgb, var(--warn) 45%, transparent);
    background: color-mix(in srgb, var(--warn) 12%, transparent);
  }

  .tabs {
    display: flex;
    gap: 2px;
    overflow-x: auto;
  }

  .tab {
    all: unset;
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 5px 12px;
    border-radius: 6px;
    font-size: 13px;
    color: var(--fg-muted);
    cursor: default;
    white-space: nowrap;
  }

  .tab:hover {
    background: var(--bg-hover);
  }

  .tab.active {
    background: var(--bg-selected);
    color: var(--fg);
    font-weight: 600;
  }

  .count {
    font-size: 11px;
    min-width: 16px;
    text-align: center;
    padding: 0 4px;
    border-radius: 8px;
    background: color-mix(in srgb, var(--fg-muted) 18%, transparent);
  }

  .scroll {
    flex: 1 1 auto;
    overflow-y: scroll;
    /* Reserve the scrollbar track at every row count, so the columns do not
       shift sideways as devices come and go. */
    scrollbar-gutter: stable;
  }

  footer {
    flex: 0 0 auto;
    height: 150px;
    /* The detail pane exists to be read off and pasted elsewhere — instance
       ids, location paths, identity keys. The list above stays unselectable so
       dragging across rows still selects rows. */
    user-select: text;
    cursor: text;
    padding: 12px 14px;
    border-top: 1px solid var(--border);
    background: var(--bg-header);
    display: grid;
    grid-template-columns: 1fr 190px;
    gap: 12px;
    align-items: start;
  }

  .detail {
    overflow: hidden;
  }

  .detail-head {
    display: flex;
    align-items: baseline;
    gap: 10px;
    margin-bottom: 8px;
    overflow: hidden;
  }

  .detail-head code {
    font-size: 11px;
    color: var(--fg-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  dl {
    display: grid;
    grid-template-columns: 76px 1fr;
    gap: 3px 12px;
    margin: 0;
    font-size: 12px;
  }

  dt {
    color: var(--fg-muted);
  }

  dd {
    margin: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .key {
    color: var(--ok);
    font-weight: 600;
  }

  .type {
    color: var(--fg-muted);
    font-size: 12px;
  }

  .note {
    color: var(--fg-muted);
    font-size: 12px;
  }

  .note.wrap {
    white-space: normal;
    line-height: 1.5;
    text-align: right;
  }

  .detail-actions {
    display: flex;
    justify-content: flex-end;
  }

  .error {
    flex: 0 0 auto;
    margin: 0;
    padding: 8px 14px;
    font-size: 12px;
    color: var(--danger);
    background: color-mix(in srgb, var(--danger) 10%, transparent);
    border-top: 1px solid color-mix(in srgb, var(--danger) 30%, transparent);
  }

  .toast {
    position: fixed;
    bottom: 20px;
    left: 50%;
    transform: translateX(-50%);
    padding: 7px 16px;
    border-radius: 16px;
    font-size: 12px;
    background: var(--bg-selected);
    border: 1px solid var(--border);
  }
</style>
