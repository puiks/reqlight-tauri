import { beforeEach, describe, expect, it, vi } from 'vite-plus/test'

vi.mock('../commands', () => ({
  wsConnect: vi.fn(),
  wsSend: vi.fn(),
  wsDisconnect: vi.fn(),
}))

vi.mock('../utils/errors', () => ({
  handleError: vi.fn(),
}))

import { wsStore } from './websocket.svelte'

describe('wsStore', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    // Reset store state
    wsStore.url = ''
    wsStore.messageInput = ''
    ;(wsStore as any).connectionId = null
    ;(wsStore as any).status = 'disconnected'
    ;(wsStore as any).messages = []
  })

  it('starts in disconnected state', () => {
    expect(wsStore.status).toBe('disconnected')
    expect(wsStore.isConnected).toBe(false)
    expect(wsStore.messages).toEqual([])
  })

  it('canConnect is false when URL is empty', () => {
    wsStore.url = ''
    expect(wsStore.canConnect).toBe(false)
  })

  it('canConnect is true when URL is set and disconnected', () => {
    wsStore.url = 'ws://localhost:8080'
    expect(wsStore.canConnect).toBe(true)
  })

  it('handleEvent adds received message', () => {
    // Simulate being connected
    ;(wsStore as any).connectionId = 'test-conn'
    ;(wsStore as any).status = 'connected'

    wsStore.handleEvent({
      connection_id: 'test-conn',
      event_type: 'message',
      data: 'hello from server',
    })

    expect(wsStore.messages).toHaveLength(1)
    expect(wsStore.messages[0].direction).toBe('received')
    expect(wsStore.messages[0].content).toBe('hello from server')
  })

  it('handleEvent ignores events for other connections', () => {
    ;(wsStore as any).connectionId = 'my-conn'

    wsStore.handleEvent({
      connection_id: 'other-conn',
      event_type: 'message',
      data: 'wrong connection',
    })

    expect(wsStore.messages).toHaveLength(0)
  })

  it('handleEvent sets disconnected on disconnect event', () => {
    ;(wsStore as any).connectionId = 'test-conn'
    ;(wsStore as any).status = 'connected'

    wsStore.handleEvent({
      connection_id: 'test-conn',
      event_type: 'disconnected',
      data: null,
    })

    expect(wsStore.status).toBe('disconnected')
    expect(wsStore.connectionId).toBeNull()
  })
})
