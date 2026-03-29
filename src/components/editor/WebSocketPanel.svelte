<script lang="ts">
  import { wsStore } from "../../lib/stores/websocket.svelte";
  import { createEmptyPair } from "../../lib/type-helpers";
  import WsMessageList from "./WsMessageList.svelte";

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      wsStore.send();
    }
  }

  let showHeaders = $state(false);

  function addHeader() {
    wsStore.headers = [...wsStore.headers, createEmptyPair()];
  }

  function removeHeader(id: string) {
    wsStore.headers = wsStore.headers.filter((h) => h.id !== id);
    if (wsStore.headers.length === 0) wsStore.headers = [createEmptyPair()];
  }
</script>

<div class="ws-panel">
  <div class="ws-url-bar">
    <span class="ws-badge">WS</span>
    <input
      type="text"
      class="ws-url-input"
      placeholder="ws://localhost:8080 or wss://..."
      bind:value={wsStore.url}
      disabled={wsStore.isConnected}
    />
    {#if wsStore.isConnected}
      <button class="disconnect-btn" onclick={() => wsStore.disconnect()}>
        Disconnect
      </button>
    {:else}
      <button
        class="connect-btn"
        onclick={() => wsStore.connect()}
        disabled={!wsStore.canConnect}
      >
        {wsStore.status === "connecting" ? "Connecting..." : "Connect"}
      </button>
    {/if}
  </div>

  <div class="ws-options">
    <button
      class="toggle-btn"
      class:active={showHeaders}
      onclick={() => (showHeaders = !showHeaders)}
    >
      Headers ({wsStore.headers.filter((h) => h.key.trim()).length})
    </button>
    <label class="reconnect-toggle">
      <input type="checkbox" bind:checked={wsStore.autoReconnect} />
      <span>Auto-reconnect</span>
    </label>
    <span class="status-indicator">
      <span class="status-dot" class:connected={wsStore.isConnected}></span>
      <span class="status-text">
        {wsStore.status === "connected"
          ? "Connected"
          : wsStore.status === "connecting"
            ? "Connecting..."
            : "Disconnected"}
      </span>
    </span>
  </div>

  {#if showHeaders}
    <div class="ws-headers">
      {#each wsStore.headers as header (header.id)}
        <div class="header-row">
          <input
            type="checkbox"
            bind:checked={header.isEnabled}
            disabled={wsStore.isConnected}
          />
          <input
            type="text"
            class="header-key"
            placeholder="Header name"
            bind:value={header.key}
            disabled={wsStore.isConnected}
          />
          <input
            type="text"
            class="header-value"
            placeholder="Value"
            bind:value={header.value}
            disabled={wsStore.isConnected}
          />
          <button
            class="remove-btn"
            onclick={() => removeHeader(header.id)}
            disabled={wsStore.isConnected}
          >✕</button>
        </div>
      {/each}
      <button class="add-btn" onclick={addHeader} disabled={wsStore.isConnected}>
        + Add Header
      </button>
    </div>
  {/if}

  {#if wsStore.isConnected}
    <div class="ws-input-bar">
      <input
        type="text"
        class="msg-input"
        placeholder="Type a message..."
        bind:value={wsStore.messageInput}
        onkeydown={handleKeydown}
      />
      <button
        class="send-msg-btn"
        onclick={() => wsStore.send()}
        disabled={!wsStore.messageInput.trim()}
      >
        Send
      </button>
    </div>
  {/if}

  <WsMessageList messages={wsStore.messages} isConnected={wsStore.isConnected} />
</div>

<style>
  .ws-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }
  .ws-url-bar {
    display: flex;
    gap: var(--sp-sm);
    padding: var(--sp-md);
    border-bottom: 1px solid var(--border-color);
    align-items: center;
  }
  .ws-badge {
    font-family: var(--font-mono);
    font-weight: 700;
    font-size: var(--fs-caption);
    color: var(--color-method-post);
    background: color-mix(in srgb, var(--color-method-post) 12%, transparent);
    padding: 2px 6px;
    border-radius: var(--radius-sm);
  }
  .ws-url-input {
    flex: 1;
    font-family: var(--font-mono);
    font-size: var(--fs-body);
  }
  .connect-btn {
    background: var(--color-success);
    color: white;
    font-weight: 600;
    padding: var(--sp-xs) var(--sp-lg);
    border-radius: var(--radius-sm);
  }
  .connect-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .disconnect-btn {
    background: var(--color-error);
    color: white;
    font-weight: 600;
    padding: var(--sp-xs) var(--sp-lg);
    border-radius: var(--radius-sm);
  }
  .ws-options {
    display: flex;
    align-items: center;
    gap: var(--sp-md);
    padding: var(--sp-xs) var(--sp-md);
    font-size: var(--fs-caption);
    color: var(--text-secondary);
    border-bottom: 1px solid var(--border-color);
  }
  .toggle-btn {
    font-size: var(--fs-caption);
    padding: 2px var(--sp-sm);
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
  }
  .toggle-btn.active {
    background: var(--bg-selected);
    color: var(--color-info);
    font-weight: 600;
  }
  .reconnect-toggle {
    display: flex;
    align-items: center;
    gap: var(--sp-xs);
    cursor: pointer;
    font-size: var(--fs-caption);
  }
  .reconnect-toggle input {
    margin: 0;
  }
  .status-indicator {
    display: flex;
    align-items: center;
    gap: var(--sp-xs);
    margin-left: auto;
  }
  .status-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--text-tertiary);
  }
  .status-dot.connected {
    background: var(--color-success);
  }
  .status-text {
    color: var(--text-tertiary);
  }
  .ws-headers {
    padding: var(--sp-sm) var(--sp-md);
    border-bottom: 1px solid var(--border-color);
    display: flex;
    flex-direction: column;
    gap: var(--sp-xs);
    max-height: 150px;
    overflow-y: auto;
  }
  .header-row {
    display: flex;
    align-items: center;
    gap: var(--sp-xs);
  }
  .header-key,
  .header-value {
    flex: 1;
    font-family: var(--font-mono);
    font-size: var(--fs-caption);
    padding: 2px var(--sp-xs);
  }
  .header-key {
    max-width: 40%;
  }
  .remove-btn {
    font-size: var(--fs-caption);
    color: var(--text-tertiary);
    padding: 0 4px;
  }
  .remove-btn:hover {
    color: var(--color-error);
  }
  .add-btn {
    font-size: var(--fs-caption);
    color: var(--color-info);
    align-self: flex-start;
    padding: 2px var(--sp-sm);
  }
  .ws-input-bar {
    display: flex;
    gap: var(--sp-sm);
    padding: var(--sp-sm) var(--sp-md);
    border-bottom: 1px solid var(--border-color);
  }
  .msg-input {
    flex: 1;
    font-family: var(--font-mono);
    font-size: var(--fs-small);
  }
  .send-msg-btn {
    background: var(--color-info);
    color: white;
    font-weight: 600;
    padding: var(--sp-xs) var(--sp-md);
    border-radius: var(--radius-sm);
    font-size: var(--fs-small);
  }
  .send-msg-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
