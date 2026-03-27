import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { toastStore } from "./toast.svelte";

describe("toastStore", () => {
  beforeEach(() => {
    vi.useFakeTimers();
    toastStore.message = null;
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("sets message on show", () => {
    toastStore.show("hello");
    expect(toastStore.message).toBe("hello");
  });

  it("clears message after default duration (2000ms)", () => {
    toastStore.show("hello");
    vi.advanceTimersByTime(1999);
    expect(toastStore.message).toBe("hello");
    vi.advanceTimersByTime(1);
    expect(toastStore.message).toBeNull();
  });

  it("clears message after custom duration", () => {
    toastStore.show("hello", 500);
    vi.advanceTimersByTime(500);
    expect(toastStore.message).toBeNull();
  });

  it("resets timer on consecutive calls", () => {
    toastStore.show("first", 1000);
    vi.advanceTimersByTime(800);
    toastStore.show("second", 1000);
    vi.advanceTimersByTime(800);
    // first timer would have fired at 1000, but second resets
    expect(toastStore.message).toBe("second");
    vi.advanceTimersByTime(200);
    expect(toastStore.message).toBeNull();
  });
});
