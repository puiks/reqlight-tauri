import { beforeEach, describe, expect, it, vi } from 'vite-plus/test'

vi.mock('../commands', () => ({
  sendRequest: vi.fn(),
}))

vi.mock('../utils/jsonpath', () => ({
  extractByPath: vi.fn(),
}))

vi.mock('./environment.svelte', () => ({
  environmentStore: {
    activeEnvironment: null,
    setVariable: vi.fn(),
  },
}))

vi.mock('./app.svelte', () => ({
  appStore: {
    proxyConfig: { enabled: false, proxyUrl: '', noProxy: '' },
  },
}))

import { runnerStore } from './runner.svelte'

describe('runnerStore', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    runnerStore.reset()
  })

  describe('initial state', () => {
    it('starts idle with empty results', () => {
      expect(runnerStore.status).toBe('idle')
      expect(runnerStore.results).toEqual([])
      expect(runnerStore.currentIndex).toBe(0)
      expect(runnerStore.totalRequests).toBe(0)
    })
  })

  describe('computed properties', () => {
    it('progress is 0 when no requests', () => {
      expect(runnerStore.progress).toBe(0)
    })

    it('passCount and failCount count correctly', () => {
      ;(runnerStore as any).results = [
        { passed: true, statusCode: 200, elapsedTime: 50 },
        { passed: false, statusCode: 500, elapsedTime: 30 },
        { passed: true, statusCode: 201, elapsedTime: 80 },
      ]
      expect(runnerStore.passCount).toBe(2)
      expect(runnerStore.failCount).toBe(1)
    })

    it('totalElapsed sums elapsed times', () => {
      ;(runnerStore as any).results = [
        { passed: true, elapsedTime: 100 },
        { passed: true, elapsedTime: 200 },
        { passed: false, elapsedTime: null },
      ]
      expect(runnerStore.totalElapsed).toBe(300)
    })
  })

  describe('stop', () => {
    it('sets shouldStop flag', () => {
      runnerStore.stop()
      expect((runnerStore as any).shouldStop).toBe(true)
    })
  })

  describe('reset', () => {
    it('resets all state to defaults', () => {
      ;(runnerStore as any).status = 'running'
      ;(runnerStore as any).results = [{ passed: true }]
      ;(runnerStore as any).currentIndex = 5
      ;(runnerStore as any).totalRequests = 10
      ;(runnerStore as any).collectionName = 'Test'
      ;(runnerStore as any).shouldStop = true

      runnerStore.reset()

      expect(runnerStore.status).toBe('idle')
      expect(runnerStore.results).toEqual([])
      expect(runnerStore.currentIndex).toBe(0)
      expect(runnerStore.totalRequests).toBe(0)
      expect(runnerStore.collectionName).toBe('')
    })
  })
})
