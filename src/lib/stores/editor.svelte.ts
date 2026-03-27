import { sendRequest, cancelRequest } from "../commands";
import {
  buildAuthConfig,
  buildRequestBody,
  createEmptyAuth,
  createEmptyPair,
  getAuthType,
  getBodyContent,
  getBodyType,
  getFormPairs,
  type ApiKeyLocation,
  type AuthType,
  type BodyType,
  type EditorTab,
  type HttpMethod,
  type KeyValuePair,
  type ResponseRecord,
  type ResponseTab,
  type SavedRequest,
} from "../types";
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

  // Auth fields
  authType = $state<AuthType>("none");
  bearerToken = $state("");
  basicUsername = $state("");
  basicPassword = $state("");
  apiKeyKey = $state("");
  apiKeyValue = $state("");
  apiKeyLocation = $state<ApiKeyLocation>("header");

  // UI state
  activeEditorTab = $state<EditorTab>("params");
  activeResponseTab = $state<ResponseTab>("body");
  response = $state<ResponseRecord | null>(null);
  isLoading = $state(false);
  errorMessage = $state<string | null>(null);
  isDirty = $state(false);
  timeoutSecs = $state(DEFAULT_REQUEST_TIMEOUT);
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
    if (auth && "bearerToken" in auth) {
      this.bearerToken = auth.bearerToken._0.token;
    } else if (auth && "basicAuth" in auth) {
      this.basicUsername = auth.basicAuth._0.username;
      this.basicPassword = auth.basicAuth._0.password;
    } else if (auth && "apiKey" in auth) {
      this.apiKeyKey = auth.apiKey._0.key;
      this.apiKeyValue = auth.apiKey._0.value;
      this.apiKeyLocation = auth.apiKey._0.location;
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
      ),
      auth: buildAuthConfig(
        this.authType,
        { token: this.bearerToken },
        { username: this.basicUsername, password: this.basicPassword },
        { key: this.apiKeyKey, value: this.apiKeyValue, location: this.apiKeyLocation },
      ),
      sortOrder: 0,
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
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
      );
      const auth = buildAuthConfig(
        this.authType,
        { token: this.bearerToken },
        { username: this.basicUsername, password: this.basicPassword },
        { key: this.apiKeyKey, value: this.apiKeyValue, location: this.apiKeyLocation },
      );
      const result = await sendRequest({
        method: this.method,
        url: this.url,
        headers: this.headers,
        queryParams: this.queryParams,
        body,
        auth,
        timeoutSecs: this.timeoutSecs,
        environment: environmentStore.activeEnvironment,
      });
      this.response = result;

      // Add to history with full request snapshot for replay
      historyStore.addEntry({
        method: this.method,
        url: this.url,
        statusCode: result.statusCode,
        elapsedTime: result.elapsedTime,
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
    this.authType = "none";
    this.bearerToken = "";
    this.basicUsername = "";
    this.basicPassword = "";
    this.apiKeyKey = "";
    this.apiKeyValue = "";
    this.apiKeyLocation = "header";
    this.response = null;
    this.errorMessage = null;
    this.isDirty = false;
  }
}

export const editorStore = new EditorStore();
