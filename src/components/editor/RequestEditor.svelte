<script lang="ts">
  import { editorStore } from "../../lib/stores/editor.svelte";
  import URLBar from "./URLBar.svelte";
  import EditorTabs from "./EditorTabs.svelte";
  import KeyValueEditor from "./KeyValueEditor.svelte";
  import BodyEditor from "./BodyEditor.svelte";
  import AuthEditor from "./AuthEditor.svelte";
  import WebSocketPanel from "./WebSocketPanel.svelte";
  import EmptyState from "../shared/EmptyState.svelte";

  let { onimportcurl }: { onimportcurl: () => void } = $props();
</script>

<div class="editor">
  <div class="protocol-bar">
    <button
      class="proto-tab"
      class:active={editorStore.protocolMode === "http"}
      onclick={() => (editorStore.protocolMode = "http")}
    >
      HTTP
    </button>
    <button
      class="proto-tab"
      class:active={editorStore.protocolMode === "ws"}
      onclick={() => (editorStore.protocolMode = "ws")}
    >
      WebSocket
    </button>
  </div>

  {#if editorStore.protocolMode === "ws"}
    <WebSocketPanel />
  {:else if editorStore.requestId}
    <div class="name-bar">
      <input
        type="text"
        class="name-input"
        bind:value={editorStore.name}
        oninput={() => editorStore.markDirty()}
        placeholder="Request name"
      />
    </div>

    <URLBar {onimportcurl} />

    <EditorTabs />

    <div class="tab-content">
      {#if editorStore.activeEditorTab === "params"}
        <KeyValueEditor
          bind:pairs={editorStore.queryParams}
          onchange={() => editorStore.markDirty()}
        />
      {:else if editorStore.activeEditorTab === "headers"}
        <KeyValueEditor
          bind:pairs={editorStore.headers}
          onchange={() => editorStore.markDirty()}
        />
      {:else if editorStore.activeEditorTab === "auth"}
        <AuthEditor />
      {:else if editorStore.activeEditorTab === "body"}
        <BodyEditor />
      {/if}
    </div>
  {:else}
    <div class="empty">
      <EmptyState
        icon="⚡"
        title="Welcome to Reqlight"
        message="Select a request from the sidebar or create a new one to get started."
      />
    </div>
  {/if}
</div>

<style>
  .editor {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }
  .protocol-bar {
    display: flex;
    border-bottom: 1px solid var(--border-color);
    flex-shrink: 0;
  }
  .proto-tab {
    padding: var(--sp-xs) var(--sp-md);
    font-size: var(--fs-caption);
    font-weight: 600;
    color: var(--text-tertiary);
    border-bottom: 2px solid transparent;
    border-radius: 0;
    background: transparent;
  }
  .proto-tab:hover {
    color: var(--text-primary);
  }
  .proto-tab.active {
    color: var(--color-info);
    border-bottom-color: var(--color-info);
  }
  .name-bar {
    padding: var(--sp-sm) var(--sp-md);
    border-bottom: 1px solid var(--border-light);
  }
  .name-input {
    border: none;
    background: transparent;
    font-size: var(--fs-callout);
    font-weight: 600;
    width: 100%;
    padding: var(--sp-xs) 0;
  }
  .name-input:focus {
    box-shadow: none;
  }
  .tab-content {
    flex: 1;
    overflow: auto;
    padding: var(--sp-sm) var(--sp-md);
  }
  .empty {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
  }
</style>
