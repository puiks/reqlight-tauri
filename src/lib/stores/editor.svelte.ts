import { sendRequest, cancelRequest } from "../commands";
import {
  buildAuthConfig,
  buildRequestBody,
  createEmptyAuth,
  createEmptyExtractionRule,
  createEmptyMultipartField,
  createEmptyOAuth2Config,
  createEmptyPair,
  getAuthType,
  getBodyContent,
  getBodyType,
  getFormPairs,
  getGraphQLContent,
  getMultipartFields,
  type ApiKeyLocation,
  type AuthType,
  type BodyType,
  type EditorTab,
  type ExtractionRule,
  type HttpMethod,
  type KeyValuePair,
  type MultipartField,
  type OAuth2Config,
  type OAuthGrantType,
  type ResponseRecord,
  type ResponseTab,
  type SavedRequest,
} from "../types";
import { extractByPath } from "../utils/jsonpath";
import { DEFAULT_REQUEST_TIMEOUT } from "../constants";
import { appStore } from "./app.svelte";
import { environmentStore } from "./environment.svelte";
import { historyStore } from "./history.svelte";

class EditorStore {
  // Request fields
  requestId = $state<string | null>(null);
  name = $state("New Request");
  method = $state<HttpMethod>("GET");
  url = $state("");
  queryParams = $state<KeyValuePair[]>([createEmptyPair()]);
  headers = $state<KeyValuePair[]>([createEmptyPair()]);
  bodyType = $state<BodyType>("none");
  jsonBody = $state("");
  rawBody = $state("");
  formPairs = $state<KeyValuePair[]>([createEmptyPair()]);
  multipartFields = $state<MultipartField[]>([createEmptyMultipartField()]);
  graphqlQuery = $state("");
  graphqlVariables = $state("");
  extractionRules = $state<ExtractionRule[]>([createEmptyExtractionRule()]);

  // Auth fields
  authType = $state<AuthType>("none");
  bearerToken = $state("");
  basicUsername = $state("");
  basicPassword = $state("");
  apiKeyKey = $state("");
  apiKeyValue = $state("");
  apiKeyLocation = $state<ApiKeyLocation>("header");
  oauthGrantType = $state<OAuthGrantType>("client_credentials");
  oauthClientId = $state("");
  oauthClientSecret = $state("");
  oauthAuthUrl = $state("");
  oauthTokenUrl = $state("");
  oauthScopes = $state("");
  oauthAccessToken = $state("");
  oauthRefreshToken = $state("");
  oauthTokenExpiry = $state<string | null>(null);

  // UI state
  activeEditorTab = $state<EditorTab>("params");
  activeResponseTab = $state<ResponseTab>("body");
  response = $state<ResponseRecord | null>(null);
  pinnedResponse = $state<ResponseRecord | null>(null);
  isLoading = $state(false);
  errorMessage = $state<string | null>(null);
  isDirty = $state(false);
  timeoutSecs = $state(DEFAULT_REQUEST_TIMEOUT);
  followRedirects = $state(true);
  protocolMode = $state<"http" | "ws">("http");

  // Derived
  get isUrlValid(): boolean {
    const u = this.url.trim();
    if (!u) return true; // empty is ok (just can't send)
    if (u.includes("{{")) return true; // allow variables
    try {
      new URL(u);
      return true;
    } catch {
      return u.startsWith("http://") || u.startsWith("https://");
    }
  }

  get canSend(): boolean {
    return this.url.trim().length > 0 && !this.isLoading;
  }

  // Load from a saved request
  loadFrom(request: SavedRequest) {
    this.saveIfDirty();
    this.requestId = request.id;
    this.name = request.name;
    this.method = request.method;
    this.url = request.url;
    this.queryParams = request.queryParams.length
      ? [...request.queryParams]
      : [createEmptyPair()];
    this.headers = request.headers.length
      ? [...request.headers]
      : [createEmptyPair()];
    this.bodyType = getBodyType(request.body);
    this.jsonBody = getBodyContent(request.body);
    this.rawBody =
      "rawText" in request.body ? request.body.rawText._0 : "";
    this.formPairs = getFormPairs(request.body).length
      ? [...getFormPairs(request.body)]
      : [createEmptyPair()];
    this.multipartFields = getMultipartFields(request.body).length
      ? [...getMultipartFields(request.body)]
      : [createEmptyMultipartField()];
    const gql = getGraphQLContent(request.body);
    this.graphqlQuery = gql.query;
    this.graphqlVariables = gql.variables;
    this.extractionRules = request.responseExtractions?.length
      ? [...request.responseExtractions]
      : [createEmptyExtractionRule()];
    this.loadAuth(request.auth);
    this.response = null;
    this.errorMessage = null;
    this.isDirty = false;
  }

  private loadAuth(auth?: import("../types").AuthConfig) {
    this.authType = getAuthType(auth);
    this.bearerToken = "";
    this.basicUsername = "";
    this.basicPassword = "";
    this.apiKeyKey = "";
    this.apiKeyValue = "";
    this.apiKeyLocation = "header";
    this.resetOAuth2Fields();
    if (auth && "bearerToken" in auth) {
      this.bearerToken = auth.bearerToken._0.token;
    } else if (auth && "basicAuth" in auth) {
      this.basicUsername = auth.basicAuth._0.username;
      this.basicPassword = auth.basicAuth._0.password;
    } else if (auth && "apiKey" in auth) {
      this.apiKeyKey = auth.apiKey._0.key;
      this.apiKeyValue = auth.apiKey._0.value;
      this.apiKeyLocation = auth.apiKey._0.location;
    } else if (auth && "oauth2" in auth) {
      const o = auth.oauth2;
      this.oauthGrantType = o.grantType;
      this.oauthClientId = o.clientId;
      this.oauthClientSecret = o.clientSecret;
      this.oauthAuthUrl = o.authUrl;
      this.oauthTokenUrl = o.tokenUrl;
      this.oauthScopes = o.scopes;
      this.oauthAccessToken = o.accessToken;
      this.oauthRefreshToken = o.refreshToken;
      this.oauthTokenExpiry = o.tokenExpiry;
    }
  }

  // Build a SavedRequest from current editor state
  toSavedRequest(): SavedRequest | null {
    if (!this.requestId) return null;
    return {
      id: this.requestId,
      name: this.name,
      method: this.method,
      url: this.url,
      queryParams: this.queryParams.filter((p) => !p.key && !p.value ? false : true),
      headers: this.headers.filter((p) => !p.key && !p.value ? false : true),
      body: buildRequestBody(
        this.bodyType,
        this.jsonBody,
        this.rawBody,
        this.formPairs.filter((p) => !p.key && !p.value ? false : true),
        this.multipartFields.filter((f) => !f.name ? false : true),
        { query: this.graphqlQuery, variables: this.graphqlVariables },
      ),
      auth: buildAuthConfig(
        this.authType,
        { token: this.bearerToken },
        { username: this.basicUsername, password: this.basicPassword },
        { key: this.apiKeyKey, value: this.apiKeyValue, location: this.apiKeyLocation },
        this.buildOAuth2Config(),
      ),
      sortOrder: 0,
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
      responseExtractions: this.extractionRules.filter(
        (r) => r.variableName || r.jsonPath,
      ),
    };
  }

  // Save current editor state to the store
  saveIfDirty() {
    if (!this.isDirty || !this.requestId) return;
    const request = this.toSavedRequest();
    if (request) {
      appStore.updateRequest(request);
      this.isDirty = false;
    }
  }

  markDirty() {
    this.isDirty = true;
  }

  // Send the current request
  async send() {
    if (!this.canSend) return;

    // Synchronously set loading to prevent double-send from rapid clicks
    this.isLoading = true;
    this.errorMessage = null;
    this.response = null;

    try {
      const body = buildRequestBody(
        this.bodyType,
        this.jsonBody,
        this.rawBody,
        this.formPairs,
        this.multipartFields,
        { query: this.graphqlQuery, variables: this.graphqlVariables },
      );
      const auth = buildAuthConfig(
        this.authType,
        { token: this.bearerToken },
        { username: this.basicUsername, password: this.basicPassword },
        { key: this.apiKeyKey, value: this.apiKeyValue, location: this.apiKeyLocation },
        this.buildOAuth2Config(),
      );
      const result = await sendRequest({
        method: this.method,
        url: this.url,
        headers: this.headers,
        queryParams: this.queryParams,
        body,
        auth,
        timeoutSecs: this.timeoutSecs,
        followRedirects: this.followRedirects,
        environment: environmentStore.activeEnvironment,
        proxyConfig: appStore.proxyConfig.enabled ? appStore.proxyConfig : undefined,
      });
      this.response = result;
      this.applyExtractions(result);

      // Add to history with full request snapshot for replay
      historyStore.addEntry({
        method: this.method,
        url: this.url,
        statusCode: result.statusCode,
        elapsedTime: result.elapsedTime,
        requestId: this.requestId ?? null,
        requestName: this.name || null,
        snapshot: this.toSavedRequest() ?? undefined,
      });
    } catch (e) {
      this.errorMessage =
        e instanceof Error ? e.message : String(e);
    } finally {
      this.isLoading = false;
    }
  }

  // Cancel the in-flight request via Rust-side Notify
  async cancel() {
    if (!this.isLoading) return;
    try {
      await cancelRequest();
    } catch {
      // best-effort
    }
    this.isLoading = false;
  }

  // Apply extraction rules to response and write to environment
  private applyExtractions(response: ResponseRecord) {
    if (!response.bodyString || !response.isJson) return;
    const enabledRules = this.extractionRules.filter(
      (r) => r.isEnabled && r.variableName && r.jsonPath,
    );
    if (enabledRules.length === 0) return;

    let parsed: unknown;
    try {
      parsed = JSON.parse(response.bodyString);
    } catch {
      return;
    }

    for (const rule of enabledRules) {
      const value = extractByPath(parsed, rule.jsonPath);
      if (value !== undefined) {
        environmentStore.setVariable(rule.variableName, value);
      }
    }
  }

  // Reset to empty
  reset() {
    this.requestId = null;
    this.name = "New Request";
    this.method = "GET";
    this.url = "";
    this.queryParams = [createEmptyPair()];
    this.headers = [createEmptyPair()];
    this.bodyType = "none";
    this.jsonBody = "";
    this.rawBody = "";
    this.formPairs = [createEmptyPair()];
    this.multipartFields = [createEmptyMultipartField()];
    this.graphqlQuery = "";
    this.graphqlVariables = "";
    this.extractionRules = [createEmptyExtractionRule()];
    this.authType = "none";
    this.bearerToken = "";
    this.basicUsername = "";
    this.basicPassword = "";
    this.apiKeyKey = "";
    this.apiKeyValue = "";
    this.apiKeyLocation = "header";
    this.resetOAuth2Fields();
    this.response = null;
    this.pinnedResponse = null;
    this.errorMessage = null;
    this.isDirty = false;
  }

  private resetOAuth2Fields() {
    this.oauthGrantType = "client_credentials";
    this.oauthClientId = "";
    this.oauthClientSecret = "";
    this.oauthAuthUrl = "";
    this.oauthTokenUrl = "";
    this.oauthScopes = "";
    this.oauthAccessToken = "";
    this.oauthRefreshToken = "";
    this.oauthTokenExpiry = null;
  }

  pinResponse() {
    if (this.response) {
      this.pinnedResponse = { ...this.response };
    }
  }

  unpinResponse() {
    this.pinnedResponse = null;
  }

  buildOAuth2Config(): OAuth2Config {
    return {
      grantType: this.oauthGrantType,
      clientId: this.oauthClientId,
      clientSecret: this.oauthClientSecret,
      authUrl: this.oauthAuthUrl,
      tokenUrl: this.oauthTokenUrl,
      scopes: this.oauthScopes,
      accessToken: this.oauthAccessToken,
      refreshToken: this.oauthRefreshToken,
      tokenExpiry: this.oauthTokenExpiry,
    };
  }
}

export const editorStore = new EditorStore();
