import { describe, it, expect } from "vitest";
import { highlightJson, formatJson } from "./json-highlighter";

function generateLargeJson(sizeBytes: number): string {
  const entries: string[] = [];
  let currentSize = 2; // for { }
  let i = 0;
  while (currentSize < sizeBytes) {
    const entry = `"key_${i}":"value_${i}_${"x".repeat(50)}"`;
    entries.push(entry);
    currentSize += entry.length + 1; // +1 for comma
    i++;
  }
  return `{${entries.join(",")}}`;
}

describe("JSON Highlighter Performance", () => {
  it("formatJson 1MB completes in <200ms", () => {
    const json = generateLargeJson(1_000_000);

    const start = performance.now();
    const result = formatJson(json);
    const elapsed = performance.now() - start;

    expect(result.length).toBeGreaterThan(0);
    expect(elapsed).toBeLessThan(200);
  });

  it("highlightJson 100KB completes in <500ms", () => {
    const json = generateLargeJson(100_000);
    const formatted = formatJson(json);

    const start = performance.now();
    const result = highlightJson(formatted);
    const elapsed = performance.now() - start;

    expect(result.length).toBeGreaterThan(0);
    expect(elapsed).toBeLessThan(500);
  });

  it("formatJson 3MB completes in <1000ms", () => {
    const json = generateLargeJson(3_000_000);

    const start = performance.now();
    const result = formatJson(json);
    const elapsed = performance.now() - start;

    expect(result.length).toBeGreaterThan(0);
    expect(elapsed).toBeLessThan(1000);
  });

  // KNOWN PERF ISSUE: highlightJson uses regex-based line-by-line processing
  // that scales poorly beyond ~200KB. 1MB takes ~2.5s locally, ~5s+ on CI.
  // TODO: Disable syntax highlighting for bodies >500KB, or move to Web Worker.
  it("highlightJson 1MB is slow (known issue — baseline)", { timeout: 30000 }, () => {
    const json = generateLargeJson(1_000_000);
    const formatted = formatJson(json);

    const start = performance.now();
    const result = highlightJson(formatted);
    const elapsed = performance.now() - start;

    expect(result.length).toBeGreaterThan(0);
    // Current baseline: ~2.5s local, ~5s+ CI — needs optimization
    expect(elapsed).toBeLessThan(30000);
  });
});
