export interface KeyValuePair {
  id: string;
  key: string;
  value: string;
  isEnabled: boolean;
  isSecret: boolean;
}

export type HttpMethod = "GET" | "POST" | "PUT" | "PATCH" | "DELETE";

export type RequestBody =
  | { none: Record<string, never> }
  | { json: { _0: string } }
  | { formData: { _0: KeyValuePair[] } }
  | { rawText: { _0: string } };

export type BodyType = "none" | "json" | "formData" | "rawText";

export interface SavedRequest {
  id: string;
  name: string;
  method: HttpMethod;
  url: string;
  queryParams: KeyValuePair[];
  headers: KeyValuePair[];
  body: RequestBody;
  sortOrder: number;
  createdAt: string;
  updatedAt: string;
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
}

export interface AppState {
  collections: RequestCollection[];
  environments: RequestEnvironment[];
  activeEnvironmentId: string | null;
  lastSelectedCollectionId: string | null;
  lastSelectedRequestId: string | null;
  history: RequestHistoryEntry[];
}

export type EditorTab = "params" | "headers" | "body";
export type ResponseTab = "body" | "headers";
export type AppAppearance = "system" | "light" | "dark";

export const HTTP_METHODS: HttpMethod[] = [
  "GET",
  "POST",
  "PUT",
  "PATCH",
  "DELETE",
];

export const METHOD_COLORS: Record<HttpMethod, string> = {
  GET: "var(--color-method-get)",
  POST: "var(--color-method-post)",
  PUT: "var(--color-method-put)",
  PATCH: "var(--color-method-patch)",
  DELETE: "var(--color-method-delete)",
};

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
  return "none";
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
  }
}
