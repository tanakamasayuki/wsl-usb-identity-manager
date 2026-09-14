<script lang="ts">
  import { t } from "./i18n";
  import type { DeviceView, ProbeOption } from "./types";

  interface Props {
    device: DeviceView;
    probe: ProbeOption;
    running: boolean;
    onconfirm: () => void;
    oncancel: () => void;
  }

  let { device, probe, running, onconfirm, oncancel }: Props = $props();
</script>

<svelte:window onkeydown={(e) => e.key === "Escape" && !running && oncancel()} />

<!-- Requirement R4.7: the side effects are stated before anything happens. -->
<div class="backdrop" role="presentation" onclick={() => !running && oncancel()}></div>
<div class="dialog" role="dialog" aria-modal="true" aria-labelledby="probe-title">
  <h2 id="probe-title">{t("probe.title")}</h2>

  <dl>
    <dt>{t("probe.device")}</dt>
    <dd>{device.name}</dd>
    <dt>{t("probe.port")}</dt>
    <dd>{device.comPort ?? device.busId ?? "—"}</dd>
    <dt>{t("probe.method")}</dt>
    <dd>{probe.family}</dd>
  </dl>

  <p class="effect">
    <strong>{t("probe.warning")}</strong><br />
    {t(probe.sideEffect)}
  </p>

  <div class="actions">
    <button onclick={oncancel} disabled={running}>{t("probe.cancel")}</button>
    <button class="primary" onclick={onconfirm} disabled={running}>
      {running ? t("probe.running") : t("probe.run")}
    </button>
  </div>
</div>

<style>
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
    width: min(460px, calc(100vw - 32px));
    background: var(--bg-header);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 20px;
    box-shadow: 0 18px 48px rgba(0, 0, 0, 0.45);
  }

  h2 {
    margin: 0 0 14px;
    font-size: 15px;
  }

  dl {
    display: grid;
    grid-template-columns: 80px 1fr;
    gap: 4px 12px;
    margin: 0 0 14px;
    font-size: 13px;
  }

  dt {
    color: var(--fg-muted);
  }

  dd {
    margin: 0;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .effect {
    font-size: 13px;
    line-height: 1.6;
    background: color-mix(in srgb, var(--warn) 12%, transparent);
    border: 1px solid color-mix(in srgb, var(--warn) 40%, transparent);
    border-radius: 6px;
    padding: 10px 12px;
    margin: 0 0 18px;
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }
</style>
