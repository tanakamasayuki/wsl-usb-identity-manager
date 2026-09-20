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
    /** Where `vhfilter.exe` is, when it is not somewhere already searched. */
    vhfilterPath: string;
    /** Where it was found, if it was, and where else was looked. */
    vhfilter: string | null;
    vhfilterHow: "configured" | "searched" | null;
    vhfilterSearchPath: string[];
    onvhfiltersetup: () => void;
    onchange: (next: {
      enabled: boolean;
      excludeList: string;
      confirmBeforeIdentify: boolean;
      startWithWindows: boolean;
      vhfilterPath: string;
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
    vhfilterPath,
    vhfilter,
    vhfilterHow,
    vhfilterSearchPath,
    onvhfiltersetup,
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
      vhfilterPath,
      ...patch,
    });
  }
</script>

<svelte:window onkeydown={(e) => e.key === "Escape" && onclose()} />

<div class="backdrop" role="presentation" onclick={onclose}></div>
<div class="panel" role="dialog" aria-modal="true" aria-labelledby="settings-title">
  <h2 id="settings-title">{t("settings.title")}</h2>

  <!-- Two columns: what the user changes on the left, what they look up on the
       right. In one column the panel grew taller than the window, and a
       settings dialog that scrolls hides the thing someone opened it for. -->
  <div class="columns">
    <section>
      <h3>{t("settings.behaviour")}</h3>

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
    </section>

    <section>
      <h3>{t("settings.remembered")}</h3>
      <!-- The one part of the file that can be wrong without anything having gone
           wrong: a board moved to another port leaves its old entry behind (R7.9). -->
      <div class="file">
        <div class="file-text">
          <span>{t("settings.remembered.count", { count: remembered })}</span>
        </div>
        <button disabled={remembered === 0} onclick={onforget}>
          {t("settings.remembered.forget")}
        </button>
      </div>
      <!-- Below the row rather than beside the button: squeezed into half a
           column it wrapped to four lines and set the section's height. -->
      <p class="note">{t("settings.remembered.note")}</p>

          <h3>{t("settings.ppps")}</h3>
      <!-- Found rather than shipped: the tool is VirtualHere's, ships as a bare
           executable with no installer and no stated redistribution terms. -->
      {#if vhfilter}
        <!-- "Found on its own" is worth saying: it is what tells the user the
             search did its job and the path below can stay empty. -->
        <div class="file">
          <div class="file-text">
            <span class="ok"
              >&check; {vhfilterHow === "configured"
                ? t("settings.ppps.from_path")
                : t("settings.ppps.detected")}</span
            >
            <code>{vhfilter}</code>
          </div>
        </div>
      {:else}
        <p class="note">{t("settings.ppps.missing")}</p>
        {#each vhfilterSearchPath as dir (dir)}
          <p class="note"><code>{dir}</code></p>
        {/each}
        <div class="file">
          <div class="file-text">
            <span class="note">{t("settings.ppps.setup.note")}</span>
          </div>
          <button onclick={onvhfiltersetup}>{t("settings.ppps.setup")}</button>
        </div>
      {/if}
      <!-- No placeholder on the input: an example path would name a folder that
           does not exist and is not one of the ones searched, which reads as an
           instruction to put the file there. -->
      <label class="field">
        <span>{t("settings.ppps.path")}</span>
        <input
          type="text"
          value={vhfilterPath}
          oninput={(e) => change({ vhfilterPath: e.currentTarget.value })}
        />
      </label>

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
    </section>
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
    width: min(860px, calc(100vw - 32px));
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

  .columns {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 0 32px;
    align-items: start;
  }

  /* Both columns start level, whatever each one happens to begin with. */
  .columns > section > :first-child {
    margin-top: 0;
  }

  /* Narrower than two columns can hold. The window cannot be made this small,
     but nothing here depends on that staying true. */
  @media (max-width: 720px) {
    .columns {
      grid-template-columns: 1fr;
    }
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

  .ok {
    color: var(--ok);
    font-weight: 600;
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
