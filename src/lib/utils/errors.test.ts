import { beforeEach, describe, expect, it, vi } from 'vite-plus/test'

// Mock the toast store before importing errors module
vi.mock('../stores/toast.svelte', () => ({
  toastStore: { show: vi.fn() },
}))

import { toastStore } from '../stores/toast.svelte'
import { handleError, withErrorHandling } from './errors'

describe('handleError', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    vi.spyOn(console, 'error').mockImplementation(() => {})
  })

  it('logs Error instances with context', () => {
    handleError(new Error('boom'), 'test.context')
    expect(console.error).toHaveBeenCalledWith('[test.context]', 'boom')
  })

  it('logs string errors', () => {
    handleError('something failed', 'ctx')
    expect(console.error).toHaveBeenCalledWith('[ctx]', 'something failed')
  })

  it('shows toast by default', () => {
    handleError('msg', 'ctx')
    expect(toastStore.show).toHaveBeenCalledWith('Error: msg')
  })

  it('suppresses toast when silent option is set', () => {
    handleError('msg', 'ctx', { silent: true })
    expect(toastStore.show).not.toHaveBeenCalled()
  })
})

describe('withErrorHandling', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    vi.spyOn(console, 'error').mockImplementation(() => {})
  })

  it('returns result from wrapped function', async () => {
    const fn = async (x: unknown) => `result-${x}`
    const wrapped = withErrorHandling(fn, 'test')
    const result = await wrapped('a')
    expect(result).toBe('result-a')
  })

  it('catches errors and calls handleError', async () => {
    const fn = async () => {
      throw new Error('async failure')
    }
    const wrapped = withErrorHandling(fn, 'test.wrap')
    await wrapped()
    expect(console.error).toHaveBeenCalledWith('[test.wrap]', 'async failure')
    expect(toastStore.show).toHaveBeenCalled()
  })
})
