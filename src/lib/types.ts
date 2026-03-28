export interface KeyValuePair {
  id: string;
  key: string;
  value: string;
  isEnabled: boolean;
  isSecret: boolean;
}

export type HttpMethod = "GET" | "POST" | "PUT" | "PATCH" | "DELETE" | "HEAD" | "OPTIONS";

export interface MultipartField {
  id: string;
  name: string;
  value: string;
  filePath?: string;
  isEnabled: boolean;
}

export type RequestBody =
  | { none: Record<string, never> }
  | { json: { _0: string } }
  | { formData: { _0: KeyValuePair[] } }
  | { rawText: { _0: string } }
  | { multipart: { _0: MultipartField[] } }
  | { graphql: { query: string; variables: string } };

export type BodyType = "none" | "json" | "formData" | "rawText" | "multipart" | "graphql";

// Auth types
export type AuthType = "none" | "bearerToken" | "basicAuth" | "apiKey" | "oauth2";
export type ApiKeyLocation = "header" | "query";
export type OAuthGrantType = "authorization_code" | "client_credentials";

export interface OAuth2Config {
  grantType: OAuthGrantType;
  clientId: string;
  clientSecret: string;
  authUrl: string;
  tokenUrl: string;
  scopes: string;
  accessToken: string;
  refreshToken: string;
  tokenExpiry: string | null;
}

export type AuthConfig =
  | { none: Record<string, never> }
  | { bearerToken: { _0: { token: string } } }
  | { basicAuth: { _0: { username: string; password: string } } }
  | { apiKey: { _0: { key: string; value: string; location: ApiKeyLocation } } }
  | { oauth2: OAuth2Config };

export interface ExtractionRule {
  id: string;
  variableName: string;
  jsonPath: string;
  isEnabled: boolean;
}

export interface SavedRequest {
  id: string;
  name: string;
  method: HttpMethod;
  url: string;
  queryParams: KeyValuePair[];
  headers: KeyValuePair[];
  body: RequestBody;
  auth?: AuthConfig;
  sortOrder: number;
  createdAt: string;
  updatedAt: string;
  responseExtractions?: ExtractionRule[];
}

export interface RequestCollection {
  id: string;
  name: string;
  requests: SavedRequest[];
  sortOrder: number;
  createdAt: string;
}

export interface RequestEnvironment {
  id: string;
  name: string;
  variables: KeyValuePair[];
}

export interface RequestHistoryEntry {
  id: string;
  method: HttpMethod;
  url: string;
  statusCode: number | null;
  timestamp: string;
  elapsedTime: number | null;
  /** ID of the saved request this was sent from (null if detached) */
  requestId?: string | null;
  /** Name of the request at time of send */
  requestName?: string | null;
  /** Full request snapshot for replay (frontend-only, not persisted to Rust) */
  snapshot?: SavedRequest;
}

export interface HeaderPair {
  key: string;
  value: string;
}

export interface ResponseRecord {
  statusCode: number;
  headers: HeaderPair[];
  bodyString: string | null;
  elapsedTime: number;
  bodySize: number;
  isJson: boolean;
  isTruncated: boolean;
  contentType: string;
}

export interface ProxyConfig {
  proxyUrl: string;
  noProxy: string;
  enabled: boolean;
}

export interface AppState {
  collections: RequestCollection[];
  environments: RequestEnvironment[];
  activeEnvironmentId: string | null;
  lastSelectedCollectionId: string | null;
  lastSelectedRequestId: string | null;
  history: RequestHistoryEntry[];
  proxyConfig?: ProxyConfig;
}

export type CodegenLanguage = "javascript-fetch" | "javascript-axios" | "python-requests" | "curl";

export type EditorTab = "params" | "headers" | "auth" | "body" | "extract";
export type ResponseTab = "body" | "headers";

export const HTTP_METHODS: HttpMethod[] = [
  "GET",
  "POST",
  "PUT",
  "PATCH",
  "DELETE",
  "HEAD",
  "OPTIONS",
];

export const METHOD_COLORS: Record<HttpMethod, string> = {
  GET: "var(--color-method-get)",
  POST: "var(--color-method-post)",
  PUT: "var(--color-method-put)",
  PATCH: "var(--color-method-patch)",
  DELETE: "var(--color-method-delete)",
  HEAD: "var(--color-method-head)",
  OPTIONS: "var(--color-method-options)",
};

// Helper: create an empty ExtractionRule
export function createEmptyExtractionRule(): ExtractionRule {
  return {
    id: crypto.randomUUID(),
    variableName: "",
    jsonPath: "",
    isEnabled: true,
  };
}

// Helper: create an empty KeyValuePair
export function createEmptyPair(): KeyValuePair {
  return {
    id: crypto.randomUUID(),
    key: "",
    value: "",
    isEnabled: true,
    isSecret: false,
  };
}

// Helper: create a default empty RequestBody
export function createEmptyBody(): RequestBody {
  return { none: {} };
}

// Helper: get body type from RequestBody
export function getBodyType(body: RequestBody): BodyType {
  if ("none" in body) return "none";
  if ("json" in body) return "json";
  if ("formData" in body) return "formData";
  if ("rawText" in body) return "rawText";
  if ("multipart" in body) return "multipart";
  if ("graphql" in body) return "graphql";
  return "none";
}

export function getGraphQLContent(body: RequestBody): { query: string; variables: string } {
  if ("graphql" in body) return body.graphql;
  return { query: "", variables: "" };
}

export function getMultipartFields(body: RequestBody): MultipartField[] {
  if ("multipart" in body) return body.multipart._0;
  return [];
}

export function createEmptyMultipartField(): MultipartField {
  return {
    id: crypto.randomUUID(),
    name: "",
    value: "",
    isEnabled: true,
  };
}

// Helper: get body content from RequestBody
export function getBodyContent(body: RequestBody): string {
  if ("json" in body) return body.json._0;
  if ("rawText" in body) return body.rawText._0;
  return "";
}

// Helper: get form pairs from RequestBody
export function getFormPairs(body: RequestBody): KeyValuePair[] {
  if ("formData" in body) return body.formData._0;
  return [];
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
    case "none":
      return { none: {} };
    case "json":
      return { json: { _0: jsonText } };
    case "formData":
      return { formData: { _0: formPairs } };
    case "rawText":
      return { rawText: { _0: rawText } };
    case "multipart":
      return { multipart: { _0: multipartFields ?? [] } };
    case "graphql":
      return { graphql: graphql ?? { query: "", variables: "" } };
  }
}

// Auth helpers
export function createEmptyAuth(): AuthConfig {
  return { none: {} };
}

export function getAuthType(auth?: AuthConfig): AuthType {
  if (!auth) return "none";
  if ("none" in auth) return "none";
  if ("bearerToken" in auth) return "bearerToken";
  if ("basicAuth" in auth) return "basicAuth";
  if ("apiKey" in auth) return "apiKey";
  if ("oauth2" in auth) return "oauth2";
  return "none";
}

export function buildAuthConfig(
  type: AuthType,
  bearer: { token: string },
  basic: { username: string; password: string },
  apiKey: { key: string; value: string; location: ApiKeyLocation },
  oauth2?: OAuth2Config,
): AuthConfig {
  switch (type) {
    case "none":
      return { none: {} };
    case "bearerToken":
      return { bearerToken: { _0: { token: bearer.token } } };
    case "basicAuth":
      return { basicAuth: { _0: { username: basic.username, password: basic.password } } };
    case "apiKey":
      return { apiKey: { _0: { key: apiKey.key, value: apiKey.value, location: apiKey.location } } };
    case "oauth2":
      return { oauth2: oauth2 ?? createEmptyOAuth2Config() };
  }
}

export function createEmptyOAuth2Config(): OAuth2Config {
  return {
    grantType: "client_credentials",
    clientId: "",
    clientSecret: "",
    authUrl: "",
    tokenUrl: "",
    scopes: "",
    accessToken: "",
    refreshToken: "",
    tokenExpiry: null,
  };
}

// Collection Runner types
export interface CollectionRunResult {
  requestId: string;
  requestName: string;
  method: HttpMethod;
  url: string;
  statusCode: number | null;
  elapsedTime: number | null;
  passed: boolean;
  errorMessage?: string;
}

export type CollectionRunStatus = "idle" | "running" | "stopped" | "completed";

// WebSocket types
export type ConnectionStatus = "disconnected" | "connecting" | "connected";

export interface WsMessage {
  id: string;
  direction: "sent" | "received";
  content: string;
  timestamp: string;
}

export interface WsEvent {
  connection_id: string;
  event_type: "message" | "connected" | "disconnected" | "error";
  data: string | null;
}
