<script lang="ts">
  import AutoAttachPanel from "./lib/AutoAttachPanel.svelte";
  import BusyOverlay from "./lib/BusyOverlay.svelte";
  import ContextMenu from "./lib/ContextMenu.svelte";
  import DeviceTable from "./lib/DeviceTable.svelte";
  import ProbeDialog from "./lib/ProbeDialog.svelte";
  import SettingsPanel from "./lib/SettingsPanel.svelte";
  import {
    appVersion,
    checkUsbipd,
    clearRemembered,
    hideWindow,
    listDevices,
    log,
    onCloseRequested,
    onIdentifyAll,
    onOpenSettings,
    onSettingsChanged,
    openTarget,
    probeDevice,
    readSettings,
    runOperation,
    setTray,
    showWindow,
    writeSettings,
  } from "./lib/api";
  import { t } from "./lib/i18n";
  import type {
    Availability,
    AutoAttachRule,
    Candidate,
    DeviceView,
    Operation,
    ProbeOption,
  } from "./lib/types";

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

  /**
   * Automatic attach (§9).
   *
   * Off by default and switched from the toolbar rather than from a dialog:
   * handing devices to WSL by itself is the kind of thing to turn off in the
   * middle of working, when something needs to stay on the Windows side.
   */
  let autoAttach = $state(false);
  let autoAttachRules = $state<AutoAttachRule[]>([]);
  let autoAttachOpen = $state(false);
  /** Devices being attached by a rule rather than by the user. */
  let attachingIds = $state(new Set<string>());
  /**
   * Devices a rule has already acted on, so a failure does not retry every two
   * seconds — and a manual detach is not immediately undone by the same rule.
   *
   * Cleared for a device when it stops being present, i.e. when it is actually
   * unplugged. Attaching and detaching do not clear it: `present` stays true
   * across both, which is exactly the distinction wanted here.
   */
  let autoAttachTried = new Set<string>();
  let autoAttaching = false;

  /**
   * Whether usbipd is there at all.
   *
   * Without this, a machine with no usbipd-win shows an empty list and a red
   * error from every operation — which says what failed but not that the thing
   * it needs was never installed (R13.1).
   */
  let usbipd = $state<Availability | null>(null);
  /** Where the settings and the log are, for the settings panel (R13.8). */
  let settingsPath = $state("");
  let logPath = $state("");
  /** Which build this is. Shown next to the log path, which is what a report needs. */
  let version = $state("");
  /** How many devices have a remembered name or identification (R4.21). */
  let remembered = $state(0);
  /**
   * Whether closing the window has been explained once.
   *
   * Closing leaves the application in the tray, because automatic attach and
   * automatic identification only run while it is alive. "I closed it and it is
   * still running" is the thing about that which has to be said out loud —
   * once, not every time.
   */
  let toldAboutTray = $state(false);
  let closingNotice = $state(false);
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

  const counts = $derived(
    Object.fromEntries(FILTERS.map((f) => [f.id, devices.filter(f.match).length])) as Record<
      Filter,
      number
    >,
  );
  const shown = $derived(devices.filter(FILTERS.find((f) => f.id === filter)!.match));

  /** Every connected device that could be identified and has not been. */
  const unidentified = $derived(
    devices.filter((d) => d.present && !d.identity && d.probes.some((p) => p.available)),
  );

  /**
   * The selected device, but only while it is one of the rows on screen.
   *
   * Found in `shown` rather than in `devices`: the detail pane describes the
   * selected row, so with no row selected there is nothing for it to describe.
   * Searching the whole list instead left the pane showing a device that the
   * current tab does not contain — no row highlighted, and buttons acting on
   * something the user could not see.
   *
   * `selectedId` itself is kept, so going back to the tab the device is in
   * brings the selection back with it.
   */
  const selected = $derived(shown.find((d) => d.instanceId === selectedId) ?? null);

  /**
   * Records a failure and shows it.
   *
   * The banner stays until the user dismisses it or starts something new. It
   * used to be cleared by the next successful poll, which gave a usbipd error
   * two seconds on screen before it vanished — long enough to notice, not
   * nearly long enough to read.
   */
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
      // Cleared only on success: a usbipd that started answering again should
      // take its own notice down.
      if (usbipd?.status !== "ok") void recheckUsbipd();
    } catch (e) {
      fail("listing devices", e);
      // The listing failing is the moment to find out whether usbipd is even
      // installed, rather than reporting the same error over and over.
      void recheckUsbipd();
    }
  }

  /**
   * Keeps the tray menu in step with what the window shows.
   *
   * The labels are sent from here because the translations are here (R10.5);
   * the backend holds no catalogue of its own.
   */
  let lastTray = "";

  function refreshTray() {
    const counted = t("tray.status", {
      connected: counts.connected,
      shared: counts.shared,
      attached: counts.attached,
    });
    const view = {
      tooltip: `${t("app.name")}${version ? ` ${version}` : ""}\n${counted}`,
      // While something is running, the menu says so. Identify-all can be
      // started from here with the window closed, and several seconds of
      // boards restarting with nothing on screen would be opaque.
      status: busy ?? counted,
      autoAttachLabel: t("toolbar.auto_attach"),
      autoAttachOn: autoAttach,
      identifyAllLabel:
        unidentified.length > 0
          ? t("tray.identify_all", { count: unidentified.length })
          : t("toolbar.identify_all"),
      identifyAllEnabled: unidentified.length > 0 && busy === null,
      openLabel: t("tray.open"),
      settingsLabel: t("tray.settings"),
      quitLabel: t("tray.quit"),
    };
    const next = JSON.stringify(view);
    if (next === lastTray) return;
    lastTray = next;
    void setTray(view).catch((e) => log("error", `updating the tray: ${e}`));
  }

  // Counts change with every poll, and the switch can be flipped from either
  // side, so the menu follows both.
  $effect(() => {
    if (!settingsOpen) return;
    // Devices are remembered as they are seen and identified, all of which
    // happens with this panel closed. Reading the count once at startup would
    // show a number that was right when the application began.
    readSettings()
      .then((stored) => (remembered = stored.remembered))
      .catch((e) => fail("reading the settings", e));
  });

  $effect(() => {
    void counts;
    void autoAttach;
    void unidentified;
    void busy;
    refreshTray();
  });

  /**
   * Identify-all, chosen from the tray.
   *
   * Runs under the same confirmation setting as the button (R4.7) — and the
   * confirmation lives in the window, so the window is brought back first when
   * one is due. With confirmation turned off it goes ahead, which is what
   * turning it off asked for.
   */
  async function identifyAllFromTray() {
    if (unidentified.length === 0 || busy !== null) return;
    if (confirmBeforeIdentify) {
      await showWindow().catch((e) => fail("showing the window", e));
    }
    startIdentifyAll();
  }

  /** Closes to the tray, explaining what that means the first time. */
  function requestClose() {
    if (toldAboutTray) {
      void hideWindow().catch((e) => fail("hiding the window", e));
      return;
    }
    closingNotice = true;
  }

  function acceptClosingNotice() {
    closingNotice = false;
    toldAboutTray = true;
    void saveSettings();
    void hideWindow().catch((e) => fail("hiding the window", e));
  }

  /** When the last check ran, so a failing usbipd is not asked twice a second. */
  let lastUsbipdCheck = 0;

  async function recheckUsbipd() {
    // Two processes per call, and `refresh` reaches this every two seconds
    // while anything is wrong. The state it reports does not change that fast.
    const now = Date.now();
    if (now - lastUsbipdCheck < 10_000) return;
    lastUsbipdCheck = now;
    try {
      const next = await checkUsbipd();
      if (next.status !== usbipd?.status) {
        log("info", `usbipd: ${JSON.stringify(next)}`);
      }
      usbipd = next;
    } catch (e) {
      log("error", `checking usbipd: ${e}`);
    }
  }

  function reveal(target: Parameters<typeof openTarget>[0]) {
    openTarget(target).catch((e) => fail(`opening ${target}`, e));
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

    // Unplugged devices are forgiven whatever happened last time, so plugging
    // one back in is a fresh start for the rules.
    const present = new Set(next.filter((d) => d.present).map((d) => d.instanceId));
    for (const instanceId of [...autoAttachTried]) {
      if (!present.has(instanceId)) autoAttachTried.delete(instanceId);
    }

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
    void drainAutoAttach();
  }

  /**
   * Devices a rule names that could be attached right now.
   *
   * Deliberately waits for identification to finish. A device queued for a
   * probe is left alone until the probe has run, because attaching it first
   * would take it away from Windows before it could be asked what it is — and
   * would also decide the question an identity rule is waiting on.
   */
  function autoAttachReady(): DeviceView[] {
    if (!autoAttach) return [];
    return devices.filter(
      (device) =>
        device.autoAttach.matched !== null &&
        device.actions.attach &&
        !autoAttachTried.has(device.instanceId) &&
        !probingIds.has(device.instanceId) &&
        !pending.some((p) => p.instanceId === device.instanceId),
    );
  }

  /**
   * Attaches what the rules name, one at a time.
   *
   * Sequential for the same reason the probe queue is: the list is re-read
   * between operations, and usbipd is doing one thing at a time regardless.
   */
  async function drainAutoAttach() {
    if (autoAttaching) return;
    autoAttaching = true;
    try {
      for (;;) {
        // A user operation owns usbipd while it runs, including the UAC prompt
        // in front of it.
        if (busy) break;
        const [device] = autoAttachReady();
        if (!device) break;

        // Marked before the attempt, not after: a failure must not come back
        // round on the next poll and fail again.
        autoAttachTried.add(device.instanceId);
        log(
          "info",
          `auto attach ${device.instanceId} (matched on ${device.autoAttach.matched})`,
        );
        await attachAutomatically(device);
      }
    } finally {
      autoAttaching = false;
    }
  }

  /**
   * Runs one attach the user did not ask for.
   *
   * No modal overlay, for the same reason an automatic probe has none: taking
   * the window away for something they did not start is worse than the small
   * chance of a click landing during it. The row says what is happening.
   */
  async function attachAutomatically(device: DeviceView) {
    attachingIds = new Set(attachingIds).add(device.instanceId);
    try {
      devices = await runOperation(device.instanceId, "attach");
      error = null;
    } catch (e) {
      fail(`auto attach on ${device.instanceId}`, e);
      await refresh();
    } finally {
      const next = new Set(attachingIds);
      next.delete(device.instanceId);
      attachingIds = next;
    }
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
    // Already identified this session. Nothing is read back from disk: an
    // identity lasts only as long as the device stays plugged in (R4.3).
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
        autoAttach = stored.settings.autoAttach;
        autoAttachRules = stored.settings.autoAttachRules;
        toldAboutTray = stored.settings.toldAboutTray;
        settingsWritable = stored.writable;
        settingsPath = stored.path;
        logPath = stored.logPath;
        remembered = stored.remembered;
        settingsLoaded = true;
        log("info", `settings from ${stored.path}`);
      })
      .catch((e) => fail("reading the settings", e));

    // The tray closes the window and flips automatic attach, so both come back
    // as events rather than as something this side started.
    const unlisten: Promise<() => void>[] = [
      onCloseRequested(requestClose),
      onIdentifyAll(() => void identifyAllFromTray()),
      // The window is shown by the backend before this arrives.
      onOpenSettings(() => (settingsOpen = true)),
      onSettingsChanged(() =>
        readSettings()
          .then((stored) => {
            autoAttach = stored.settings.autoAttach;
            autoAttachRules = stored.settings.autoAttachRules;
            autoAttachTried.clear();
            void drainAutoAttach();
          })
          .catch((e) => fail("re-reading the settings", e)),
      ),
    ];

    appVersion()
      .then((found) => (version = found))
      .catch((e) => log("error", `reading the version: ${e}`));

    void recheckUsbipd();
    refresh();
    // Polling stands in for the device-change notifications of requirement
    // R8.1. It is safe because listing never probes (R4.6).
    //
    // Suspended while an operation runs: usbipd is busy with that, and a reply
    // that arrives mid-attach describes a state that is already gone.
    const timer = setInterval(() => {
      if (!busy && !autoAttaching) refresh();
    }, 2000);
    return () => {
      clearInterval(timer);
      for (const pending of unlisten) void pending.then((stop) => stop());
    };
  });

  /** The probe that could run against a device, or why none can. */
  function probeFor(device: DeviceView): { probe: ProbeOption } | { reason: string } {
    const probe = device.probes.find((p) => p.available);
    if (probe) return { probe };
    const blocked = device.probes.find((p) => p.reason);
    return { reason: t(blocked?.reason ?? "probe.blocked.none") };
  }

  function saveSettings() {
    // Nothing is saved until the stored values have arrived, so an early change
    // cannot write defaults over the file.
    if (!settingsLoaded) return Promise.resolve();
    return writeSettings({
      autoIdentify,
      autoExclude: [...excluded()],
      confirmBeforeIdentify,
      startWithWindows,
      autoAttach,
      autoAttachRules,
      toldAboutTray,
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
      // A failure says nothing about how far usbipd got. The list is only
      // handed back on success, so without this the screen keeps showing what
      // was true before the attempt — next to an error saying it was not done.
      await refresh();
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
      showToast(t("menu.copied"));
    } catch (e) {
      fail("copying to the clipboard", e);
    }
  }

  let toastTimer: ReturnType<typeof setTimeout> | undefined;

  /** `seconds` is longer for anything with a number in it worth reading. */
  function showToast(message: string, seconds = 1.4) {
    toast = message;
    clearTimeout(toastTimer);
    toastTimer = setTimeout(() => (toast = null), seconds * 1000);
  }

  const sameValue = (a: string, b: string) =>
    a.trim().toLowerCase() === b.trim().toLowerCase();

  const covers = (rule: AutoAttachRule, candidate: Candidate) =>
    rule.kind === candidate.kind && sameValue(rule.value, candidate.value);

  /** How many listed devices a rule names right now. */
  function countMatching(rule: AutoAttachRule): number {
    return devices.filter((device) =>
      device.autoAttach.candidates.some((candidate) => covers(rule, candidate)),
    ).length;
  }

  function applyRules(next: AutoAttachRule[]) {
    autoAttachRules = next;
    // A changed rule set deserves a fresh look at everything: a device passed
    // over because nothing named it may be named now.
    autoAttachTried.clear();
    void saveSettings();
    void drainAutoAttach();
  }

  function toggleAutoAttach() {
    autoAttach = !autoAttach;
    autoAttachTried.clear();
    void saveSettings();
    void drainAutoAttach();
  }

  /**
   * Sets which of a device's own values a rule names it by, or none.
   *
   * The choices are exclusive — one device, one rule — so picking one drops the
   * others. That is worth saying out loud in one case: a VID:PID rule is not
   * this device's alone, and dropping it also stops every other device of the
   * same kind being attached. The notice below is there so that does not happen
   * silently.
   */
  function setAutoAttachRule(device: DeviceView, candidate: Candidate | null) {
    const own = device.autoAttach.candidates;
    const ours = (rule: AutoAttachRule) => own.some((c) => covers(rule, c));

    // Choosing what is already chosen changes nothing.
    if (candidate && autoAttachRules.some((rule) => covers(rule, candidate))) return;
    if (!candidate && !autoAttachRules.some(ours)) return;

    const dropped = autoAttachRules.filter(ours);
    const kept = autoAttachRules.filter((rule) => !ours(rule));
    const next = candidate
      ? [...kept, { kind: candidate.kind, value: candidate.value }]
      : kept;

    const wider = dropped.find((rule) => countMatching(rule) > 1);
    if (wider) {
      showToast(
        t("auto_attach.removed_shared", {
          kind: t(`auto_attach.kind.${wider.kind}`),
          count: countMatching(wider) - 1,
        }),
        5,
      );
    }

    applyRules(next);
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
        class="toggle"
        class:on={autoAttach}
        title={t("toolbar.auto_attach.hint")}
        onclick={toggleAutoAttach}
      >
        {t("toolbar.auto_attach")}
        <span class="badge">{autoAttach ? t("toolbar.on") : t("toolbar.off")}</span>
      </button>
      <button
        title={t("toolbar.auto_attach.rules.hint")}
        onclick={() => (autoAttachOpen = true)}
      >
        {t("toolbar.auto_attach.rules")}
        {#if autoAttachRules.length > 0}
          <span class="count">{autoAttachRules.length}</span>
        {/if}
      </button>
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

  {#if usbipd && usbipd.status !== "ok"}
    <!-- Not the error banner: this is a condition rather than an event, it
         stays until it is fixed, and it says what to do about it. -->
    <div class="precondition">
      <div class="precondition-text">
        <strong>{t(`usbipd.${usbipd.status}`)}</strong>
        <span>{t(`usbipd.${usbipd.status}.what`)}</span>
        {#if usbipd.status === "not_answering"}
          <code>{usbipd.detail}</code>
        {/if}
      </div>
      {#if usbipd.status === "not_installed"}
        <button onclick={() => reveal("usbipd_releases")}>{t("usbipd.get")}</button>
      {/if}
    </div>
  {:else if usbipd?.status === "ok" && !usbipd.supported}
    <div class="precondition mild">
      <div class="precondition-text">
        <strong>{t("usbipd.old")}</strong>
        <span>{t("usbipd.old.what", { version: usbipd.version })}</span>
      </div>
    </div>
  {/if}

  <div class="scroll">
    <DeviceTable
      devices={shown}
      selected={selectedId}
      probing={probingIds}
      attaching={attachingIds}
      autoAttachOn={autoAttach}
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
    <div class="error">
      <span class="error-text">{error}</span>
      <button class="dismiss" title={t("error.dismiss")} onclick={() => (error = null)}>
        &times;
      </button>
    </div>
  {/if}

  <!-- Fixed height: selecting a device must not resize the list above it. -->
  <footer>
    {#if selected}
      {@const identity = selected.identity}
      {@const outcome = probeFor(selected)}
      <div class="detail">
        <div class="detail-head">
          <strong>{selected.name}</strong>
          {#if selected.lastName}
            <span class="was" title={t("name.last.hint")}>{selected.lastName}</span>
          {/if}
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
              {#if identity.idSource}
                <!-- Whether the board was asked or merely read. The difference
                     is what it cost to find out, which is worth stating where
                     there is room to state it. -->
                <span class="note">{t(`id_source.${identity.idSource}`)}</span>
              {/if}
            {:else if selected.lastIdentity}
              <code class="key last">{selected.lastIdentity.identityKey}</code>
              <span
                class="note"
                title={t("target.last.hint", { at: selected.lastIdentifiedAt ?? "—" })}
                >{t("detail.target.last", { at: selected.lastIdentifiedAt ?? "—" })}</span
              >
            {:else if selected.needsProbe}
              <span class="note">{t("detail.target.unknown")}</span>
            {:else}
              <span class="note">{t("detail.target.transport_only")}</span>
            {/if}
          </dd>
        </dl>
      </div>

      <div class="detail-side">
        {#if selected.autoAttach.candidates.length > 0}
          {@const chosen = selected.autoAttach.candidates.find(
            (c) => c.kind === selected.autoAttach.matched,
          )}
          <!-- Exclusive by design: one device is attached because of one thing
               about it, and which thing that is, is the whole question. -->
          <div class="auto-attach" class:off={!autoAttach}>
            <span class="aa-label">{t("detail.auto_attach")}</span>
            <label class="aa-option">
              <input
                type="radio"
                name="auto-attach"
                checked={selected.autoAttach.matched === null}
                onchange={() => setAutoAttachRule(selected, null)}
              />
              <span>{t("auto_attach.none")}</span>
            </label>
            {#each selected.autoAttach.candidates as candidate (candidate.kind)}
              <label class="aa-option" title={t(`auto_attach.kind.${candidate.kind}.hint`)}>
                <input
                  type="radio"
                  name="auto-attach"
                  checked={selected.autoAttach.matched === candidate.kind}
                  onchange={() => setAutoAttachRule(selected, candidate)}
                />
                <span>{t(`auto_attach.kind.${candidate.kind}`)}</span>
              </label>
            {/each}
          </div>
          <p class="aa-value">
            {#if !autoAttach}
              {t("detail.auto_attach.off")}
            {:else if chosen}
              <code>{chosen.value}</code>
            {:else}
              {t("detail.auto_attach.none")}
            {/if}
          </p>
        {/if}

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
    {settingsPath}
    {logPath}
    {version}
    onopen={reveal}
    enabled={autoIdentify}
    excludeList={autoExcludeList}
    {confirmBeforeIdentify}
    {startWithWindows}
    graceSeconds={GRACE_SECONDS}
    writable={settingsWritable}
    {remembered}
    onforget={() => {
      // Refreshing afterwards is what makes it visible: the greyed values are
      // in the rows, not in this panel.
      clearRemembered()
        .then((dropped) => {
          remembered = 0;
          log("info", `forgot ${dropped} remembered device(s)`);
          return refresh();
        })
        .catch((e) => fail("clearing the remembered devices", e));
    }}
    onchange={(next) => {
      autoIdentify = next.enabled;
      autoExcludeList = next.excludeList;
      confirmBeforeIdentify = next.confirmBeforeIdentify;
      startWithWindows = next.startWithWindows;
      void saveSettings();
    }}
    onclose={() => (settingsOpen = false)}
  />
{/if}

{#if closingNotice}
  <!-- Said once, and only because an application that keeps running after its
       window is gone has to say so. -->
  <div class="backdrop" role="presentation"></div>
  <div class="dialog" role="dialog" aria-modal="true" aria-labelledby="tray-title">
    <h2 id="tray-title">{t("tray.notice.title")}</h2>
    <p class="effect">{t("tray.notice.body")}</p>
    <div class="dialog-actions">
      <button class="primary" onclick={acceptClosingNotice}>{t("tray.notice.ok")}</button>
    </div>
  </div>
{/if}

{#if autoAttachOpen}
  <AutoAttachPanel
    enabled={autoAttach}
    rules={autoAttachRules}
    {devices}
    writable={settingsWritable}
    onchange={(next) => {
      autoAttach = next.enabled;
      applyRules(next.rules);
    }}
    onclose={() => (autoAttachOpen = false)}
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
    /* The toolbar wins the space it needs and the tabs scroll instead: a
       squashed button wraps its label onto two lines and changes the height of
       the whole bar. */
    flex: 0 0 auto;
  }

  nav .actions button {
    white-space: nowrap;
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

  nav .toggle {
    display: flex;
    align-items: center;
    gap: 7px;
  }

  nav .toggle.on {
    border-color: color-mix(in srgb, var(--accent) 60%, transparent);
  }

  nav .toggle .badge {
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.04em;
    color: var(--fg-faint);
  }

  nav .toggle.on .badge {
    color: var(--accent);
  }

  .detail-side {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 6px;
    min-width: 0;
  }

  .auto-attach {
    display: flex;
    flex-wrap: wrap;
    justify-content: flex-end;
    align-items: center;
    gap: 2px 10px;
    font-size: 12px;
  }

  .auto-attach.off {
    opacity: 0.6;
  }

  .aa-label {
    color: var(--fg-muted);
    margin-right: auto;
  }

  .aa-option {
    display: flex;
    align-items: center;
    gap: 3px;
    white-space: nowrap;
  }

  .aa-option input {
    margin: 0;
  }

  .aa-value {
    margin: 0;
    max-width: 100%;
    font-size: 11px;
    color: var(--fg-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
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
    /* Wide enough for the longest label ("USB シリアル番号") on one line: a
       label that wraps changes the height of every row beside it. */
    grid-template-columns: 100px minmax(0, 1fr) 96px minmax(0, 1fr);
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
    white-space: nowrap;
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

  /* What was last read, rather than what is known now. Greyed and italic in
     both places it appears, so the distinction does not have to be relearned
     between the list and this pane. */
  .key.last,
  .was {
    color: var(--fg-faint);
    font-weight: 400;
    font-style: italic;
  }

  .was {
    font-size: 12px;
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
    margin-top: auto;
  }

  /* A precondition is not an event: it sits above the list until whatever it
     names is dealt with, rather than being dismissed. */
  .precondition {
    flex: 0 0 auto;
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 12px;
    padding: 10px 14px;
    font-size: 12px;
    line-height: 1.6;
    color: var(--fg);
    background: color-mix(in srgb, var(--warn) 12%, transparent);
    border-bottom: 1px solid color-mix(in srgb, var(--warn) 40%, transparent);
    user-select: text;
  }

  .precondition.mild {
    background: color-mix(in srgb, var(--fg-muted) 8%, transparent);
    border-bottom-color: var(--border);
  }

  .precondition-text {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .precondition code {
    font-size: 11px;
    color: var(--fg-muted);
    white-space: pre-wrap;
    word-break: break-word;
  }

  .precondition button {
    flex: 0 0 auto;
  }

  /* Above the footer, not after it: the footer has a fixed height, so an
     error appended below it was pushed out of the window. */
  .error {
    flex: 0 0 auto;
    display: flex;
    align-items: flex-start;
    gap: 10px;
    padding: 8px 14px;
    font-size: 12px;
    line-height: 1.5;
    color: var(--danger);
    background: color-mix(in srgb, var(--danger) 10%, transparent);
    border-top: 1px solid color-mix(in srgb, var(--danger) 30%, transparent);
    user-select: text;
  }

  /* usbipd puts several lines on stderr. Enough room to read them, capped so a
     long one cannot push the device list off the window. */
  .error-text {
    flex: 1 1 auto;
    max-height: 7em;
    overflow-y: auto;
    white-space: pre-wrap;
    word-break: break-word;
  }

  .dismiss {
    all: unset;
    flex: 0 0 auto;
    padding: 0 6px;
    font-size: 15px;
    line-height: 1;
    color: var(--danger);
    cursor: default;
  }

  .dismiss:hover {
    opacity: 0.7;
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
