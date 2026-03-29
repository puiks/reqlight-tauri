<script lang="ts">
  import { parseCurl } from "../../lib/commands";
  import { appStore } from "../../lib/stores/app.svelte";
  import { editorStore } from "../../lib/stores/editor.svelte";
  import { toastStore } from "../../lib/stores/toast.svelte";
  import Modal from "../shared/Modal.svelte";

  let { onclose }: { onclose: () => void } = $props();

  let curlText = $state("");
  let errorMsg = $state<string | null>(null);

  async function handleImport() {
    if (!curlText.trim()) return;
    errorMsg = null;
    try {
      const request = await parseCurl(curlText);
      // Add to first collection or create one
      let collectionId = appStore.collections[0]?.id;
      if (!collectionId) {
        const c = appStore.addCollection("Imported");
        collectionId = c.id;
      }
      const added = appStore.addRequest(collectionId, request.name || "Imported Request");
      // Update with parsed data
      const updated = { ...added, ...request, id: added.id };
      appStore.updateRequest(updated);
      editorStore.loadFrom(updated);
      toastStore.show("cURL imported successfully");
      onclose();
    } catch (e) {
      errorMsg = e instanceof Error ? e.message : String(e);
    }
  }
</script>

<Modal title="Import cURL" {onclose}>
  <div class="curl-import">
    <textarea
      class="curl-input"
      placeholder="Paste your cURL command here..."
      bind:value={curlText}
      spellcheck="false"
    ></textarea>
    {#if errorMsg}
      <div class="error">{errorMsg}</div>
    {/if}
    <div class="actions">
      <button class="cancel" onclick={onclose}>Cancel</button>
      <button class="import-btn" onclick={handleImport} disabled={!curlText.trim()}>
        Import
      </button>
    </div>
  </div>
</Modal>

<style>
  .curl-import {
    display: flex;
    flex-direction: column;
    gap: var(--sp-md);
  }
  .curl-input {
    font-family: var(--font-mono);
    font-size: var(--fs-small);
    min-height: 200px;
    resize: vertical;
    padding: var(--sp-sm);
    line-height: 1.5;
  }
  .error {
    color: var(--color-error);
    font-size: var(--fs-small);
    padding: var(--sp-xs) var(--sp-sm);
    background: var(--color-error-overlay);
    border-radius: var(--radius-sm);
  }
  .actions {
    display: flex;
    justify-content: flex-end;
    gap: var(--sp-sm);
  }
  .cancel {
    background: var(--bg-tertiary);
    padding: var(--sp-xs) var(--sp-md);
  }
  .import-btn {
    background: var(--color-info);
    color: white;
    font-weight: 600;
    padding: var(--sp-xs) var(--sp-lg);
  }
  .import-btn:hover:not(:disabled) {
    opacity: 0.9;
    background: var(--color-info);
  }
  .import-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
