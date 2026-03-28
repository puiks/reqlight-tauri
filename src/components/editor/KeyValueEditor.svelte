<script lang="ts">
  import { createEmptyPair, type KeyValuePair } from "../../lib/types";

  let {
    pairs = $bindable(),
    showSecret = false,
    onchange,
  }: {
    pairs: KeyValuePair[];
    showSecret?: boolean;
    onchange?: () => void;
  } = $props();

  function handleInput() {
    ensureEmptyRow();
    onchange?.();
  }

  function toggleEnabled(id: string) {
    pairs = pairs.map((p) =>
      p.id === id ? { ...p, isEnabled: !p.isEnabled } : p,
    );
    onchange?.();
  }

  function toggleSecret(id: string) {
    pairs = pairs.map((p) =>
      p.id === id ? { ...p, isSecret: !p.isSecret } : p,
    );
    onchange?.();
  }

  function deleteRow(id: string) {
    pairs = pairs.filter((p) => p.id !== id);
    ensureEmptyRow();
    onchange?.();
  }

  function ensureEmptyRow() {
    const lastPair = pairs[pairs.length - 1];
    if (!lastPair || lastPair.key || lastPair.value) {
      pairs = [...pairs, createEmptyPair()];
    }
  }

  function updatePair(id: string, field: "key" | "value", val: string) {
    pairs = pairs.map((p) =>
      p.id === id ? { ...p, [field]: val } : p,
    );
    handleInput();
  }
</script>

<div class="kv-editor">
  <div class="header-row">
    <span class="checkbox-col"></span>
    <span class="key-col">Key</span>
    <span class="value-col">Value</span>
    {#if showSecret}<span class="secret-col">Secret</span>{/if}
    <span class="action-col"></span>
  </div>
  {#each pairs as pair (pair.id)}
    <div class="row" class:disabled={!pair.isEnabled}>
      <span class="checkbox-col">
        <input
          type="checkbox"
          checked={pair.isEnabled}
          onchange={() => toggleEnabled(pair.id)}
        />
      </span>
      <input
        class="key-col"
        type="text"
        placeholder="Key"
        value={pair.key}
        oninput={(e) => updatePair(pair.id, "key", e.currentTarget.value)}
      />
      <input
        class="value-col"
        type={pair.isSecret ? "password" : "text"}
        placeholder="Value"
        value={pair.value}
        oninput={(e) => updatePair(pair.id, "value", e.currentTarget.value)}
      />
      {#if showSecret}
        <span class="secret-col">
          <button
            class="secret-btn"
            class:active={pair.isSecret}
            onclick={() => toggleSecret(pair.id)}
            title={pair.isSecret ? "Secret (stored in keychain)" : "Not secret"}
          >
            {pair.isSecret ? "🔒" : "🔓"}
          </button>
        </span>
      {/if}
      <span class="action-col">
        {#if pair.key || pair.value}
          <button class="delete-btn" onclick={() => deleteRow(pair.id)}>×</button>
        {/if}
      </span>
    </div>
  {/each}
</div>

<style>
  .kv-editor {
    font-size: var(--fs-small);
  }
  .header-row {
    display: flex;
    align-items: center;
    gap: 1px;
    padding: var(--sp-xs) 0;
    font-size: var(--fs-footnote);
    font-weight: 600;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.3px;
    border-bottom: 1px solid var(--border-color);
  }
  .row {
    display: flex;
    align-items: center;
    gap: 1px;
    min-height: 32px;
    border-bottom: 1px solid var(--border-light);
  }
  .row.disabled {
    opacity: 0.5;
  }
  .row input[type="text"],
  .row input[type="password"] {
    border: none;
    background: transparent;
    padding: var(--sp-xs) var(--sp-sm);
    font-family: var(--font-mono);
    font-size: var(--fs-small);
    border-radius: 0;
  }
  .row input[type="text"]:focus,
  .row input[type="password"]:focus {
    background: var(--bg-hover);
    box-shadow: none;
  }
  .checkbox-col {
    width: 30px;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }
  .key-col {
    flex: 1;
    min-width: 0;
  }
  .value-col {
    flex: 1.5;
    min-width: 0;
  }
  .secret-col {
    width: 40px;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }
  .secret-btn {
    font-size: var(--fs-small);
    padding: 2px;
  }
  .action-col {
    width: 28px;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }
  .delete-btn {
    font-size: 16px;
    color: var(--text-tertiary);
    padding: 0 4px;
    line-height: 1;
  }
  .delete-btn:hover {
    color: var(--color-error);
  }
</style>
