import { afterEach, beforeEach, describe, expect, it, vi } from 'vite-plus/test'
import { initKeyboardShortcuts, registerShortcut } from './keyboard'

describe('keyboard shortcuts', () => {
  let cleanup: () => void

  beforeEach(() => {
    cleanup = initKeyboardShortcuts()
  })

  afterEach(() => {
    cleanup()
  })

  function fireKey(key: string, opts: Partial<KeyboardEventInit> = {}) {
    const event = new KeyboardEvent('keydown', {
      key,
      bubbles: true,
      ...opts,
    })
    // spy on preventDefault
    vi.spyOn(event, 'preventDefault')
    window.dispatchEvent(event)
    return event
  }

  it('triggers a registered shortcut on matching key', () => {
    const handler = vi.fn()
    registerShortcut({ key: 's', meta: true, handler })

    fireKey('s', { metaKey: true })
    expect(handler).toHaveBeenCalledOnce()
  })

  it('does not trigger when modifier does not match', () => {
    const handler = vi.fn()
    registerShortcut({ key: 's', meta: true, handler })

    fireKey('s') // no meta
    expect(handler).not.toHaveBeenCalled()
  })

  it('matches ctrlKey as meta substitute', () => {
    const handler = vi.fn()
    registerShortcut({ key: 'n', meta: true, handler })

    fireKey('n', { ctrlKey: true })
    expect(handler).toHaveBeenCalledOnce()
  })

  it('matches shift modifier', () => {
    const handler = vi.fn()
    registerShortcut({ key: 'z', meta: true, shift: true, handler })

    // Without shift — should not trigger
    fireKey('z', { metaKey: true })
    expect(handler).not.toHaveBeenCalled()

    // With shift — should trigger
    fireKey('z', { metaKey: true, shiftKey: true })
    expect(handler).toHaveBeenCalledOnce()
  })

  it('matches alt modifier', () => {
    const handler = vi.fn()
    registerShortcut({ key: 'p', alt: true, handler })

    fireKey('p') // no alt
    expect(handler).not.toHaveBeenCalled()

    fireKey('p', { altKey: true })
    expect(handler).toHaveBeenCalledOnce()
  })

  it('unregisters shortcut via returned cleanup function', () => {
    const handler = vi.fn()
    const unregister = registerShortcut({ key: 'k', meta: true, handler })

    fireKey('k', { metaKey: true })
    expect(handler).toHaveBeenCalledOnce()

    unregister()

    fireKey('k', { metaKey: true })
    expect(handler).toHaveBeenCalledOnce() // still 1, not 2
  })

  it('prevents default on matched shortcut', () => {
    registerShortcut({ key: 's', meta: true, handler: vi.fn() })

    const event = fireKey('s', { metaKey: true })
    expect(event.preventDefault).toHaveBeenCalled()
  })

  it('is case-insensitive for key matching', () => {
    const handler = vi.fn()
    registerShortcut({ key: 'q', meta: true, handler })

    fireKey('Q', { metaKey: true })
    expect(handler).toHaveBeenCalledOnce()
  })

  it('does not trigger after listener cleanup', () => {
    const handler = vi.fn()
    registerShortcut({ key: 's', meta: true, handler })

    cleanup() // remove the global listener

    fireKey('s', { metaKey: true })
    expect(handler).not.toHaveBeenCalled()
  })
})
