import { cancelRequest, executeScript, sendRequest } from '../commands'
import type { ScriptResult } from '../commands'
import { DEFAULT_REQUEST_TIMEOUT } from '../constants'
import {
  buildAuthConfig,
  buildRequestBody,
  createEmptyAssertionRule,
  createEmptyExtractionRule,
  createEmptyMultipartField,
  createEmptyPair,
  getBodyContent,
  getBodyType,
  getFormPairs,
  getGraphQLContent,
  getMultipartFields,
  isAssertionComplete,
} from '../type-helpers'
import type {
  ApiKeyLocation,
  AssertionRule,
  AuthType,
  BodyType,
  EditorTab,
  ExtractionRule,
  HttpMethod,
  KeyValuePair,
  MultipartField,
  OAuthGrantType,
  ResponseRecord,
  ResponseTab,
  SavedRequest,
} from '../types'
import { buildOAuth2ConfigFromFields, parseAuthConfig } from '../utils/auth-helpers'
import { applyExtractionRules } from '../utils/extraction'
import { findUnmatchedVariables } from '../utils/variables'
import { appStore } from './app.svelte'
import { environmentStore } from './environment.svelte'
import { historyStore } from './history.svelte'

class EditorStore {
  // Request fields
  requestId = $state<string | null>(null)
  name = $state('New Request')
  method = $state<HttpMethod>('GET')
  url = $state('')
  queryParams = $state<KeyValuePair[]>([createEmptyPair()])
  headers = $state<KeyValuePair[]>([createEmptyPair()])
  bodyType = $state<BodyType>('none')
  jsonBody = $state('')
  rawBody = $state('')
  formPairs = $state<KeyValuePair[]>([createEmptyPair()])
  multipartFields = $state<MultipartField[]>([createEmptyMultipartField()])
  graphqlQuery = $state('')
  graphqlVariables = $state('')
  extractionRules = $state<ExtractionRule[]>([createEmptyExtractionRule()])
  assertionRules = $state<AssertionRule[]>([createEmptyAssertionRule()])
  preRequestScript = $state('')
  testScript = $state('')

  // Auth fields
  authType = $state<AuthType>('none')
  bearerToken = $state('')
  basicUsername = $state('')
  basicPassword = $state('')
  apiKeyKey = $state('')
  apiKeyValue = $state('')
  apiKeyLocation = $state<ApiKeyLocation>('header')
  oauthGrantType = $state<OAuthGrantType>('client_credentials')
  oauthClientId = $state('')
  oauthClientSecret = $state('')
  oauthAuthUrl = $state('')
  oauthTokenUrl = $state('')
  oauthScopes = $state('')
  oauthAccessToken = $state('')
  oauthRefreshToken = $state('')
  oauthTokenExpiry = $state<string | null>(null)

  // UI state
  activeEditorTab = $state<EditorTab>('params')
  activeResponseTab = $state<ResponseTab>('body')
  response = $state<ResponseRecord | null>(null)
  pinnedResponse = $state<ResponseRecord | null>(null)
  isLoading = $state(false)
  errorMessage = $state<string | null>(null)
  isDirty = $state(false)
  timeoutSecs = $state(DEFAULT_REQUEST_TIMEOUT)
  followRedirects = $state(true)
  protocolMode = $state<'http' | 'ws'>('http')
  variableWarning = $state<string | null>(null)
  scriptResult = $state<ScriptResult | null>(null)

  get isUrlValid(): boolean {
    const u = this.url.trim()
    if (!u) return true
    if (u.includes('{{')) return true
    try {
      new URL(u)
      return true
    } catch {
      return u.startsWith('http://') || u.startsWith('https://')
    }
  }

  get canSend(): boolean {
    return this.url.trim().length > 0 && !this.isLoading
  }

  loadFrom(request: SavedRequest) {
    this.saveIfDirty()
    this.requestId = request.id
    this.name = request.name
    this.method = request.method
    this.url = request.url
    this.queryParams = request.queryParams.length ? [...request.queryParams] : [createEmptyPair()]
    this.headers = request.headers.length ? [...request.headers] : [createEmptyPair()]
    this.bodyType = getBodyType(request.body)
    this.jsonBody = getBodyContent(request.body)
    this.rawBody =
      typeof request.body === 'object' && 'rawText' in request.body ? request.body.rawText : ''
    this.formPairs = getFormPairs(request.body).length
      ? [...getFormPairs(request.body)]
      : [createEmptyPair()]
    this.multipartFields = getMultipartFields(request.body).length
      ? [...getMultipartFields(request.body)]
      : [createEmptyMultipartField()]
    const gql = getGraphQLContent(request.body)
    this.graphqlQuery = gql.query
    this.graphqlVariables = gql.variables
    this.extractionRules = request.responseExtractions?.length
      ? [...request.responseExtractions]
      : [createEmptyExtractionRule()]
    this.assertionRules = request.assertions?.length
      ? [...request.assertions]
      : [createEmptyAssertionRule()]
    this.preRequestScript = request.preRequestScript ?? ''
    this.testScript = request.testScript ?? ''
    this.applyAuthFields(parseAuthConfig(request.auth))
    this.timeoutSecs = request.timeoutSecs ?? DEFAULT_REQUEST_TIMEOUT
    this.response = null
    this.errorMessage = null
    this.isDirty = false
  }

  private applyAuthFields(fields: import('../utils/auth-helpers').AuthFields) {
    this.authType = fields.authType
    this.bearerToken = fields.bearerToken
    this.basicUsername = fields.basicUsername
    this.basicPassword = fields.basicPassword
    this.apiKeyKey = fields.apiKeyKey
    this.apiKeyValue = fields.apiKeyValue
    this.apiKeyLocation = fields.apiKeyLocation
    this.oauthGrantType = fields.oauthGrantType
    this.oauthClientId = fields.oauthClientId
    this.oauthClientSecret = fields.oauthClientSecret
    this.oauthAuthUrl = fields.oauthAuthUrl
    this.oauthTokenUrl = fields.oauthTokenUrl
    this.oauthScopes = fields.oauthScopes
    this.oauthAccessToken = fields.oauthAccessToken
    this.oauthRefreshToken = fields.oauthRefreshToken
    this.oauthTokenExpiry = fields.oauthTokenExpiry
  }

  private currentBody(filterEmpty = false) {
    const gql = { query: this.graphqlQuery, variables: this.graphqlVariables }
    const formPairs = filterEmpty ? this.formPairs.filter((p) => p.key || p.value) : this.formPairs
    const multipart = filterEmpty
      ? this.multipartFields.filter((f) => f.name)
      : this.multipartFields
    return buildRequestBody(this.bodyType, this.jsonBody, this.rawBody, formPairs, multipart, gql)
  }

  private currentAuth() {
    return buildAuthConfig(
      this.authType,
      { token: this.bearerToken },
      { username: this.basicUsername, password: this.basicPassword },
      { key: this.apiKeyKey, value: this.apiKeyValue, location: this.apiKeyLocation },
      buildOAuth2ConfigFromFields(this),
    )
  }

  toSavedRequest(): SavedRequest | null {
    if (!this.requestId) return null
    return {
      id: this.requestId,
      name: this.name,
      method: this.method,
      url: this.url,
      queryParams: this.queryParams.filter((p) => p.key || p.value),
      headers: this.headers.filter((p) => p.key || p.value),
      body: this.currentBody(true),
      auth: this.currentAuth(),
      sortOrder: 0,
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
      responseExtractions: this.extractionRules.filter((r) => r.variableName || r.jsonPath),
      assertions: this.assertionRules.filter(isAssertionComplete),
      timeoutSecs: this.timeoutSecs !== DEFAULT_REQUEST_TIMEOUT ? this.timeoutSecs : undefined,
      preRequestScript: this.preRequestScript || undefined,
      testScript: this.testScript || undefined,
    }
  }

  saveIfDirty() {
    if (!this.isDirty || !this.requestId) return
    const request = this.toSavedRequest()
    if (request) {
      appStore.updateRequest(request)
      this.isDirty = false
    }
  }

  markDirty() {
    this.isDirty = true
  }

  async send() {
    if (!this.canSend) return
    this.isLoading = true
    this.errorMessage = null
    this.response = null
    this.scriptResult = null

    // Warn about unmatched variables before sending
    const saved = this.toSavedRequest()
    if (saved) {
      const envVars = environmentStore.activeEnvironment?.variables ?? []
      const unmatched = findUnmatchedVariables(saved, envVars)
      if (unmatched.length > 0) {
        this.variableWarning = `Unresolved: ${unmatched.map((v) => `{{${v}}}`).join(', ')}`
      } else {
        this.variableWarning = null
      }
    }

    try {
      // Run pre-request script if present
      if (this.preRequestScript) {
        await this.runScript(this.preRequestScript, 'pre-request')
        const sr = this.scriptResult as ScriptResult | null
        if (sr?.error) {
          this.errorMessage = `Pre-request script error: ${sr.error}`
          return
        }
      }

      const result = await sendRequest({
        method: this.method,
        url: this.url,
        headers: this.headers,
        queryParams: this.queryParams,
        body: this.currentBody(),
        auth: this.currentAuth(),
        timeoutSecs: this.timeoutSecs,
        followRedirects: this.followRedirects,
        environment: environmentStore.activeEnvironment,
        proxyConfig: appStore.proxyConfig.enabled ? appStore.proxyConfig : undefined,
      })
      this.response = result
      this.applyExtractions(result)

      // Run test script if present
      if (this.testScript) {
        await this.runScript(this.testScript, 'test', result)
      }

      historyStore.addEntry({
        method: this.method,
        url: this.url,
        statusCode: result.statusCode,
        elapsedTime: result.elapsedTime,
        requestId: this.requestId ?? null,
        requestName: this.name || null,
        snapshot: this.toSavedRequest() ?? undefined,
      })
    } catch (e) {
      this.errorMessage = e instanceof Error ? e.message : String(e)
    } finally {
      this.isLoading = false
    }
  }

  private async runScript(
    script: string,
    type: 'pre-request' | 'test',
    response?: import('../types').ResponseRecord,
  ) {
    const envVars: Record<string, string> = {}
    for (const v of environmentStore.activeEnvironment?.variables ?? []) {
      if (v.isEnabled && v.key) envVars[v.key] = v.value
    }
    const scriptReq = {
      method: this.method,
      url: this.url,
      headers: Object.fromEntries(this.headers.filter((h) => h.key).map((h) => [h.key, h.value])),
      body: this.jsonBody || this.rawBody || '',
    }
    const scriptResp = response
      ? {
          status: response.statusCode,
          headers: Object.fromEntries(
            (response.headers ?? []).map((h) => [h.key.toLowerCase(), h.value]),
          ),
          body: response.bodyString ?? '',
          time: response.elapsedTime,
        }
      : undefined
    const result = await executeScript({
      script,
      scriptType: type,
      envVars,
      request: scriptReq,
      response: scriptResp,
    })
    this.scriptResult = result
    // Apply env updates from script
    for (const [key, value] of result.envUpdates) {
      environmentStore.setVariable(key, value)
    }
  }

  async cancel() {
    if (!this.isLoading) return
    try {
      await cancelRequest()
    } catch {
      // best-effort
    }
    this.isLoading = false
  }

  private applyExtractions(response: ResponseRecord) {
    applyExtractionRules(this.extractionRules, response)
  }

  /** Load editor state from a history entry that has no saved-request match. */
  loadFromHistoryFallback(entry: { method: HttpMethod; url: string; requestName?: string | null }) {
    this.requestId = crypto.randomUUID()
    this.name = entry.requestName || 'History Replay'
    this.method = entry.method
    this.url = entry.url
    this.queryParams = [createEmptyPair()]
    this.headers = [createEmptyPair()]
    this.bodyType = 'none'
    this.jsonBody = ''
    this.rawBody = ''
    this.formPairs = [createEmptyPair()]
    this.multipartFields = [createEmptyMultipartField()]
    this.graphqlQuery = ''
    this.graphqlVariables = ''
    this.extractionRules = [createEmptyExtractionRule()]
    this.assertionRules = [createEmptyAssertionRule()]
    this.preRequestScript = ''
    this.testScript = ''
    this.applyAuthFields(parseAuthConfig(undefined))
    this.response = null
    this.errorMessage = null
    this.isDirty = false
  }

  reset() {
    this.requestId = null
    this.name = 'New Request'
    this.method = 'GET'
    this.url = ''
    this.queryParams = [createEmptyPair()]
    this.headers = [createEmptyPair()]
    this.bodyType = 'none'
    this.jsonBody = ''
    this.rawBody = ''
    this.formPairs = [createEmptyPair()]
    this.multipartFields = [createEmptyMultipartField()]
    this.graphqlQuery = ''
    this.graphqlVariables = ''
    this.extractionRules = [createEmptyExtractionRule()]
    this.assertionRules = [createEmptyAssertionRule()]
    this.preRequestScript = ''
    this.testScript = ''
    this.applyAuthFields(parseAuthConfig(undefined))
    this.response = null
    this.pinnedResponse = null
    this.errorMessage = null
    this.variableWarning = null
    this.scriptResult = null
    this.isDirty = false
  }

  pinResponse() {
    if (this.response) {
      this.pinnedResponse = { ...this.response }
    }
  }

  unpinResponse() {
    this.pinnedResponse = null
  }
}

export const editorStore = new EditorStore()
