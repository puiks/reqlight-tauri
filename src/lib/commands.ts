import type {
  AppState,
  AuthConfig,
  HttpMethod,
  KeyValuePair,
  RequestBody,
  RequestEnvironment,
  ResponseRecord,
  SavedRequest,
} from "./types";

// Check if running inside Tauri webview
const isTauri = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (isTauri) {
    const { invoke: tauriInvoke } = await import("@tauri-apps/api/core");
    return tauriInvoke<T>(cmd, args);
  }
  // Dev/E2E fallback — return mock data
  return devFallback(cmd) as T;
}

function devFallback(cmd: string): unknown {
  switch (cmd) {
    case "load_state":
      return {
        collections: [],
        environments: [],
        activeEnvironmentId: null,
        lastSelectedCollectionId: null,
        lastSelectedRequestId: null,
        history: [],
      };
    case "save_state":
    case "secret_set":
    case "secret_delete":
    case "cancel_request":
      return undefined;
    case "secret_get":
      return null;
    default:
      return undefined;
  }
}

// Persistence
export async function loadState(): Promise<AppState> {
  return invoke<AppState>("load_state");
}

export async function saveState(state: AppState): Promise<void> {
  return invoke("save_state", { state });
}

// HTTP
export async function sendRequest(params: {
  method: HttpMethod;
  url: string;
  headers: KeyValuePair[];
  queryParams: KeyValuePair[];
  body: RequestBody;
  auth: AuthConfig;
  timeoutSecs?: number;
  environment?: RequestEnvironment;
}): Promise<ResponseRecord> {
  return invoke<ResponseRecord>("send_request", params);
}

export async function cancelRequest(): Promise<void> {
  return invoke("cancel_request");
}

// Keychain
export async function secretSet(
  key: string,
  value: string,
): Promise<void> {
  return invoke("secret_set", { key, value });
}

export async function secretGet(key: string): Promise<string | null> {
  return invoke<string | null>("secret_get", { key });
}

export async function secretDelete(key: string): Promise<void> {
  return invoke("secret_delete", { key });
}

// cURL
export async function parseCurl(
  curlString: string,
): Promise<SavedRequest> {
  return invoke<SavedRequest>("parse_curl", { curlString });
}

export async function exportCurl(
  request: SavedRequest,
  environment?: RequestEnvironment,
): Promise<string> {
  return invoke<string>("export_curl", { request, environment });
}

// WebSocket
export async function wsConnect(
  connectionId: string,
  url: string,
): Promise<void> {
  return invoke("ws_connect", { connectionId, url });
}

export async function wsSend(
  connectionId: string,
  message: string,
): Promise<void> {
  return invoke("ws_send", { connectionId, message });
}

export async function wsDisconnect(
  connectionId: string,
): Promise<void> {
  return invoke("ws_disconnect", { connectionId });
}
