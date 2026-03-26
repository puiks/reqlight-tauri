<script lang="ts">
  import { editorStore } from "../../lib/stores/editor.svelte";
  import StatusBar from "./StatusBar.svelte";
  import ResponseBody from "./ResponseBody.svelte";
  import ResponseHeaders from "./ResponseHeaders.svelte";
  import EmptyState from "../shared/EmptyState.svelte";
  import type { ResponseTab } from "../../lib/types";

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

    <div class="resp-tabs">
      {#each tabs as tab}
        <button
          class="tab"
          class:active={editorStore.activeResponseTab === tab.value}
          onclick={() => (editorStore.activeResponseTab = tab.value)}
        >
          {tab.label}
        </button>
      {/each}
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
  .resp-tabs {
    display: flex;
    border-bottom: 1px solid var(--border-color);
  }
  .tab {
    padding: var(--sp-sm) var(--sp-lg);
    font-size: var(--fs-small);
    font-weight: 500;
    color: var(--text-secondary);
    border-bottom: 2px solid transparent;
    border-radius: 0;
  }
  .tab:hover {
    color: var(--text-primary);
    background: transparent;
  }
  .tab.active {
    color: var(--color-info);
    border-bottom-color: var(--color-info);
    font-weight: 600;
  }
  .tab-content {
    flex: 1;
    overflow: hidden;
  }
</style>
