import { describe, it, expect } from "vitest";
import { extractByPath } from "./jsonpath";

describe("extractByPath", () => {
  const sample = {
    data: {
      token: "abc123",
      user: { name: "Alice", age: 30 },
      items: [
        { id: 1, title: "First" },
        { id: 2, title: "Second" },
      ],
    },
    status: "ok",
    count: 42,
    active: true,
  };

  it("extracts simple dot notation", () => {
    expect(extractByPath(sample, "$.data.token")).toBe("abc123");
  });

  it("extracts nested dot notation", () => {
    expect(extractByPath(sample, "$.data.user.name")).toBe("Alice");
  });

  it("extracts number value as string", () => {
    expect(extractByPath(sample, "$.data.user.age")).toBe("30");
  });

  it("extracts boolean value as string", () => {
    expect(extractByPath(sample, "$.active")).toBe("true");
  });

  it("extracts root-level key", () => {
    expect(extractByPath(sample, "$.status")).toBe("ok");
  });

  it("extracts number at root", () => {
    expect(extractByPath(sample, "$.count")).toBe("42");
  });

  it("returns entire object for $", () => {
    const result = extractByPath(sample, "$");
    expect(result).toBeDefined();
    expect(JSON.parse(result!)).toEqual(sample);
  });

  it("extracts array element by index", () => {
    expect(extractByPath(sample, "$.data.items[0].title")).toBe("First");
    expect(extractByPath(sample, "$.data.items[1].id")).toBe("2");
  });

  it("extracts array element at root index", () => {
    const arr = [{ x: 10 }, { x: 20 }];
    expect(extractByPath(arr, "$[0].x")).toBe("10");
    expect(extractByPath(arr, "$[1].x")).toBe("20");
  });

  it("returns undefined for missing path", () => {
    expect(extractByPath(sample, "$.nonexistent")).toBeUndefined();
    expect(extractByPath(sample, "$.data.missing.deep")).toBeUndefined();
  });

  it("returns undefined for out-of-bounds array index", () => {
    expect(extractByPath(sample, "$.data.items[99].title")).toBeUndefined();
  });

  it("returns undefined for null input", () => {
    expect(extractByPath(null, "$.foo")).toBeUndefined();
  });

  it("returns undefined for undefined input", () => {
    expect(extractByPath(undefined, "$.foo")).toBeUndefined();
  });

  it("returns undefined when accessing property on non-object", () => {
    expect(extractByPath(sample, "$.status.length")).toBeUndefined();
  });

  it("returns JSON string for object value", () => {
    const result = extractByPath(sample, "$.data.user");
    expect(result).toBe('{"name":"Alice","age":30}');
  });

  it("returns JSON string for array value", () => {
    const result = extractByPath(sample, "$.data.items");
    expect(result).toBeDefined();
    expect(JSON.parse(result!)).toEqual(sample.data.items);
  });

  it("handles path without $. prefix", () => {
    expect(extractByPath(sample, "data.token")).toBe("abc123");
  });

  it("returns null as string for null value", () => {
    const obj = { val: null };
    expect(extractByPath(obj, "$.val")).toBe("null");
  });
});
