import { wsConnect, wsDisconnect, wsSend } from "../commands";
import type { ConnectionStatus, WsEvent, WsMessage } from "../types";
import { handleError } from "../utils/errors";

class WebSocketStore {
  connectionId = $state<string | null>(null);
  url = $state("");
  status = $state<ConnectionStatus>("disconnected");
  messages = $state<WsMessage[]>([]);
  messageInput = $state("");

  private unlistenFn: (() => void) | null = null;

  get isConnected(): boolean {
    return this.status === "connected";
  }

  get canConnect(): boolean {
    return this.status === "disconnected" && this.url.trim().length > 0;
  }

  async connect() {
    if (!this.canConnect) return;

    const id = crypto.randomUUID();
    this.connectionId = id;
    this.status = "connecting";
    this.messages = [];

    try {
      // Listen for Tauri events before connecting
      await this.startListening();
      await wsConnect(id, this.url);
    } catch (e) {
      this.status = "disconnected";
      this.connectionId = null;
      handleError(e, "ws.connect");
    }
  }

  async send() {
    if (!this.isConnected || !this.connectionId || !this.messageInput.trim())
      return;

    const content = this.messageInput.trim();
    try {
      await wsSend(this.connectionId, content);
      this.messages.push({
        id: crypto.randomUUID(),
        direction: "sent",
        content,
        timestamp: new Date().toISOString(),
      });
      this.messageInput = "";
    } catch (e) {
      handleError(e, "ws.send");
    }
  }

  async disconnect() {
    if (!this.connectionId) return;

    try {
      await wsDisconnect(this.connectionId);
    } catch (e) {
      // Connection may already be closed
      handleError(e, "ws.disconnect", { silent: true });
    }
    this.cleanup();
  }

  handleEvent(event: WsEvent) {
    if (event.connection_id !== this.connectionId) return;

    switch (event.event_type) {
      case "connected":
        this.status = "connected";
        break;
      case "message":
        if (event.data) {
          this.messages.push({
            id: crypto.randomUUID(),
            direction: "received",
            content: event.data,
            timestamp: new Date().toISOString(),
          });
        }
        break;
      case "disconnected":
        this.cleanup();
        break;
      case "error":
        handleError(event.data ?? "Unknown WebSocket error", "ws.event");
        this.cleanup();
        break;
    }
  }

  private async startListening() {
    // Dynamically import to avoid issues in non-Tauri environments
    try {
      const { listen } = await import("@tauri-apps/api/event");
      this.unlistenFn = await listen<WsEvent>("ws-event", (e) => {
        this.handleEvent(e.payload);
      });
    } catch {
      // Not in Tauri environment (dev/test) — no-op
    }
  }

  private cleanup() {
    this.status = "disconnected";
    this.connectionId = null;
    if (this.unlistenFn) {
      this.unlistenFn();
      this.unlistenFn = null;
    }
  }
}

export const wsStore = new WebSocketStore();
