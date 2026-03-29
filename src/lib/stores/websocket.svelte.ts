import { wsConnect, wsDisconnect, wsSend } from '../commands'
import { createEmptyPair } from '../type-helpers'
import type { ConnectionStatus, KeyValuePair, WsEvent, WsMessage } from '../types'
import { handleError } from '../utils/errors'
import { environmentStore } from './environment.svelte'

const MAX_RECONNECT_ATTEMPTS = 5
const BASE_RECONNECT_DELAY = 1000 // 1s

class WebSocketStore {
  connectionId = $state<string | null>(null)
  url = $state('')
  headers = $state<KeyValuePair[]>([createEmptyPair()])
  status = $state<ConnectionStatus>('disconnected')
  messages = $state<WsMessage[]>([])
  messageInput = $state('')
  autoReconnect = $state(false)

  private unlistenFn: (() => void) | null = null
  private reconnectAttempts = 0
  private reconnectTimer: ReturnType<typeof setTimeout> | null = null
  private userDisconnected = false

  get isConnected(): boolean {
    return this.status === 'connected'
  }

  get canConnect(): boolean {
    return this.status === 'disconnected' && this.url.trim().length > 0
  }

  async connect() {
    if (!this.canConnect) return

    const id = crypto.randomUUID()
    this.connectionId = id
    this.status = 'connecting'
    this.messages = []
    this.userDisconnected = false
    this.reconnectAttempts = 0

    try {
      await this.startListening()
      const enabledHeaders = this.headers.filter((h) => h.isEnabled && h.key.trim())
      await wsConnect(
        id,
        this.url,
        enabledHeaders.length > 0 ? enabledHeaders : undefined,
        environmentStore.activeEnvironment ?? undefined,
      )
    } catch (e) {
      this.status = 'disconnected'
      this.connectionId = null
      handleError(e, 'ws.connect')
    }
  }

  async send() {
    if (!this.isConnected || !this.connectionId || !this.messageInput.trim()) return

    const content = this.messageInput.trim()
    try {
      await wsSend(this.connectionId, content)
      this.messages.push({
        id: crypto.randomUUID(),
        direction: 'sent',
        content,
        timestamp: new Date().toISOString(),
      })
      this.messageInput = ''
    } catch (e) {
      handleError(e, 'ws.send')
    }
  }

  async disconnect() {
    this.userDisconnected = true
    this.cancelReconnect()

    if (!this.connectionId) return

    try {
      await wsDisconnect(this.connectionId)
    } catch (e) {
      handleError(e, 'ws.disconnect', { silent: true })
    }
    this.cleanup()
  }

  handleEvent(event: WsEvent) {
    if (event.connection_id !== this.connectionId) return

    switch (event.event_type) {
      case 'connected':
        this.status = 'connected'
        this.reconnectAttempts = 0
        break
      case 'message':
        if (event.data) {
          this.messages.push({
            id: crypto.randomUUID(),
            direction: 'received',
            content: event.data,
            timestamp: new Date().toISOString(),
          })
        }
        break
      case 'disconnected':
        this.cleanup()
        this.tryReconnect()
        break
      case 'error':
        handleError(event.data ?? 'Unknown WebSocket error', 'ws.event')
        this.cleanup()
        this.tryReconnect()
        break
    }
  }

  private tryReconnect() {
    if (
      !this.autoReconnect ||
      this.userDisconnected ||
      this.reconnectAttempts >= MAX_RECONNECT_ATTEMPTS ||
      !this.url.trim()
    ) {
      return
    }

    this.reconnectAttempts++
    const delay = BASE_RECONNECT_DELAY * 2 ** (this.reconnectAttempts - 1)
    this.status = 'connecting'

    this.reconnectTimer = setTimeout(async () => {
      if (this.userDisconnected) return

      const id = crypto.randomUUID()
      this.connectionId = id

      try {
        await this.startListening()
        const enabledHeaders = this.headers.filter((h) => h.isEnabled && h.key.trim())
        await wsConnect(
          id,
          this.url,
          enabledHeaders.length > 0 ? enabledHeaders : undefined,
          environmentStore.activeEnvironment ?? undefined,
        )
      } catch {
        this.cleanup()
        this.tryReconnect()
      }
    }, delay)
  }

  private cancelReconnect() {
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer)
      this.reconnectTimer = null
    }
  }

  private async startListening() {
    try {
      const { listen } = await import('@tauri-apps/api/event')
      this.unlistenFn = await listen<WsEvent>('ws-event', (e) => {
        this.handleEvent(e.payload)
      })
    } catch {
      // Not in Tauri environment (dev/test) — no-op
    }
  }

  private cleanup() {
    this.status = 'disconnected'
    this.connectionId = null
    if (this.unlistenFn) {
      this.unlistenFn()
      this.unlistenFn = null
    }
  }
}

export const wsStore = new WebSocketStore()
