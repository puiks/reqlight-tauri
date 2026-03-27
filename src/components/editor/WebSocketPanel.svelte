<script lang="ts">
  import { wsStore } from "../../lib/stores/websocket.svelte";

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      wsStore.send();
    }
  }

  function formatTime(iso: string): string {
    return new Date(iso).toLocaleTimeString();
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

  <div class="ws-status">
    <span class="status-dot" class:connected={wsStore.isConnected}></span>
    <span class="status-text">
      {wsStore.status === "connected"
        ? "Connected"
        : wsStore.status === "connecting"
          ? "Connecting..."
          : "Disconnected"}
    </span>
  </div>

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

  <div class="ws-messages">
    {#each wsStore.messages as msg (msg.id)}
      <div class="ws-msg" class:sent={msg.direction === "sent"}>
        <span class="msg-direction">
          {msg.direction === "sent" ? "↑" : "↓"}
        </span>
        <span class="msg-content">{msg.content}</span>
        <span class="msg-time">{formatTime(msg.timestamp)}</span>
      </div>
    {:else}
      <div class="ws-empty">
        {wsStore.isConnected
          ? "No messages yet. Send one above."
          : "Connect to a WebSocket server to start."}
      </div>
    {/each}
  </div>
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
  .ws-status {
    display: flex;
    align-items: center;
    gap: var(--sp-xs);
    padding: var(--sp-xs) var(--sp-md);
    font-size: var(--fs-caption);
    color: var(--text-tertiary);
    border-bottom: 1px solid var(--border-color);
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
  .ws-messages {
    flex: 1;
    overflow-y: auto;
    padding: var(--sp-sm);
  }
  .ws-msg {
    display: flex;
    gap: var(--sp-sm);
    padding: var(--sp-xs) var(--sp-sm);
    border-radius: var(--radius-sm);
    font-family: var(--font-mono);
    font-size: var(--fs-small);
    align-items: flex-start;
  }
  .ws-msg.sent {
    background: color-mix(in srgb, var(--color-info) 8%, transparent);
  }
  .msg-direction {
    font-weight: 700;
    flex-shrink: 0;
    width: 16px;
    text-align: center;
  }
  .ws-msg.sent .msg-direction {
    color: var(--color-info);
  }
  .ws-msg:not(.sent) .msg-direction {
    color: var(--color-success);
  }
  .msg-content {
    flex: 1;
    word-break: break-all;
    white-space: pre-wrap;
  }
  .msg-time {
    font-size: var(--fs-caption);
    color: var(--text-tertiary);
    flex-shrink: 0;
  }
  .ws-empty {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--text-tertiary);
    font-size: var(--fs-small);
  }
</style>
