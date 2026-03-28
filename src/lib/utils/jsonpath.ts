/**
 * Extract a value from a parsed JSON object using a simple JSONPath expression.
 * Supports: $.foo.bar, $.items[0].name, $.data[2].nested.key
 * Returns the stringified value, or undefined if the path doesn't resolve.
 */
export function extractByPath(obj: unknown, path: string): string | undefined {
  if (obj === null || obj === undefined) return undefined;

  // Normalize: strip leading "$." or "$"
  let normalized = path.trim();
  if (normalized.startsWith("$.")) {
    normalized = normalized.slice(2);
  } else if (normalized.startsWith("$[")) {
    normalized = normalized.slice(1);
  } else if (normalized === "$") {
    return stringify(obj);
  } else if (normalized.startsWith("$")) {
    normalized = normalized.slice(1);
  }

  if (!normalized) return stringify(obj);

  // Tokenize: split by "." but handle array indices like "items[0]"
  const tokens = tokenize(normalized);

  let current: unknown = obj;
  for (const token of tokens) {
    if (current === null || current === undefined) return undefined;

    const arrayMatch = token.match(/^([^[]*)\[(\d+)\]$/);
    if (arrayMatch) {
      const [, key, indexStr] = arrayMatch;
      if (key) {
        if (typeof current !== "object") return undefined;
        current = (current as Record<string, unknown>)[key];
      }
      if (!Array.isArray(current)) return undefined;
      const index = parseInt(indexStr, 10);
      current = current[index];
    } else {
      if (typeof current !== "object" || current === null) return undefined;
      current = (current as Record<string, unknown>)[token];
    }
  }

  return stringify(current);
}

function tokenize(path: string): string[] {
  const tokens: string[] = [];
  let current = "";
  let inBracket = false;
  for (let i = 0; i < path.length; i++) {
    const ch = path[i];
    if (ch === "[") inBracket = true;
    if (ch === "]") inBracket = false;
    if (ch === "." && !inBracket) {
      if (current) tokens.push(current);
      current = "";
    } else {
      current += ch;
    }
  }
  if (current) tokens.push(current);
  return tokens;
}

function stringify(value: unknown): string | undefined {
  if (value === undefined) return undefined;
  if (value === null) return "null";
  if (typeof value === "string") return value;
  if (typeof value === "number" || typeof value === "boolean") return String(value);
  return JSON.stringify(value);
}
