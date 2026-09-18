<script lang="ts">
  import { t } from "./i18n";
  import type { OpenTarget } from "./types";

  interface Props {
    /** Where the settings file is. Shown so it can be found and read. */
    settingsPath: string;
    /** Where the log is. A release build has no console to print it to. */
    logPath: string;
    /** Which build this is. */
    version: string;
    onopen: (target: OpenTarget) => void;
    enabled: boolean;
    /** VID:PID that must never be probed automatically. */
    excludeList: string;
    /** Ask before probing, stating what it does to the board (R4.7). */
    confirmBeforeIdentify: boolean;
    startWithWindows: boolean;
    /** The window, in seconds, during which an arrival may be probed (R4.8). */
    graceSeconds: number;
    /** False when the stored file was refused; nothing is being saved. */
    writable: boolean;
    /** How many devices have a remembered name or identification (R4.21). */
    remembered: number;
    onforget: () => void;
    onchange: (next: {
      enabled: boolean;
      excludeList: string;
      confirmBeforeIdentify: boolean;
      startWithWindows: boolean;
    }) => void;
    onclose: () => void;
  }

  let {
    settingsPath,
    logPath,
    version,
    onopen,
    enabled,
    excludeList,
    confirmBeforeIdentify,
    startWithWindows,
    graceSeconds,
    writable,
    remembered,
    onforget,
    onchange,
    onclose,
  }: Props = $props();

  /** Current values with one field replaced, so each control sends the whole set. */
  function change(patch: Partial<Parameters<Props["onchange"]>[0]>) {
    onchange({
      enabled,
      excludeList,
      confirmBeforeIdentify,
      startWithWindows,
      ...patch,
    });
  }
</script>

<svelte:window onkeydown={(e) => e.key === "Escape" && onclose()} />

<div class="backdrop" role="presentation" onclick={onclose}></div>
<div class="panel" role="dialog" aria-modal="true" aria-labelledby="settings-title">
  <h2 id="settings-title">{t("settings.title")}</h2>

  <label class="row">
    <input
      type="checkbox"
      checked={enabled}
      onchange={(e) => change({ enabled: e.currentTarget.checked })}
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
      oninput={(e) => change({ excludeList: e.currentTarget.value })}
    />
  </label>
  <p class="note">{t("settings.exclude.note")}</p>

  <label class="row spaced">
    <input
      type="checkbox"
      checked={confirmBeforeIdentify}
      onchange={(e) => change({ confirmBeforeIdentify: e.currentTarget.checked })}
    />
    <span>{t("settings.confirm.label")}</span>
  </label>
  <p class="note">{t("settings.confirm.note")}</p>

  <label class="row spaced">
    <input
      type="checkbox"
      checked={startWithWindows}
      onchange={(e) => change({ startWithWindows: e.currentTarget.checked })}
    />
    <span>{t("settings.startup.label")}</span>
  </label>
  <p class="note">{t("settings.startup.note")}</p>

  {#if !writable}
    <p class="note warn">{t("settings.unsaved")}</p>
  {/if}

  <h3>{t("settings.remembered")}</h3>
  <!-- The one part of the file that can be wrong without anything having gone
       wrong: a board moved to another port leaves its old entry behind (R7.9). -->
  <div class="file">
    <div class="file-text">
      <span>{t("settings.remembered.count", { count: remembered })}</span>
      <span class="note">{t("settings.remembered.note")}</span>
    </div>
    <button disabled={remembered === 0} onclick={onforget}>
      {t("settings.remembered.forget")}
    </button>
  </div>

  <h3>{t("settings.about")}</h3>
  <!-- Next to the log, because a problem report needs both and this is where
       someone goes looking for the log. -->
  <p class="version"><code>{t("app.name")} {version}</code></p>

  <h3>{t("settings.files")}</h3>
  <!-- The log records every usbipd command and every probe, and is the first
       thing to look at when something goes wrong. A release build has no
       console, so this is the only place its path appears (R13.8). -->
  <div class="file">
    <div class="file-text">
      <span>{t("settings.log")}</span>
      <code>{logPath}</code>
    </div>
    <button onclick={() => onopen("log_folder")}>{t("settings.open_folder")}</button>
  </div>
  <div class="file">
    <div class="file-text">
      <span>{t("settings.settings_file")}</span>
      <code>{settingsPath}</code>
    </div>
    <button onclick={() => onopen("settings_folder")}>{t("settings.open_folder")}</button>
  </div>

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
    max-height: calc(100vh - 48px);
    overflow-y: auto;
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

  .row.spaced {
    margin-top: 16px;
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

  h3 {
    margin: 22px 0 8px;
    font-size: 13px;
    color: var(--fg-muted);
  }

  .version {
    margin: 0;
    font-size: 12px;
  }

  .version code {
    font-size: 12px;
    color: var(--fg-muted);
  }

  .file {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    margin-top: 8px;
  }

  .file-text {
    display: flex;
    flex-direction: column;
    gap: 1px;
    min-width: 0;
    font-size: 12px;
  }

  .file-text code {
    font-size: 11px;
    color: var(--fg-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .file button {
    flex: 0 0 auto;
    font-size: 12px;
    padding: 4px 10px;
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
