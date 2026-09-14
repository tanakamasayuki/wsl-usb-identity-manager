<script lang="ts">
  import { t } from "./i18n";

  interface Props {
    enabled: boolean;
    /** VID:PID that must never be probed automatically. */
    excludeList: string;
    /** The window, in seconds, during which an arrival may be probed (R4.8). */
    graceSeconds: number;
    onchange: (next: { enabled: boolean; excludeList: string }) => void;
    onclose: () => void;
  }

  let { enabled, excludeList, graceSeconds, onchange, onclose }: Props = $props();
</script>

<svelte:window onkeydown={(e) => e.key === "Escape" && onclose()} />

<div class="backdrop" role="presentation" onclick={onclose}></div>
<div class="panel" role="dialog" aria-modal="true" aria-labelledby="settings-title">
  <h2 id="settings-title">{t("settings.title")}</h2>

  <label class="row">
    <input
      type="checkbox"
      checked={enabled}
      onchange={(e) => onchange({ enabled: e.currentTarget.checked, excludeList })}
    />
    <span>{t("settings.auto.label")}</span>
  </label>
  <p class="note">{t("settings.auto.note", { seconds: graceSeconds })}</p>

  <label class="field">
    <span>{t("settings.exclude.label")}</span>
    <input
      type="text"
      value={excludeList}
      placeholder={t("settings.exclude.placeholder")}
      oninput={(e) => onchange({ enabled, excludeList: e.currentTarget.value })}
    />
  </label>
  <p class="note">{t("settings.exclude.note")}</p>

  <p class="note warn">{t("settings.unsaved")}</p>

  <div class="actions">
    <button onclick={onclose}>{t("settings.close")}</button>
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.35);
  }

  .panel {
    position: fixed;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    width: min(520px, calc(100vw - 32px));
    background: var(--bg-header);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 20px;
    box-shadow: 0 18px 48px rgba(0, 0, 0, 0.45);
  }

  h2 {
    margin: 0 0 16px;
    font-size: 15px;
  }

  .row {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
  }

  .field {
    display: block;
    margin-top: 16px;
    font-size: 13px;
  }

  .field input {
    display: block;
    width: 100%;
    margin-top: 6px;
    padding: 6px 8px;
    font: inherit;
    font-family: var(--mono);
    color: var(--fg);
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 5px;
  }

  .note {
    margin: 6px 0 0;
    font-size: 12px;
    line-height: 1.6;
    color: var(--fg-muted);
  }

  .note.warn {
    margin-top: 16px;
    color: var(--warn);
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    margin-top: 18px;
  }
</style>
