import { sendRequest } from '../commands'
import type {
  CollectionRunResult,
  CollectionRunStatus,
  RequestCollection,
  SavedRequest,
} from '../types'
import { applyExtractionRules } from '../utils/extraction'
import { appStore } from './app.svelte'
import { environmentStore } from './environment.svelte'

class RunnerStore {
  status = $state<CollectionRunStatus>('idle')
  results = $state<CollectionRunResult[]>([])
  currentIndex = $state(0)
  totalRequests = $state(0)
  collectionName = $state('')
  private shouldStop = false

  get progress(): number {
    return this.totalRequests > 0 ? this.currentIndex / this.totalRequests : 0
  }

  get passCount(): number {
    return this.results.filter((r) => r.passed).length
  }

  get failCount(): number {
    return this.results.filter((r) => !r.passed).length
  }

  get totalElapsed(): number {
    return this.results.reduce((sum, r) => sum + (r.elapsedTime ?? 0), 0)
  }

  async runCollection(collection: RequestCollection) {
    this.collectionName = collection.name
    this.results = []
    this.currentIndex = 0
    this.totalRequests = collection.requests.length
    this.shouldStop = false
    this.status = 'running'

    for (let i = 0; i < collection.requests.length; i++) {
      if (this.shouldStop) {
        this.status = 'stopped'
        return
      }

      this.currentIndex = i
      const request = collection.requests[i]
      const result = await this.executeRequest(request)
      this.results.push(result)
    }

    this.currentIndex = this.totalRequests
    this.status = 'completed'
  }

  private async executeRequest(request: SavedRequest): Promise<CollectionRunResult> {
    const base: Omit<
      CollectionRunResult,
      'statusCode' | 'elapsedTime' | 'passed' | 'errorMessage'
    > = {
      requestId: request.id,
      requestName: request.name,
      method: request.method,
      url: request.url,
    }

    try {
      const response = await sendRequest({
        method: request.method,
        url: request.url,
        headers: request.headers,
        queryParams: request.queryParams,
        body: request.body,
        auth: request.auth ?? 'none',
        timeoutSecs: request.timeoutSecs,
        environment: environmentStore.activeEnvironment,
        proxyConfig: appStore.proxyConfig.enabled ? appStore.proxyConfig : undefined,
      })

      // Apply extractions
      applyExtractionRules(request.responseExtractions ?? [], response)

      const passed = response.statusCode >= 200 && response.statusCode < 300
      return {
        ...base,
        statusCode: response.statusCode,
        elapsedTime: response.elapsedTime,
        passed,
      }
    } catch (e) {
      return {
        ...base,
        statusCode: null,
        elapsedTime: null,
        passed: false,
        errorMessage: e instanceof Error ? e.message : String(e),
      }
    }
  }

  stop() {
    this.shouldStop = true
  }

  reset() {
    this.status = 'idle'
    this.results = []
    this.currentIndex = 0
    this.totalRequests = 0
    this.collectionName = ''
    this.shouldStop = false
  }
}

export const runnerStore = new RunnerStore()
