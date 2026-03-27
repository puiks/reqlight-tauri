import { describe, expect, it } from "vitest";
import { formatJson, highlightJson, validateJson } from "./json-highlighter";

describe("highlightJson", () => {
  it("highlights a simple object with key and string value", () => {
    const result = highlightJson('{"name":"Alice"}');
    expect(result).toContain('class="json-key"');
    expect(result).toContain('class="json-string"');
    expect(result).toContain('class="json-brace"');
    expect(result).toContain('class="json-colon"');
  });

  it("highlights number values", () => {
    const result = highlightJson('{"age":30}');
    expect(result).toContain('class="json-number"');
    expect(result).toContain("30");
  });

  it("highlights boolean values", () => {
    const result = highlightJson('{"ok":true,"fail":false}');
    expect(result).toContain('class="json-boolean"');
  });

  it("highlights null values", () => {
    const result = highlightJson('{"v":null}');
    expect(result).toContain('class="json-null"');
  });

  it("highlights arrays with brackets", () => {
    const result = highlightJson("[1,2,3]");
    expect(result).toContain('class="json-bracket"');
    expect(result).toContain('class="json-number"');
    expect(result).toContain('class="json-comma"');
  });

  it("handles nested objects", () => {
    const json = '{"a":{"b":1}}';
    const result = highlightJson(json);
    // Should have multiple brace tokens for nested structure
    const braceMatches = result.match(/json-brace/g);
    expect(braceMatches!.length).toBeGreaterThanOrEqual(4); // at least 4 braces + whitespace
  });

  it("escapes HTML characters in values", () => {
    const result = highlightJson('{"html":"<b>bold</b>"}');
    expect(result).toContain("&lt;b&gt;bold&lt;/b&gt;");
    expect(result).not.toContain("<b>bold</b>");
  });

  it("handles escaped quotes in strings", () => {
    const result = highlightJson('{"msg":"say \\"hi\\""}');
    expect(result).toContain("json-string");
  });

  it("returns empty string for empty input", () => {
    expect(highlightJson("")).toBe("");
  });

  it("handles negative numbers", () => {
    const result = highlightJson('{"temp":-5}');
    expect(result).toContain('class="json-number"');
    expect(result).toContain("-5");
  });

  it("handles scientific notation numbers", () => {
    const result = highlightJson('{"big":1e10}');
    expect(result).toContain('class="json-number"');
    expect(result).toContain("1e10");
  });

  it("distinguishes keys from string values in objects", () => {
    const result = highlightJson('{"key":"value"}');
    expect(result).toContain('class="json-key">&quot;key&quot;</span>');
    expect(result).toContain('class="json-string">&quot;value&quot;</span>');
  });

  it("treats strings in arrays as values, not keys", () => {
    const result = highlightJson('["a","b"]');
    // In an array context, strings should be "string" type, not "key"
    expect(result).toContain('class="json-string"');
    expect(result).not.toContain('class="json-key"');
  });
});

describe("formatJson", () => {
  it("formats compact JSON with indentation", () => {
    const result = formatJson('{"a":1,"b":2}');
    expect(result).toBe('{\n  "a": 1,\n  "b": 2\n}');
  });

  it("returns original string for invalid JSON", () => {
    const invalid = "{not json}";
    expect(formatJson(invalid)).toBe(invalid);
  });

  it("handles already formatted JSON", () => {
    const formatted = '{\n  "a": 1\n}';
    const result = formatJson(formatted);
    expect(result).toBe(formatted);
  });

  it("handles empty object", () => {
    expect(formatJson("{}")).toBe("{}");
  });

  it("handles empty array", () => {
    expect(formatJson("[]")).toBe("[]");
  });
});

describe("validateJson", () => {
  it("returns null for valid JSON object", () => {
    expect(validateJson('{"a":1}')).toBeNull();
  });

  it("returns null for valid JSON array", () => {
    expect(validateJson("[1,2,3]")).toBeNull();
  });

  it("returns null for empty/whitespace input", () => {
    expect(validateJson("")).toBeNull();
    expect(validateJson("  ")).toBeNull();
  });

  it("returns error message for invalid JSON", () => {
    const result = validateJson("{invalid}");
    expect(result).not.toBeNull();
    expect(typeof result).toBe("string");
  });

  it("returns error message for trailing comma", () => {
    const result = validateJson('{"a":1,}');
    expect(result).not.toBeNull();
  });

  it("returns null for valid primitives", () => {
    expect(validateJson("42")).toBeNull();
    expect(validateJson('"hello"')).toBeNull();
    expect(validateJson("true")).toBeNull();
    expect(validateJson("null")).toBeNull();
  });
});
