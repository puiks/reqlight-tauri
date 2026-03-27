/** Lightweight JSON syntax highlighter.
 *  Ported from Swift's JSONHighlighter — tokenizes JSON and wraps
 *  each token in a <span class="json-*"> for CSS styling.
 */
import { escapeHtml } from "./html";

type TokenType =
  | "key"
  | "string"
  | "number"
  | "boolean"
  | "null"
  | "brace"
  | "bracket"
  | "colon"
  | "comma";

interface Token {
  type: TokenType;
  value: string;
}

export function highlightJson(json: string): string {
  const tokens = tokenize(json);
  return tokens
    .map((t) => {
      const escaped = escapeHtml(t.value);
      return `<span class="json-${t.type}">${escaped}</span>`;
    })
    .join("");
}

function tokenize(input: string): Token[] {
  const tokens: Token[] = [];
  let i = 0;
  let expectingKey = true;

  while (i < input.length) {
    const ch = input[i];

    // Whitespace
    if (/\s/.test(ch)) {
      let start = i;
      while (i < input.length && /\s/.test(input[i])) i++;
      tokens.push({ type: "brace", value: input.slice(start, i) }); // neutral
      continue;
    }

    // Strings (keys or values)
    if (ch === '"') {
      const start = i;
      i++; // skip opening "
      while (i < input.length) {
        if (input[i] === "\\") {
          i += 2; // skip escaped char
        } else if (input[i] === '"') {
          i++; // skip closing "
          break;
        } else {
          i++;
        }
      }
      const value = input.slice(start, i);
      tokens.push({ type: expectingKey ? "key" : "string", value });
      continue;
    }

    // Numbers
    if (ch === "-" || (ch >= "0" && ch <= "9")) {
      const start = i;
      if (ch === "-") i++;
      while (i < input.length && /[0-9.eE+\-]/.test(input[i])) i++;
      tokens.push({ type: "number", value: input.slice(start, i) });
      expectingKey = false;
      continue;
    }

    // Booleans
    if (input.slice(i, i + 4) === "true") {
      tokens.push({ type: "boolean", value: "true" });
      i += 4;
      expectingKey = false;
      continue;
    }
    if (input.slice(i, i + 5) === "false") {
      tokens.push({ type: "boolean", value: "false" });
      i += 5;
      expectingKey = false;
      continue;
    }

    // Null
    if (input.slice(i, i + 4) === "null") {
      tokens.push({ type: "null", value: "null" });
      i += 4;
      expectingKey = false;
      continue;
    }

    // Structural characters
    if (ch === "{") {
      tokens.push({ type: "brace", value: "{" });
      expectingKey = true;
      i++;
      continue;
    }
    if (ch === "}") {
      tokens.push({ type: "brace", value: "}" });
      expectingKey = false;
      i++;
      continue;
    }
    if (ch === "[") {
      tokens.push({ type: "bracket", value: "[" });
      expectingKey = false;
      i++;
      continue;
    }
    if (ch === "]") {
      tokens.push({ type: "bracket", value: "]" });
      expectingKey = false;
      i++;
      continue;
    }
    if (ch === ":") {
      tokens.push({ type: "colon", value: ":" });
      expectingKey = false;
      i++;
      continue;
    }
    if (ch === ",") {
      tokens.push({ type: "comma", value: "," });
      // After comma in object context → next is key; in array → value
      // We approximate: after comma, if the last non-whitespace was } or ] or string/number → key
      expectingKey = isInObjectContext(tokens);
      i++;
      continue;
    }

    // Unknown character — skip
    tokens.push({ type: "brace", value: ch });
    i++;
  }

  return tokens;
}

function isInObjectContext(tokens: Token[]): boolean {
  // Walk backwards to find the most recent unmatched { or [
  let depth = 0;
  for (let i = tokens.length - 1; i >= 0; i--) {
    const v = tokens[i].value;
    if (v === "}" || v === "]") depth++;
    else if (v === "{") {
      if (depth === 0) return true; // in object
      depth--;
    } else if (v === "[") {
      if (depth === 0) return false; // in array
      depth--;
    }
  }
  return true;
}


/** Try to format JSON with 2-space indent. Returns original on failure. */
export function formatJson(input: string): string {
  try {
    return JSON.stringify(JSON.parse(input), null, 2);
  } catch {
    return input;
  }
}

/** Validate JSON string. Returns null if valid, error message if invalid. */
export function validateJson(input: string): string | null {
  if (!input.trim()) return null;
  try {
    JSON.parse(input);
    return null;
  } catch (e) {
    return (e as Error).message;
  }
}
