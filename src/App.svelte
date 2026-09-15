<script lang="ts">
  import BusyOverlay from "./lib/BusyOverlay.svelte";
  import ContextMenu from "./lib/ContextMenu.svelte";
  import DeviceTable from "./lib/DeviceTable.svelte";
  import ProbeDialog from "./lib/ProbeDialog.svelte";
  import SettingsPanel from "./lib/SettingsPanel.svelte";
  import {
    listDevices,
    log,
    probeDevice,
    readSettings,
    runOperation,
    writeSettings,
  } from "./lib/api";
  import { t } from "./lib/i18n";
  import type { DeviceView, Operation, ProbeOption } from "./lib/types";

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

  let probeTarget = $state<{ device: DeviceView; probe: ProbeOption } | null>(null);
  let menu = $state<{ device: DeviceView; x: number; y: number } | null>(null);
  /**
   * The operation the user asked for and is now waiting on.
   *
   * Set only for operations the user started. While it is set the window is
   * covered by a modal overlay: an attach takes seconds, and every other button
   * on screen would act on a state that is in the middle of changing.
   *
   * An automatic probe deliberately does not set this. The user did not ask for
   * it, so taking the window away from them would be worse than the small risk
   * of a click landing during the second it takes; the row shows its own
   * progress instead.
   */
  let busy = $state<string | null>(null);
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
  /** Ask before probing, stating what it does to the board (R4.7). */
  let confirmBeforeIdentify = $state(true);
  let startWithWindows = $state(false);
  /** False when the stored file was refused, so nothing is being saved. */
  let settingsWritable = $state(true);
  /** Set once the saved settings have arrived, so they are not saved back over. */
  let settingsLoaded = $state(false);

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

  /** Records a failure and shows it. Everything that fails goes through here. */
  function fail(what: string, e: unknown) {
    error = String(e);
    log("error", `${what}: ${error}`);
  }

  async function refresh() {
    try {
      const next = await listDevices();
      // Publish first: the queue looks arrivals up in `devices`, and an arrival
      // is by definition absent from the previous list.
      devices = next;
      noteArrivals(next);
      error = null;
    } catch (e) {
      fail("listing devices", e);
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
    // Already known, whether from this session or from the stored file.
    if (device.identity) return false;
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
        await identify(item.instanceId, probe.family, false);
      }
    } finally {
      draining = false;
    }
  }

  /** Runs one probe and records the result. Shared by manual and automatic paths. */
  /**
   * Runs one probe and records the result.
   *
   * `announce` covers the window while it runs. True when the user pressed a
   * button, false when the arrival queue started it by itself.
   */
  async function identify(instanceId: string, family: string, announce: boolean) {
    probingIds = new Set(probingIds).add(instanceId);
    if (announce) busy = t("busy.probe");
    try {
      await probeDevice(instanceId, family);
      error = null;
    } catch (e) {
      fail(`identifying ${instanceId}`, e);
    } finally {
      const next = new Set(probingIds);
      next.delete(instanceId);
      probingIds = next;
      if (announce) busy = null;
      await refresh();
    }
  }

  $effect(() => {
    readSettings()
      .then((stored) => {
        autoIdentify = stored.settings.autoIdentify;
        autoExcludeList = stored.settings.autoExclude.join(", ");
        confirmBeforeIdentify = stored.settings.confirmBeforeIdentify;
        startWithWindows = stored.settings.startWithWindows;
        settingsWritable = stored.writable;
        settingsLoaded = true;
        log("info", `settings from ${stored.path}`);
      })
      .catch((e) => fail("reading the settings", e));

    refresh();
    // Polling stands in for the device-change notifications of requirement
    // R8.1. It is safe because listing never probes (R4.6).
    //
    // Suspended while an operation runs: usbipd is busy with that, and a reply
    // that arrives mid-attach describes a state that is already gone.
    const timer = setInterval(() => {
      if (!busy) refresh();
    }, 2000);
    return () => clearInterval(timer);
  });

  /** The probe that could run against a device, or why none can. */
  function probeFor(device: DeviceView): { probe: ProbeOption } | { reason: string } {
    const probe = device.probes.find((p) => p.available);
    if (probe) return { probe };
    const blocked = device.probes.find((p) => p.reason);
    return { reason: t(blocked?.reason ?? "probe.blocked.none") };
  }

  function saveSettings() {
    return writeSettings({
      autoIdentify,
      autoExclude: [...excluded()],
      confirmBeforeIdentify,
      startWithWindows,
    }).catch((e) => {
      fail("saving the settings", e);
      // The registry refused, so the switch did not take. Put it back rather
      // than leaving the panel claiming something that is not true.
      void readSettings()
        .then((stored) => (startWithWindows = stored.settings.startWithWindows))
        .catch(() => {});
    });
  }

  function startProbe(device: DeviceView) {
    const outcome = probeFor(device);
    if (!("probe" in outcome)) return;
    // The warning is the whole point of the dialog; with it turned off there is
    // nothing left for the dialog to do.
    if (!confirmBeforeIdentify) {
      void identify(device.instanceId, outcome.probe.family, true);
      return;
    }
    probeTarget = { device, probe: outcome.probe };
  }

  /** Every connected device that could be identified and has not been. */
  const unidentified = $derived(
    devices.filter(
      (d) => d.present && !d.identity && d.probes.some((p) => p.available),
    ),
  );

  let confirmingAll = $state(false);

  function startIdentifyAll() {
    if (unidentified.length === 0) return;
    if (!confirmBeforeIdentify) {
      void identifyAll();
      return;
    }
    confirmingAll = true;
  }

  /**
   * Identifies everything outstanding, one at a time.
   *
   * Sequential because two probes would fight over adapters, and because the
   * list is re-read between them: a board that answered may have changed what
   * is outstanding.
   */
  async function identifyAll() {
    confirmingAll = false;
    const queue = unidentified.map((d) => d.instanceId);
    const total = queue.length;

    for (const [index, instanceId] of queue.entries()) {
      busy = t("busy.identify_all", { done: index + 1, total });
      const device = devices.find((d) => d.instanceId === instanceId);
      const probe = device?.probes.find((p) => p.available);
      if (!probe) continue;
      await identify(instanceId, probe.family, false);
    }
    busy = null;
  }

  async function runProbe() {
    if (!probeTarget) return;
    const { device, probe } = probeTarget;
    // Close first: the overlay below reports progress, so keeping the dialog up
    // with its own spinner would stack two answers to the same question.
    probeTarget = null;
    await identify(device.instanceId, probe.family, true);
  }

  /**
   * Runs a usbipd operation.
   *
   * Only one at a time: bind and unbind raise a UAC prompt, and a second prompt
   * stacking behind the first is not something the user can make sense of.
   */
  async function operate(device: DeviceView, operation: Operation) {
    if (busy) return;
    // Bind and unbind stop at a UAC prompt first, which is what the user is
    // actually waiting on.
    busy = t(
      operation === "bind" || operation === "unbind" ? "busy.admin" : `busy.${operation}`,
    );
    try {
      devices = await runOperation(device.instanceId, operation);
      error = null;
    } catch (e) {
      fail(`${operation} on ${device.instanceId}`, e);
    } finally {
      busy = null;
    }
  }

  /** The operations to offer for a device, in the order they are usually used. */
  function operations(device: DeviceView): { id: Operation; label: string }[] {
    const ADMIN: Operation[] = ["bind", "unbind"];
    return (["bind", "attach", "detach", "unbind"] as Operation[])
      .filter((id) => device.actions[id])
      .map((id) => ({
        label: t(`menu.${id}`) + (ADMIN.includes(id) ? t("menu.admin_suffix") : ""),
        id,
      }));
  }

  async function copy(text: string) {
    try {
      await navigator.clipboard.writeText(text);
      toast = t("menu.copied");
      setTimeout(() => (toast = null), 1400);
    } catch (e) {
      fail("copying to the clipboard", e);
    }
  }

  const menuItems = $derived.by(() => {
    if (!menu) return [];
    const device = menu.device;
    const outcome = probeFor(device);
    const identity = device.identity;
    return [
      ...operations(device).map((op) => ({
        label: op.label,
        disabledReason: busy,
        action: () => operate(device, op.id),
      })),
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
        action: () => copy(identity?.identityKey ?? device.serial ?? ""),
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
      <button
        disabled={unidentified.length === 0 || busy !== null}
        title={unidentified.length === 0
          ? t("toolbar.identify_all.none")
          : t("toolbar.identify_all.hint")}
        onclick={startIdentifyAll}
      >
        {t("toolbar.identify_all")}
        {#if unidentified.length > 0}
          <span class="count">{unidentified.length}</span>
        {/if}
      </button>
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

  {#if error}
    <p class="error">{error}</p>
  {/if}

  <!-- Fixed height: selecting a device must not resize the list above it. -->
  <footer>
    {#if selected}
      {@const identity = selected.identity}
      {@const outcome = probeFor(selected)}
      <div class="detail">
        <div class="detail-head">
          <strong>{selected.name}</strong>
          {#if identity}
            <span class="type">{identity.deviceType}</span>
          {/if}
          <code>{selected.instanceId}</code>
        </div>

        <!--
          Two columns of pairs rather than one: the USB identifiers had to fit
          without the pane growing downwards, and the list above it is worth
          more rows than this is.
        -->
        <dl>
          <dt>{t("detail.port")}</dt>
          <dd>{selected.busId ?? "—"} / {selected.comPort ?? "—"}</dd>
          <dt>{t("detail.vidpid")}</dt>
          <dd>
            <code>{selected.vidPid ?? "—"}</code>
            {#if selected.revision}
              <span class="note">rev {selected.revision}</span>
            {/if}
          </dd>

          <dt>{t("detail.location")}</dt>
          <dd class="wide" title={t("detail.location.hint")}>
            <code>{selected.portChain ?? "—"}</code>
            <span class="note">{selected.locationPath ?? ""}</span>
          </dd>

          <dt>{t("detail.driver")}</dt>
          <dd>
            {selected.driver ?? "—"}
            {#if selected.driverVersion}
              <span class="note">{selected.driverVersion}</span>
            {/if}
          </dd>
          <dt>{t("detail.vendor")}</dt>
          <dd title={selected.vendor ? t("detail.from_usb_ids") : undefined}>
            {selected.vendor ?? "—"}
          </dd>

          <dt>{t("detail.transport")}</dt>
          <dd>
            {#if selected.serial}
              <code>{selected.serial}</code>
            {:else}
              <span class="note" title={t("transport.none.hint")}>{t("transport.none")}</span>
            {/if}
          </dd>
          <dt>{t("detail.usb_product")}</dt>
          <dd title={selected.usbProduct ? t("detail.from_usb_ids") : undefined}>
            {selected.usbProduct ?? "—"}
          </dd>

          <dt>{t("detail.target")}</dt>
          <dd class="wide">
            {#if selected.problemCode !== null}
              <span class="problem"
                >{t("detail.problem", { code: selected.problemCode })}</span
              >
            {/if}
            {#if identity}
              <code class="key">{identity.identityKey}</code>
              <span class="note">{identity.deviceType} / {identity.deviceId}</span>
            {:else if selected.needsProbe}
              <span class="note">{t("detail.target.unknown")}</span>
            {:else}
              <span class="note">{t("detail.target.transport_only")}</span>
            {/if}
          </dd>
        </dl>
      </div>

      <div class="detail-actions">
        {#each operations(selected) as op (op.id)}
          <button disabled={busy !== null} onclick={() => operate(selected, op.id)}>
            {op.label}
          </button>
        {/each}
        {#if !("reason" in outcome)}
          <button class="primary" onclick={() => startProbe(selected)}>
            {t("menu.identify")}
          </button>
        {/if}
      </div>
    {:else}
      <p class="note">{t("detail.select")}</p>
    {/if}
  </footer>
</main>

{#if toast}
  <div class="toast">{toast}</div>
{/if}

{#if menu}
  <ContextMenu x={menu.x} y={menu.y} items={menuItems} onclose={() => (menu = null)} />
{/if}

{#if confirmingAll}
  <!-- Requirement R4.7 applied to the whole set: how many boards restart is
       the part the user needs before agreeing, not just that some will. -->
  <div class="backdrop" role="presentation" onclick={() => (confirmingAll = false)}></div>
  <div class="dialog" role="dialog" aria-modal="true" aria-labelledby="all-title">
    <h2 id="all-title">{t("identify_all.title")}</h2>
    <p class="effect">
      {t("identify_all.count", { count: unidentified.length })}<br />
      <strong>{t("identify_all.warning")}</strong>
    </p>
    <div class="dialog-actions">
      <button onclick={() => (confirmingAll = false)}>{t("probe.cancel")}</button>
      <button class="primary" onclick={identifyAll}>{t("identify_all.run")}</button>
    </div>
  </div>
{/if}

{#if settingsOpen}
  <SettingsPanel
    enabled={autoIdentify}
    excludeList={autoExcludeList}
    {confirmBeforeIdentify}
    {startWithWindows}
    graceSeconds={GRACE_SECONDS}
    writable={settingsWritable}
    onchange={(next) => {
      autoIdentify = next.enabled;
      autoExcludeList = next.excludeList;
      confirmBeforeIdentify = next.confirmBeforeIdentify;
      startWithWindows = next.startWithWindows;
      if (!settingsLoaded) return;
      void saveSettings();
    }}
    onclose={() => (settingsOpen = false)}
  />
{/if}

{#if probeTarget}
  <ProbeDialog
    device={probeTarget.device}
    probe={probeTarget.probe}
    onconfirm={runProbe}
    oncancel={() => (probeTarget = null)}
  />
{/if}

<!-- Last, so it sits over everything else including the dialogs. -->
{#if busy}
  <BusyOverlay what={busy} />
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

  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.45);
  }

  .dialog {
    position: fixed;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    width: min(440px, calc(100vw - 32px));
    padding: 20px;
    background: var(--bg-header);
    border: 1px solid var(--border);
    border-radius: 8px;
    box-shadow: 0 18px 48px rgba(0, 0, 0, 0.45);
  }

  .dialog h2 {
    margin: 0 0 14px;
    font-size: 15px;
  }

  .dialog .effect {
    margin: 0 0 18px;
    padding: 10px 12px;
    font-size: 13px;
    line-height: 1.6;
    color: var(--fg);
    background: color-mix(in srgb, var(--warn) 12%, transparent);
    border: 1px solid color-mix(in srgb, var(--warn) 40%, transparent);
    border-radius: 6px;
  }

  .dialog-actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
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
    /* Fixed, not `auto`: the number of buttons changes with the device, and
       an auto column would resize the detail area — moving the VID:PID and
       vendor columns sideways every time the selection changed. */
    grid-template-columns: minmax(0, 1fr) 344px;
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
    grid-template-columns: 76px minmax(0, 1fr) 82px minmax(0, 1fr);
    gap: 3px 12px;
    margin: 0;
    font-size: 12px;
  }

  /* Values that are long enough to deserve the whole row. */
  dd.wide {
    grid-column: 2 / -1;
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

  .problem {
    color: var(--danger);
    margin-right: 8px;
  }

  .detail-actions {
    display: flex;
    flex-wrap: wrap;
    justify-content: flex-end;
    gap: 6px;
  }

  /* Above the footer, not after it: the footer has a fixed height, so an
     error appended below it was pushed out of the window. */
  .error {
    flex: 0 0 auto;
    margin: 0;
    padding: 8px 14px;
    font-size: 12px;
    line-height: 1.5;
    color: var(--danger);
    background: color-mix(in srgb, var(--danger) 10%, transparent);
    border-top: 1px solid color-mix(in srgb, var(--danger) 30%, transparent);
    user-select: text;
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
