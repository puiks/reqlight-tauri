<script lang="ts">
  import type { WsMessage } from "../../lib/types";

  let { messages, isConnected }: { messages: WsMessage[]; isConnected: boolean } = $props();

  function formatTime(iso: string): string {
    return new Date(iso).toLocaleTimeString();
  }
</script>

<div class="ws-messages">
  {#each messages as msg (msg.id)}
    <div class="ws-msg" class:sent={msg.direction === "sent"}>
      <span class="msg-direction">
        {msg.direction === "sent" ? "↑" : "↓"}
      </span>
      <span class="msg-content">{msg.content}</span>
      <span class="msg-time">{formatTime(msg.timestamp)}</span>
    </div>
  {:else}
    <div class="ws-empty">
      {isConnected
        ? "No messages yet. Send one above."
        : "Connect to a WebSocket server to start."}
    </div>
  {/each}
</div>

<style>
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
