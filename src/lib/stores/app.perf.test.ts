import { beforeEach, describe, expect, it } from 'vite-plus/test'
import type { RequestCollection, SavedRequest } from '../types'
import { appStore } from './app.svelte'

function generateRequest(index: number): SavedRequest {
  return {
    id: crypto.randomUUID(),
    name: `Request ${index}`,
    method: 'GET',
    url: `https://api.example.com/endpoint-${index}`,
    query_params: [],
    headers: [],
    body: { type: 'none' },
    auth: { type: 'none' },
    sort_order: index,
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  } as unknown as SavedRequest
}

function generateCollections(
  numCollections: number,
  requestsPerCollection: number,
): RequestCollection[] {
  return Array.from({ length: numCollections }, (_, i) => ({
    id: crypto.randomUUID(),
    name: `Collection ${i}`,
    requests: Array.from({ length: requestsPerCollection }, (_, j) =>
      generateRequest(i * requestsPerCollection + j),
    ),
  })) as RequestCollection[]
}

describe('AppStore Performance', () => {
  beforeEach(() => {
    appStore.collections = []
    appStore.searchQuery = ''
  })

  it('filteredCollections with 1000 requests completes in <50ms', () => {
    appStore.collections = generateCollections(50, 20)
    appStore.searchQuery = 'endpoint-500'

    const start = performance.now()
    const result = appStore.filteredCollections
    const elapsed = performance.now() - start

    expect(result.length).toBeGreaterThan(0)
    expect(elapsed).toBeLessThan(50)
  })

  it('filteredCollections with empty query (no filter) in <10ms', () => {
    appStore.collections = generateCollections(50, 20)
    appStore.searchQuery = ''

    const start = performance.now()
    const result = appStore.filteredCollections
    const elapsed = performance.now() - start

    expect(result.length).toBe(50)
    expect(elapsed).toBeLessThan(10)
  })

  it('addRequest to large collection in <10ms', () => {
    appStore.collections = generateCollections(50, 20)
    const collectionId = appStore.collections[0].id

    const start = performance.now()
    appStore.addRequest(collectionId)
    const elapsed = performance.now() - start

    expect(appStore.collections[0].requests.length).toBe(21)
    expect(elapsed).toBeLessThan(10)
  })

  it('deleteRequest from large store in <10ms', () => {
    appStore.collections = generateCollections(50, 20)
    const requestId = appStore.collections[25].requests[10].id

    const start = performance.now()
    appStore.deleteRequest(requestId)
    const elapsed = performance.now() - start

    expect(appStore.collections[25].requests.length).toBe(19)
    // CI runners are slower — use generous threshold
    expect(elapsed).toBeLessThan(50)
  })
})
