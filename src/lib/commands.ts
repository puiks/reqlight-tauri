import type {
  AppState,
  AuthConfig,
  HttpMethod,
  KeyValuePair,
  ProxyConfig,
  RequestBody,
  RequestEnvironment,
  ResponseRecord,
  SavedRequest,
} from './types'

// Check if running inside Tauri webview
const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window

async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (isTauri) {
    const { invoke: tauriInvoke } = await import('@tauri-apps/api/core')
    return tauriInvoke<T>(cmd, args)
  }
  // Dev/E2E fallback — return mock data
  return devFallback(cmd) as T
}

function devFallback(cmd: string): unknown {
  switch (cmd) {
    case 'load_state':
      return {
        collections: [],
        environments: [],
        activeEnvironmentId: null,
        lastSelectedCollectionId: null,
        lastSelectedRequestId: null,
        history: [],
      }
    case 'save_state':
    case 'secret_set':
    case 'secret_delete':
    case 'cancel_request':
      return undefined
    case 'secret_get':
      return null
    default:
      return undefined
  }
}

// Persistence
export async function loadState(): Promise<AppState> {
  return invoke<AppState>('load_state')
}

export async function saveState(state: AppState): Promise<void> {
  return invoke('save_state', { state })
}

// HTTP
export async function sendRequest(params: {
  method: HttpMethod
  url: string
  headers: KeyValuePair[]
  queryParams: KeyValuePair[]
  body: RequestBody
  auth: AuthConfig
  timeoutSecs?: number
  followRedirects?: boolean
  environment?: RequestEnvironment
  proxyConfig?: ProxyConfig
}): Promise<ResponseRecord> {
  return invoke<ResponseRecord>('send_request', params)
}

export async function cancelRequest(): Promise<void> {
  return invoke('cancel_request')
}

// Keychain
export async function secretSet(key: string, value: string): Promise<void> {
  return invoke('secret_set', { key, value })
}

export async function secretGet(key: string): Promise<string | null> {
  return invoke<string | null>('secret_get', { key })
}

export async function secretDelete(key: string): Promise<void> {
  return invoke('secret_delete', { key })
}

// cURL
export async function parseCurl(curlString: string): Promise<SavedRequest> {
  return invoke<SavedRequest>('parse_curl', { curlString })
}

export async function exportCurl(
  request: SavedRequest,
  environment?: RequestEnvironment,
): Promise<string> {
  return invoke<string>('export_curl', { request, environment })
}

// Collection I/O
export async function importPostmanCollection(
  jsonStr: string,
): Promise<import('./types').RequestCollection> {
  return invoke('import_postman_collection', { jsonStr })
}

export async function exportPostmanCollection(
  collection: import('./types').RequestCollection,
): Promise<string> {
  return invoke('export_postman_collection', { collection })
}

export async function importPostmanEnvironment(
  jsonStr: string,
): Promise<import('./types').RequestEnvironment> {
  return invoke('import_postman_environment', { jsonStr })
}

export async function exportPostmanEnvironment(
  environment: import('./types').RequestEnvironment,
): Promise<string> {
  return invoke('export_postman_environment', { environment })
}

// OpenAPI Import
export async function importOpenapi(spec: string): Promise<import('./types').RequestCollection[]> {
  return invoke('import_openapi', { spec })
}

// HAR Import
export async function importHar(jsonStr: string): Promise<import('./types').RequestCollection> {
  return invoke('import_har', { jsonStr })
}

// Code Generation
export async function generateCode(
  request: SavedRequest,
  language: string,
  environment?: RequestEnvironment,
): Promise<string> {
  return invoke<string>('generate_code', { request, environment, language })
}

// OAuth 2.0
export interface OAuthTokenResult {
  accessToken: string
  refreshToken: string
  expiresIn: number | null
}

export async function oauthClientCredentials(
  tokenUrl: string,
  clientId: string,
  clientSecret: string,
  scopes: string,
): Promise<OAuthTokenResult> {
  return invoke('oauth_client_credentials', { tokenUrl, clientId, clientSecret, scopes })
}

export async function oauthAuthorizationCode(params: {
  authUrl: string
  tokenUrl: string
  clientId: string
  clientSecret: string
  scopes: string
}): Promise<OAuthTokenResult> {
  return invoke('oauth_authorization_code', { params })
}

export async function oauthRefreshToken(
  tokenUrl: string,
  refreshToken: string,
  clientId: string,
  clientSecret: string,
): Promise<OAuthTokenResult> {
  return invoke('oauth_refresh_token', { tokenUrl, refreshToken, clientId, clientSecret })
}

// Scripting
export interface ScriptRequestData {
  method: string
  url: string
  headers: Record<string, string>
  body: string
}

export interface ScriptResponseData {
  status: number
  headers: Record<string, string>
  body: string
  time: number
}

export interface ScriptTestResult {
  name: string
  passed: boolean
  message: string | null
}

export interface ScriptResult {
  envUpdates: [string, string][]
  testResults: ScriptTestResult[]
  consoleOutput: string[]
  error: string | null
}

export async function executeScript(params: {
  script: string
  scriptType: 'pre-request' | 'test'
  envVars: Record<string, string>
  request: ScriptRequestData
  response?: ScriptResponseData
}): Promise<ScriptResult> {
  return invoke<ScriptResult>('execute_script', params)
}

// WebSocket
export async function wsConnect(
  connectionId: string,
  url: string,
  headers?: KeyValuePair[],
  environment?: RequestEnvironment,
): Promise<void> {
  return invoke('ws_connect', { connectionId, url, headers, environment })
}

export async function wsSend(connectionId: string, message: string): Promise<void> {
  return invoke('ws_send', { connectionId, message })
}

export async function wsDisconnect(connectionId: string): Promise<void> {
  return invoke('ws_disconnect', { connectionId })
}
