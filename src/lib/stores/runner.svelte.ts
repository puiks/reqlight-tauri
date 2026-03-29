import { sendRequest } from '../commands'
import type {
  CollectionRunResult,
  CollectionRunStatus,
  RequestCollection,
  SavedRequest,
} from '../types'
import { evaluateAssertions } from '../utils/assertion'
import { applyExtractionRules } from '../utils/extraction'
import { findUnmatchedVariables } from '../utils/variables'
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
    // Detect unmatched variables before sending
    const envVars = environmentStore.activeEnvironment?.variables ?? []
    const unmatchedVariables = findUnmatchedVariables(request, envVars)

    const base: Omit<
      CollectionRunResult,
      'statusCode' | 'elapsedTime' | 'passed' | 'errorMessage'
    > = {
      requestId: request.id,
      requestName: request.name,
      method: request.method,
      url: request.url,
      unmatchedVariables: unmatchedVariables.length > 0 ? unmatchedVariables : undefined,
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

      // Evaluate assertions
      const assertions = request.assertions?.filter((a) => a.isEnabled) ?? []
      const assertionResults =
        assertions.length > 0 ? evaluateAssertions(assertions, response) : undefined

      // Pass/fail: if assertions exist, all must pass; otherwise fall back to 2xx check
      const passed = assertionResults
        ? assertionResults.every((r) => r.passed)
        : response.statusCode >= 200 && response.statusCode < 300

      // Store truncated body for debugging (first 2KB)
      const responseBody = response.bodyString ? response.bodyString.slice(0, 2048) : null

      return {
        ...base,
        statusCode: response.statusCode,
        elapsedTime: response.elapsedTime,
        passed,
        assertionResults,
        responseBody,
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
