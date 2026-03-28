import { beforeEach, describe, expect, it, vi } from 'vite-plus/test'

vi.mock('../commands', () => ({
  sendRequest: vi.fn(),
  cancelRequest: vi.fn(),
}))

vi.mock('../utils/errors', () => ({
  handleError: vi.fn(),
}))

vi.mock('./app.svelte', () => ({
  appStore: {
    updateRequest: vi.fn(),
    proxyConfig: { enabled: false, proxyUrl: '', noProxy: '' },
  },
}))

vi.mock('./environment.svelte', () => ({
  environmentStore: {
    activeEnvironment: null,
    setVariable: vi.fn(),
  },
}))

vi.mock('./history.svelte', () => ({
  historyStore: {
    addEntry: vi.fn(),
  },
}))

import type { SavedRequest } from '../types'
import { createEmptyAuth, createEmptyPair } from '../types'
import { editorStore } from './editor.svelte'

function makeSavedRequest(overrides: Partial<SavedRequest> = {}): SavedRequest {
  return {
    id: 'req-1',
    name: 'Test Request',
    method: 'GET',
    url: 'https://example.com',
    queryParams: [],
    headers: [],
    body: 'none',
    auth: createEmptyAuth(),
    sortOrder: 0,
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
    responseExtractions: [],
    ...overrides,
  }
}

describe('editorStore', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    editorStore.reset()
  })

  describe('initial state', () => {
    it('starts with default values', () => {
      expect(editorStore.requestId).toBeNull()
      expect(editorStore.name).toBe('New Request')
      expect(editorStore.method).toBe('GET')
      expect(editorStore.url).toBe('')
      expect(editorStore.isLoading).toBe(false)
      expect(editorStore.response).toBeNull()
    })

    it('canSend is false when URL is empty', () => {
      expect(editorStore.canSend).toBe(false)
    })
  })

  describe('loadFrom', () => {
    it('loads request fields from SavedRequest', () => {
      const req = makeSavedRequest({
        method: 'POST',
        url: 'https://api.test.com',
        name: 'My API',
      })

      editorStore.loadFrom(req)

      expect(editorStore.requestId).toBe('req-1')
      expect(editorStore.name).toBe('My API')
      expect(editorStore.method).toBe('POST')
      expect(editorStore.url).toBe('https://api.test.com')
      expect(editorStore.isDirty).toBe(false)
    })

    it('loads bearer token auth', () => {
      const req = makeSavedRequest({
        auth: { bearerToken: { token: 'my-token' } },
      })

      editorStore.loadFrom(req)

      expect(editorStore.authType).toBe('bearerToken')
      expect(editorStore.bearerToken).toBe('my-token')
    })

    it('loads basic auth', () => {
      const req = makeSavedRequest({
        auth: { basicAuth: { username: 'user', password: 'pass' } },
      })

      editorStore.loadFrom(req)

      expect(editorStore.authType).toBe('basicAuth')
      expect(editorStore.basicUsername).toBe('user')
      expect(editorStore.basicPassword).toBe('pass')
    })

    it('clears response and error on load', () => {
      editorStore.errorMessage = 'old error'
      const req = makeSavedRequest()
      editorStore.loadFrom(req)
      expect(editorStore.response).toBeNull()
      expect(editorStore.errorMessage).toBeNull()
    })
  })

  describe('toSavedRequest', () => {
    it('returns null when no requestId', () => {
      expect(editorStore.toSavedRequest()).toBeNull()
    })

    it('builds SavedRequest from current state', () => {
      editorStore.requestId = 'req-1'
      editorStore.name = 'My Request'
      editorStore.method = 'POST'
      editorStore.url = 'https://api.com'

      const result = editorStore.toSavedRequest()
      expect(result).not.toBeNull()
      expect(result!.id).toBe('req-1')
      expect(result!.method).toBe('POST')
      expect(result!.url).toBe('https://api.com')
    })

    it('filters empty query params and headers', () => {
      editorStore.requestId = 'req-1'
      editorStore.queryParams = [
        { ...createEmptyPair(), key: 'page', value: '1' },
        createEmptyPair(), // empty, should be filtered
      ]
      editorStore.headers = [createEmptyPair()] // all empty

      const result = editorStore.toSavedRequest()!
      expect(result.queryParams).toHaveLength(1)
      expect(result.queryParams[0].key).toBe('page')
      expect(result.headers).toHaveLength(0)
    })
  })

  describe('isUrlValid', () => {
    it('accepts empty URL', () => {
      editorStore.url = ''
      expect(editorStore.isUrlValid).toBe(true)
    })

    it('accepts valid URLs', () => {
      editorStore.url = 'https://example.com'
      expect(editorStore.isUrlValid).toBe(true)
    })

    it('accepts URLs with template variables', () => {
      editorStore.url = '{{baseUrl}}/api/users'
      expect(editorStore.isUrlValid).toBe(true)
    })

    it('accepts http:// prefix even without valid URL', () => {
      editorStore.url = 'http://'
      expect(editorStore.isUrlValid).toBe(true)
    })
  })

  describe('pinResponse / unpinResponse', () => {
    it('pins current response', () => {
      editorStore.response = {
        statusCode: 200,
        bodyString: 'ok',
        elapsedTime: 100,
        headers: [],
        isJson: false,
        isTruncated: false,
        bodySize: 2,
        contentType: '',
      }

      editorStore.pinResponse()
      expect(editorStore.pinnedResponse).not.toBeNull()
      expect(editorStore.pinnedResponse!.statusCode).toBe(200)
    })

    it('unpins response', () => {
      editorStore.pinnedResponse = {} as any
      editorStore.unpinResponse()
      expect(editorStore.pinnedResponse).toBeNull()
    })
  })

  describe('reset', () => {
    it('resets all fields to defaults', () => {
      editorStore.requestId = 'req-1'
      editorStore.name = 'Modified'
      editorStore.method = 'POST'
      editorStore.url = 'https://test.com'
      editorStore.isDirty = true
      editorStore.authType = 'bearerToken'
      editorStore.bearerToken = 'token'

      editorStore.reset()

      expect(editorStore.requestId).toBeNull()
      expect(editorStore.name).toBe('New Request')
      expect(editorStore.method).toBe('GET')
      expect(editorStore.url).toBe('')
      expect(editorStore.isDirty).toBe(false)
      expect(editorStore.authType).toBe('none')
      expect(editorStore.bearerToken).toBe('')
    })
  })

  describe('markDirty', () => {
    it('sets isDirty to true', () => {
      expect(editorStore.isDirty).toBe(false)
      editorStore.markDirty()
      expect(editorStore.isDirty).toBe(true)
    })
  })
})
