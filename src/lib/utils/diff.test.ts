import { describe, it, expect } from "vitest";
import { computeDiff } from "./diff";

describe("computeDiff", () => {
  it("returns no changes for identical strings", () => {
    const result = computeDiff("hello\nworld", "hello\nworld");
    expect(result.hasChanges).toBe(false);
    expect(result.left).toHaveLength(2);
    expect(result.right).toHaveLength(2);
    expect(result.left[0].type).toBe("same");
    expect(result.left[1].type).toBe("same");
  });

  it("detects added lines", () => {
    const result = computeDiff("a\nb", "a\nb\nc");
    expect(result.hasChanges).toBe(true);
    const addedRight = result.right.filter((l) => l.type === "added");
    expect(addedRight).toHaveLength(1);
    expect(addedRight[0].content).toBe("c");
  });

  it("detects removed lines", () => {
    const result = computeDiff("a\nb\nc", "a\nc");
    expect(result.hasChanges).toBe(true);
    const removedLeft = result.left.filter((l) => l.type === "removed");
    expect(removedLeft).toHaveLength(1);
    expect(removedLeft[0].content).toBe("b");
  });

  it("handles completely different content", () => {
    const result = computeDiff("foo\nbar", "baz\nqux");
    expect(result.hasChanges).toBe(true);
  });

  it("handles empty old text", () => {
    const result = computeDiff("", "new line");
    expect(result.hasChanges).toBe(true);
    const addedRight = result.right.filter((l) => l.type === "added");
    expect(addedRight.length).toBeGreaterThan(0);
  });

  it("handles empty new text", () => {
    const result = computeDiff("old line", "");
    expect(result.hasChanges).toBe(true);
    const removedLeft = result.left.filter((l) => l.type === "removed");
    expect(removedLeft.length).toBeGreaterThan(0);
  });

  it("handles both empty", () => {
    const result = computeDiff("", "");
    expect(result.hasChanges).toBe(false);
  });

  it("assigns correct line numbers", () => {
    const result = computeDiff("a\nb\nc", "a\nx\nc");
    // "a" and "c" are common; "b" removed, "x" added
    const sameLeft = result.left.filter((l) => l.type === "same");
    expect(sameLeft[0].lineNumber).toBe(1);
    expect(sameLeft[1].lineNumber).toBe(3);
  });

  it("handles multiline JSON diff", () => {
    const old = JSON.stringify({ name: "Alice", age: 30 }, null, 2);
    const newStr = JSON.stringify({ name: "Bob", age: 30 }, null, 2);
    const result = computeDiff(old, newStr);
    expect(result.hasChanges).toBe(true);
    // "age" line should be common
    const sameLines = result.left.filter((l) => l.type === "same");
    expect(sameLines.some((l) => l.content.includes("age"))).toBe(true);
  });
});
