export interface KeyValuePair {
  id: string
  key: string
  value: string
  isEnabled: boolean
  isSecret: boolean
}

export type HttpMethod = 'GET' | 'POST' | 'PUT' | 'PATCH' | 'DELETE' | 'HEAD' | 'OPTIONS'

export interface MultipartField {
  id: string
  name: string
  value: string
  filePath?: string
  isEnabled: boolean
}

export type RequestBody =
  | 'none'
  | { json: string }
  | { formData: KeyValuePair[] }
  | { rawText: string }
  | { multipart: MultipartField[] }
  | { graphql: { query: string; variables: string } }

export type BodyType = 'none' | 'json' | 'formData' | 'rawText' | 'multipart' | 'graphql'

// Auth types
export type AuthType = 'none' | 'bearerToken' | 'basicAuth' | 'apiKey' | 'oauth2'
export type ApiKeyLocation = 'header' | 'query'
export type OAuthGrantType = 'authorization_code' | 'client_credentials'

export interface OAuth2Config {
  grantType: OAuthGrantType
  clientId: string
  clientSecret: string
  authUrl: string
  tokenUrl: string
  scopes: string
  accessToken: string
  refreshToken: string
  tokenExpiry: string | null
}

export type AuthConfig =
  | 'none'
  | { bearerToken: { token: string } }
  | { basicAuth: { username: string; password: string } }
  | { apiKey: { key: string; value: string; location: ApiKeyLocation } }
  | { oauth2: OAuth2Config }

export interface ExtractionRule {
  id: string
  variableName: string
  jsonPath: string
  isEnabled: boolean
}

export interface SavedRequest {
  id: string
  name: string
  method: HttpMethod
  url: string
  queryParams: KeyValuePair[]
  headers: KeyValuePair[]
  body: RequestBody
  auth?: AuthConfig
  sortOrder: number
  createdAt: string
  updatedAt: string
  responseExtractions?: ExtractionRule[]
  timeoutSecs?: number
}

export interface RequestCollection {
  id: string
  name: string
  requests: SavedRequest[]
  sortOrder: number
  createdAt: string
}

export interface RequestEnvironment {
  id: string
  name: string
  variables: KeyValuePair[]
}

export interface RequestHistoryEntry {
  id: string
  method: HttpMethod
  url: string
  statusCode: number | null
  timestamp: string
  elapsedTime: number | null
  /** ID of the saved request this was sent from (null if detached) */
  requestId?: string | null
  /** Name of the request at time of send */
  requestName?: string | null
  /** Full request snapshot for replay (frontend-only, not persisted to Rust) */
  snapshot?: SavedRequest
}

export interface HeaderPair {
  key: string
  value: string
}

export interface ResponseRecord {
  statusCode: number
  headers: HeaderPair[]
  bodyString: string | null
  elapsedTime: number
  bodySize: number
  isJson: boolean
  isTruncated: boolean
  contentType: string
}

export interface ProxyConfig {
  proxyUrl: string
  noProxy: string
  enabled: boolean
}

export interface AppState {
  collections: RequestCollection[]
  environments: RequestEnvironment[]
  activeEnvironmentId: string | null
  lastSelectedCollectionId: string | null
  lastSelectedRequestId: string | null
  history: RequestHistoryEntry[]
  proxyConfig?: ProxyConfig
}

export type CodegenLanguage = 'javascript-fetch' | 'javascript-axios' | 'python-requests' | 'curl'

export type EditorTab = 'params' | 'headers' | 'auth' | 'body' | 'extract'
export type ResponseTab = 'body' | 'headers'

export const HTTP_METHODS: HttpMethod[] = [
  'GET',
  'POST',
  'PUT',
  'PATCH',
  'DELETE',
  'HEAD',
  'OPTIONS',
]

export const METHOD_COLORS: Record<HttpMethod, string> = {
  GET: 'var(--color-method-get)',
  POST: 'var(--color-method-post)',
  PUT: 'var(--color-method-put)',
  PATCH: 'var(--color-method-patch)',
  DELETE: 'var(--color-method-delete)',
  HEAD: 'var(--color-method-head)',
  OPTIONS: 'var(--color-method-options)',
}

// Collection Runner types
export interface CollectionRunResult {
  requestId: string
  requestName: string
  method: HttpMethod
  url: string
  statusCode: number | null
  elapsedTime: number | null
  passed: boolean
  errorMessage?: string
}

export type CollectionRunStatus = 'idle' | 'running' | 'stopped' | 'completed'

// WebSocket types
export type ConnectionStatus = 'disconnected' | 'connecting' | 'connected'

export interface WsMessage {
  id: string
  direction: 'sent' | 'received'
  content: string
  timestamp: string
}

export interface WsEvent {
  connection_id: string
  event_type: 'message' | 'connected' | 'disconnected' | 'error'
  data: string | null
}
