import { describe, expect, it, vi } from 'vite-plus/test'
import { ObservableStore } from './observable.svelte'

class TestStore extends ObservableStore {
  triggerNotify() {
    this.notify()
  }
}

describe('ObservableStore', () => {
  it('calls registered callback on notify', () => {
    const store = new TestStore()
    const cb = vi.fn()
    store.onStateChange(cb)
    store.triggerNotify()
    expect(cb).toHaveBeenCalledTimes(1)
  })

  it('does not throw if no callback registered', () => {
    const store = new TestStore()
    expect(() => store.triggerNotify()).not.toThrow()
  })

  it('replaces previous callback on re-registration', () => {
    const store = new TestStore()
    const cb1 = vi.fn()
    const cb2 = vi.fn()
    store.onStateChange(cb1)
    store.onStateChange(cb2)
    store.triggerNotify()
    expect(cb1).not.toHaveBeenCalled()
    expect(cb2).toHaveBeenCalledTimes(1)
  })
})
