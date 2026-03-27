import { describe, it, expect, vi, beforeEach } from "vitest";
import { handleError, withErrorHandling } from "./errors";

vi.mock("../stores/toast.svelte", () => ({
  toastStore: { show: vi.fn() },
}));

import { toastStore } from "../stores/toast.svelte";

describe("handleError", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.spyOn(console, "error").mockImplementation(() => {});
  });

  it("logs Error instance to console with context", () => {
    handleError(new Error("boom"), "test-ctx");
    expect(console.error).toHaveBeenCalledWith("[test-ctx]", "boom");
  });

  it("logs string error to console", () => {
    handleError("string error", "ctx");
    expect(console.error).toHaveBeenCalledWith("[ctx]", "string error");
  });

  it("shows toast by default", () => {
    handleError("fail", "ctx");
    expect(toastStore.show).toHaveBeenCalledWith("Error: fail");
  });

  it("skips toast when silent", () => {
    handleError("fail", "ctx", { silent: true });
    expect(toastStore.show).not.toHaveBeenCalled();
  });
});

describe("withErrorHandling", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.spyOn(console, "error").mockImplementation(() => {});
  });

  it("calls wrapped function normally", async () => {
    const fn = vi.fn().mockResolvedValue("ok");
    const wrapped = withErrorHandling(fn, "ctx");
    const result = await wrapped();
    expect(fn).toHaveBeenCalled();
    expect(result).toBe("ok");
  });

  it("catches errors and calls handleError", async () => {
    const fn = vi.fn().mockRejectedValue(new Error("async fail"));
    const wrapped = withErrorHandling(fn, "ctx");
    await wrapped();
    expect(console.error).toHaveBeenCalledWith("[ctx]", "async fail");
    expect(toastStore.show).toHaveBeenCalled();
  });
});
