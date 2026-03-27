import { describe, it, expect } from "vitest";
import { escapeHtml } from "./html";

describe("escapeHtml", () => {
  it("escapes ampersands", () => {
    expect(escapeHtml("a&b")).toBe("a&amp;b");
  });

  it("escapes angle brackets", () => {
    expect(escapeHtml("<div>")).toBe("&lt;div&gt;");
  });

  it("escapes double quotes", () => {
    expect(escapeHtml('"hello"')).toBe("&quot;hello&quot;");
  });

  it("escapes all special characters together", () => {
    expect(escapeHtml('<a href="x">&')).toBe("&lt;a href=&quot;x&quot;&gt;&amp;");
  });

  it("returns empty string for empty input", () => {
    expect(escapeHtml("")).toBe("");
  });

  it("returns plain text unchanged", () => {
    expect(escapeHtml("hello world")).toBe("hello world");
  });
});
