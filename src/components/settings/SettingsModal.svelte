<script lang="ts">
  import Modal from "../shared/Modal.svelte";
  import LabeledField from "../shared/LabeledField.svelte";
  import { appStore } from "../../lib/stores/app.svelte";

  let { onclose }: { onclose: () => void } = $props();

  type ThemeMode = "system" | "light" | "dark";

  let themeMode = $state<ThemeMode>(
    (localStorage.getItem("themeMode") as ThemeMode) ?? "system",
  );

  function applyTheme(mode: ThemeMode) {
    themeMode = mode;
    localStorage.setItem("themeMode", mode);

    if (mode === "system") {
      localStorage.removeItem("theme");
      const prefersDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
      document.documentElement.setAttribute("data-theme", prefersDark ? "dark" : "light");
    } else {
      localStorage.setItem("theme", mode);
      document.documentElement.setAttribute("data-theme", mode);
    }
  }

  const themeModes: { value: ThemeMode; label: string }[] = [
    { value: "system", label: "System" },
    { value: "light", label: "Light" },
    { value: "dark", label: "Dark" },
  ];
</script>

<Modal title="Settings" {onclose}>
  <div class="settings">
    <section class="section">
      <h3 class="section-title">Appearance</h3>
      <div class="section-body">
        <div class="field-row">
          <span class="field-label">Theme</span>
          <div class="theme-options">
            {#each themeModes as mode}
              <button
                class="theme-option"
                class:active={themeMode === mode.value}
                onclick={() => applyTheme(mode.value)}
              >
                {mode.label}
              </button>
            {/each}
          </div>
        </div>
      </div>
    </section>

    <section class="section">
      <div class="section-header">
        <h3 class="section-title">Proxy</h3>
        <label class="toggle-label">
          <input
            type="checkbox"
            bind:checked={appStore.proxyConfig.enabled}
            onchange={() => appStore.scheduleSave()}
          />
          <span>Enabled</span>
        </label>
      </div>
      <div class="section-body">
        <LabeledField
          label="Proxy URL"
          bind:value={appStore.proxyConfig.proxyUrl}
          placeholder="https://proxy:8080"
          hint={appStore.proxyConfig.enabled && appStore.proxyConfig.proxyUrl.startsWith("http://")
            ? "Warning: HTTP proxy is unencrypted. Use HTTPS for sensitive traffic."
            : ""}
          disabled={!appStore.proxyConfig.enabled}
          oninput={() => appStore.scheduleSave()}
        />

        <LabeledField
          label="No Proxy"
          bind:value={appStore.proxyConfig.noProxy}
          placeholder="localhost, 127.0.0.1, .internal"
          hint="Comma-separated hostnames to bypass proxy"
          disabled={!appStore.proxyConfig.enabled}
          oninput={() => appStore.scheduleSave()}
        />
      </div>
    </section>
  </div>
</Modal>

<style>
  .settings {
    min-width: 420px;
    display: flex;
    flex-direction: column;
    gap: var(--sp-lg);
  }
  .section {
    display: flex;
    flex-direction: column;
    gap: var(--sp-sm);
  }
  .section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  .section-title {
    font-size: var(--fs-callout);
    font-weight: 600;
    color: var(--text-secondary);
    margin: 0;
  }
  .section-body {
    display: flex;
    flex-direction: column;
    gap: var(--sp-sm);
  }
  .field-row {
    display: flex;
    align-items: center;
    gap: var(--sp-md);
  }
  .field-label {
    font-size: var(--fs-callout);
    font-weight: 500;
    color: var(--text-secondary);
    min-width: 80px;
    flex-shrink: 0;
  }
  .toggle-label {
    display: flex;
    align-items: center;
    gap: var(--sp-xs);
    font-size: var(--fs-small);
    color: var(--text-tertiary);
    cursor: pointer;
  }
  .toggle-label input[type="checkbox"] {
    width: 14px;
    height: 14px;
    cursor: pointer;
  }
  .theme-options {
    display: flex;
    gap: var(--sp-xs);
    background: var(--bg-tertiary);
    border-radius: var(--radius-md);
    padding: 2px;
  }
  .theme-option {
    flex: 1;
    padding: var(--sp-xs) var(--sp-md);
    font-size: var(--fs-callout);
    color: var(--text-secondary);
    border-radius: var(--radius-sm);
    text-align: center;
  }
  .theme-option:hover {
    color: var(--text-primary);
  }
  .theme-option.active {
    background: var(--bg-primary);
    color: var(--text-primary);
    font-weight: 600;
    box-shadow: var(--shadow-elevated);
  }
</style>
