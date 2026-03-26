import { sendRequest, cancelRequest } from "../commands";
import {
  buildRequestBody,
  createEmptyPair,
  getBodyContent,
  getBodyType,
  getFormPairs,
  type BodyType,
  type EditorTab,
  type HttpMethod,
  type KeyValuePair,
  type ResponseRecord,
  type ResponseTab,
  type SavedRequest,
} from "../types";
import { appStore } from "./app.svelte";

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

  // UI state
  activeEditorTab = $state<EditorTab>("params");
  activeResponseTab = $state<ResponseTab>("body");
  response = $state<ResponseRecord | null>(null);
  isLoading = $state(false);
  errorMessage = $state<string | null>(null);
  isDirty = $state(false);

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
    this.response = null;
    this.errorMessage = null;
    this.isDirty = false;
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
      const result = await sendRequest({
        method: this.method,
        url: this.url,
        headers: this.headers,
        queryParams: this.queryParams,
        body,
        environment: appStore.activeEnvironment,
      });
      this.response = result;

      // Add to history
      appStore.addHistoryEntry({
        method: this.method,
        url: this.url,
        statusCode: result.statusCode,
        elapsedTime: result.elapsedTime,
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
    this.response = null;
    this.errorMessage = null;
    this.isDirty = false;
  }
}

export const editorStore = new EditorStore();
