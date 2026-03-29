import type {
  AuthConfig,
  AuthType,
  ApiKeyLocation,
  BodyType,
  ExtractionRule,
  KeyValuePair,
  MultipartField,
  OAuth2Config,
  RequestBody,
} from './types'

// Helper: create an empty ExtractionRule
export function createEmptyExtractionRule(): ExtractionRule {
  return {
    id: crypto.randomUUID(),
    variableName: '',
    jsonPath: '',
    isEnabled: true,
  }
}

// Helper: create an empty KeyValuePair
export function createEmptyPair(): KeyValuePair {
  return {
    id: crypto.randomUUID(),
    key: '',
    value: '',
    isEnabled: true,
    isSecret: false,
  }
}

// Helper: create a default empty RequestBody
export function createEmptyBody(): RequestBody {
  return 'none'
}

// Helper: get body type from RequestBody
export function getBodyType(body: RequestBody): BodyType {
  if (body === 'none') return 'none'
  if (typeof body === 'object' && 'json' in body) return 'json'
  if (typeof body === 'object' && 'formData' in body) return 'formData'
  if (typeof body === 'object' && 'rawText' in body) return 'rawText'
  if (typeof body === 'object' && 'multipart' in body) return 'multipart'
  if (typeof body === 'object' && 'graphql' in body) return 'graphql'
  return 'none'
}

export function getGraphQLContent(body: RequestBody): { query: string; variables: string } {
  if (typeof body === 'object' && 'graphql' in body) return body.graphql
  return { query: '', variables: '' }
}

export function getMultipartFields(body: RequestBody): MultipartField[] {
  if (typeof body === 'object' && 'multipart' in body) return body.multipart
  return []
}

export function createEmptyMultipartField(): MultipartField {
  return {
    id: crypto.randomUUID(),
    name: '',
    value: '',
    isEnabled: true,
  }
}

// Helper: get body content from RequestBody
export function getBodyContent(body: RequestBody): string {
  if (typeof body === 'object' && 'json' in body) return body.json
  if (typeof body === 'object' && 'rawText' in body) return body.rawText
  return ''
}

// Helper: get form pairs from RequestBody
export function getFormPairs(body: RequestBody): KeyValuePair[] {
  if (typeof body === 'object' && 'formData' in body) return body.formData
  return []
}

// Helper: build RequestBody from parts
export function buildRequestBody(
  type: BodyType,
  jsonText: string,
  rawText: string,
  formPairs: KeyValuePair[],
  multipartFields?: MultipartField[],
  graphql?: { query: string; variables: string },
): RequestBody {
  switch (type) {
    case 'none':
      return 'none'
    case 'json':
      return { json: jsonText }
    case 'formData':
      return { formData: formPairs }
    case 'rawText':
      return { rawText: rawText }
    case 'multipart':
      return { multipart: multipartFields ?? [] }
    case 'graphql':
      return { graphql: graphql ?? { query: '', variables: '' } }
  }
}

// Auth helpers
export function createEmptyAuth(): AuthConfig {
  return 'none'
}

export function getAuthType(auth?: AuthConfig): AuthType {
  if (!auth || auth === 'none') return 'none'
  if (typeof auth === 'object' && 'bearerToken' in auth) return 'bearerToken'
  if (typeof auth === 'object' && 'basicAuth' in auth) return 'basicAuth'
  if (typeof auth === 'object' && 'apiKey' in auth) return 'apiKey'
  if (typeof auth === 'object' && 'oauth2' in auth) return 'oauth2'
  return 'none'
}

export function buildAuthConfig(
  type: AuthType,
  bearer: { token: string },
  basic: { username: string; password: string },
  apiKey: { key: string; value: string; location: ApiKeyLocation },
  oauth2?: OAuth2Config,
): AuthConfig {
  switch (type) {
    case 'none':
      return 'none'
    case 'bearerToken':
      return { bearerToken: { token: bearer.token } }
    case 'basicAuth':
      return { basicAuth: { username: basic.username, password: basic.password } }
    case 'apiKey':
      return { apiKey: { key: apiKey.key, value: apiKey.value, location: apiKey.location } }
    case 'oauth2':
      return { oauth2: oauth2 ?? createEmptyOAuth2Config() }
  }
}

export function createEmptyOAuth2Config(): OAuth2Config {
  return {
    grantType: 'client_credentials',
    clientId: '',
    clientSecret: '',
    authUrl: '',
    tokenUrl: '',
    scopes: '',
    accessToken: '',
    refreshToken: '',
    tokenExpiry: null,
  }
}
