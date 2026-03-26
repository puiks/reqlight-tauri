import { invoke } from "@tauri-apps/api/core";
import type {
  AppState,
  HttpMethod,
  KeyValuePair,
  RequestBody,
  RequestEnvironment,
  ResponseRecord,
  SavedRequest,
} from "./types";

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
  timeoutSecs?: number;
  environment?: RequestEnvironment;
}): Promise<ResponseRecord> {
  return invoke<ResponseRecord>("send_request", params);
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
