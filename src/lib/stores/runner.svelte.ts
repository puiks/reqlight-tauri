import { sendRequest } from '../commands'
import type {
  CollectionRunResult,
  CollectionRunStatus,
  RequestCollection,
  ResponseRecord,
  SavedRequest,
} from '../types'
import { buildAuthConfig, getAuthType } from '../types'
import { extractByPath } from '../utils/jsonpath'
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
      this.results = [...this.results, result]
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
      const authType = getAuthType(request.auth)
      const auth = buildAuthConfig(
        authType,
        {
          token:
            request.auth && typeof request.auth === 'object' && 'bearerToken' in request.auth
              ? request.auth.bearerToken.token
              : '',
        },
        {
          username:
            request.auth && typeof request.auth === 'object' && 'basicAuth' in request.auth
              ? request.auth.basicAuth.username
              : '',
          password:
            request.auth && typeof request.auth === 'object' && 'basicAuth' in request.auth
              ? request.auth.basicAuth.password
              : '',
        },
        {
          key:
            request.auth && typeof request.auth === 'object' && 'apiKey' in request.auth
              ? request.auth.apiKey.key
              : '',
          value:
            request.auth && typeof request.auth === 'object' && 'apiKey' in request.auth
              ? request.auth.apiKey.value
              : '',
          location:
            request.auth && typeof request.auth === 'object' && 'apiKey' in request.auth
              ? request.auth.apiKey.location
              : 'header',
        },
      )

      const response = await sendRequest({
        method: request.method,
        url: request.url,
        headers: request.headers,
        queryParams: request.queryParams,
        body: request.body,
        auth,
        environment: environmentStore.activeEnvironment,
        proxyConfig: appStore.proxyConfig.enabled ? appStore.proxyConfig : undefined,
      })

      // Apply extractions
      this.applyExtractions(request, response)

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

  private applyExtractions(request: SavedRequest, response: ResponseRecord) {
    if (!response.bodyString || !response.isJson) return
    const rules = (request.responseExtractions ?? []).filter(
      (r) => r.isEnabled && r.variableName && r.jsonPath,
    )
    if (rules.length === 0) return

    let parsed: unknown
    try {
      parsed = JSON.parse(response.bodyString)
    } catch {
      return
    }

    for (const rule of rules) {
      const value = extractByPath(parsed, rule.jsonPath)
      if (value !== undefined) {
        environmentStore.setVariable(rule.variableName, value)
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
