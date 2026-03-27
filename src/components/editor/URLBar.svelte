<script lang="ts">
  import { editorStore } from "../../lib/stores/editor.svelte";
  import { HTTP_METHODS, METHOD_COLORS, type HttpMethod } from "../../lib/types";

  let { onimportcurl }: { onimportcurl: () => void } = $props();

  function handleSend() {
    editorStore.send();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Enter" && (e.metaKey || e.ctrlKey)) {
      handleSend();
    }
  }
</script>

<div class="url-bar">
  <select
    class="method-select"
    bind:value={editorStore.method}
    onchange={() => editorStore.markDirty()}
    style="color: {METHOD_COLORS[editorStore.method]}"
  >
    {#each HTTP_METHODS as m}
      <option value={m} style="color: {METHOD_COLORS[m]}">{m}</option>
    {/each}
  </select>

  <input
    type="text"
    class="url-input"
    placeholder="Enter URL (e.g. https://api.example.com)"
    bind:value={editorStore.url}
    oninput={() => editorStore.markDirty()}
    onkeydown={handleKeydown}
    class:invalid={!editorStore.isUrlValid}
  />

  <div class="options-group">
    <div class="timeout-group" title="Request timeout (seconds)">
      <span class="option-label">Timeout</span>
      <input
        type="number"
        class="timeout-input"
        min="1"
        max="300"
        bind:value={editorStore.timeoutSecs}
      />
      <span class="timeout-label">s</span>
    </div>
    <button
      class="import-curl-btn"
      onclick={onimportcurl}
      title="Import from cURL"
    >
      cURL
    </button>
  </div>

  {#if editorStore.isLoading}
    <button class="cancel-btn" onclick={() => editorStore.cancel()}>Cancel</button>
  {:else}
    <button
      class="send-btn"
      onclick={handleSend}
      disabled={!editorStore.canSend}
      title="Send (⌘↩)"
    >
      Send
    </button>
  {/if}
</div>

<style>
  .url-bar {
    display: flex;
    gap: var(--sp-sm);
    padding: var(--sp-md);
    border-bottom: 1px solid var(--border-color);
    align-items: center;
  }
  .method-select {
    font-family: var(--font-mono);
    font-weight: 600;
    font-size: var(--fs-body);
    background: var(--bg-input);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    padding: var(--sp-xs) var(--sp-sm);
    width: 90px;
    cursor: pointer;
  }
  .url-input {
    flex: 1;
    font-family: var(--font-mono);
    font-size: var(--fs-body);
  }
  .url-input.invalid {
    border-color: var(--color-error);
  }
  .send-btn {
    background: var(--color-info);
    color: white;
    font-weight: 600;
    padding: var(--sp-xs) var(--sp-lg);
    border-radius: var(--radius-sm);
  }
  .send-btn:hover:not(:disabled) {
    opacity: 0.9;
    background: var(--color-info);
  }
  .send-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .cancel-btn {
    background: var(--bg-tertiary);
    font-weight: 600;
    padding: var(--sp-xs) var(--sp-lg);
    border-radius: var(--radius-sm);
  }
  .options-group {
    display: flex;
    align-items: center;
    gap: var(--sp-xs);
  }
  .timeout-group {
    display: flex;
    align-items: center;
    gap: var(--sp-xs);
  }
  .option-label {
    font-size: var(--fs-small);
    color: var(--text-tertiary);
    white-space: nowrap;
  }
  .timeout-input {
    width: 42px;
    text-align: center;
    font-family: var(--font-mono);
    font-size: var(--fs-small);
    padding: var(--sp-xs) 2px;
    -moz-appearance: textfield;
    appearance: textfield;
  }
  .timeout-input::-webkit-inner-spin-button,
  .timeout-input::-webkit-outer-spin-button {
    -webkit-appearance: none;
  }
  .timeout-label {
    font-size: var(--fs-caption);
    color: var(--text-tertiary);
  }
  .import-curl-btn {
    font-size: var(--fs-small);
    font-family: var(--font-mono);
    padding: var(--sp-xs) var(--sp-sm);
    color: var(--text-tertiary);
    border-radius: var(--radius-sm);
    white-space: nowrap;
    border: 1px dashed var(--border-color);
  }
  .import-curl-btn:hover {
    color: var(--text-primary);
    border-color: var(--text-secondary);
  }
</style>
