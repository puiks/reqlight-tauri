<script lang="ts">
  import { editorStore } from "../../lib/stores/editor.svelte";
  import StatusBar from "./StatusBar.svelte";
  import ResponseBody from "./ResponseBody.svelte";
  import ResponseHeaders from "./ResponseHeaders.svelte";
  import EmptyState from "../shared/EmptyState.svelte";
  import type { ResponseTab } from "../../lib/types";

  let { ongeneratecode, oncompare }: { ongeneratecode?: () => void; oncompare?: () => void } = $props();

  const hasPinned = $derived(editorStore.pinnedResponse !== null);
  const canCompare = $derived(hasPinned && editorStore.response !== null);

  const tabs: { value: ResponseTab; label: string }[] = [
    { value: "body", label: "Body" },
    { value: "headers", label: "Headers" },
  ];
</script>

<div class="response-view">
  {#if editorStore.isLoading}
    <div class="loading">
      <span class="spinner"></span>
      <span>Sending request...</span>
    </div>
  {:else if editorStore.errorMessage}
    <div class="error-state">
      <EmptyState
        icon="❌"
        title="Request Failed"
        message={editorStore.errorMessage}
      />
    </div>
  {:else if editorStore.response}
    <StatusBar response={editorStore.response} />

    <div class="tab-bar">
      {#each tabs as tab}
        <button
          class:active={editorStore.activeResponseTab === tab.value}
          onclick={() => (editorStore.activeResponseTab = tab.value)}
        >
          {tab.label}
        </button>
      {/each}
      <div class="tab-actions">
        {#if canCompare && oncompare}
          <button class="action-btn compare-btn" onclick={oncompare} title="Compare with pinned response">
            Diff
          </button>
        {/if}
        <button
          class="action-btn pin-btn"
          class:pinned={hasPinned}
          onclick={() => hasPinned ? editorStore.unpinResponse() : editorStore.pinResponse()}
          title={hasPinned ? "Unpin response" : "Pin this response for comparison"}
        >
          {hasPinned ? "Unpin" : "Pin"}
        </button>
        {#if ongeneratecode}
          <button class="action-btn code-btn" onclick={ongeneratecode} title="Generate Code">
            {"</>"}
          </button>
        {/if}
      </div>
    </div>

    <div class="tab-content">
      {#if editorStore.activeResponseTab === "body"}
        <ResponseBody response={editorStore.response} />
      {:else}
        <ResponseHeaders response={editorStore.response} />
      {/if}
    </div>
  {:else}
    <div class="empty">
      <EmptyState
        icon="📡"
        title="No Response"
        message="Send a request to see the response here."
      />
    </div>
  {/if}
</div>

<style>
  .response-view {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
    border-left: 1px solid var(--border-color);
  }
  .loading {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--sp-sm);
    height: 100%;
    color: var(--text-secondary);
    font-size: var(--fs-small);
  }
  .spinner {
    width: 16px;
    height: 16px;
    border: 2px solid var(--border-color);
    border-top-color: var(--color-info);
    border-radius: 50%;
    animation: spin 0.6s linear infinite;
  }
  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
  .error-state,
  .empty {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
  }
  .tab-actions {
    display: flex;
    align-items: center;
    gap: var(--sp-xs);
    margin-left: auto;
    padding-right: var(--sp-sm);
  }
  .action-btn {
    font-size: var(--fs-caption);
    padding: 2px var(--sp-sm);
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
  }
  .action-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }
  .pin-btn.pinned {
    color: var(--color-info);
    font-weight: 600;
  }
  .compare-btn {
    color: var(--color-warning, #f59e0b);
    font-weight: 500;
  }
  .code-btn {
    font-family: var(--font-mono, monospace);
  }
  .tab-content {
    flex: 1;
    overflow: hidden;
  }
</style>
