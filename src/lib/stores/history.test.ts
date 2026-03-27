import { describe, it, expect, vi, beforeEach } from "vitest";
import { historyStore } from "./history.svelte";

describe("historyStore", () => {
  beforeEach(() => {
    historyStore.history = [];
  });

  it("adds entry with auto-generated id and timestamp", () => {
    historyStore.addEntry({
      method: "GET",
      url: "https://example.com",
      statusCode: 200,
      elapsedTime: 123,
    });
    expect(historyStore.history).toHaveLength(1);
    expect(historyStore.history[0].id).toBeTruthy();
    expect(historyStore.history[0].timestamp).toBeTruthy();
    expect(historyStore.history[0].url).toBe("https://example.com");
  });

  it("prepends new entries (most recent first)", () => {
    historyStore.addEntry({
      method: "GET",
      url: "https://first.com",
      statusCode: 200,
      elapsedTime: 100,
    });
    historyStore.addEntry({
      method: "POST",
      url: "https://second.com",
      statusCode: 201,
      elapsedTime: 200,
    });
    expect(historyStore.history[0].url).toBe("https://second.com");
    expect(historyStore.history[1].url).toBe("https://first.com");
  });

  it("truncates history to MAX_HISTORY_ENTRIES", () => {
    for (let i = 0; i < 105; i++) {
      historyStore.addEntry({
        method: "GET",
        url: `https://example.com/${i}`,
        statusCode: 200,
        elapsedTime: 10,
      });
    }
    expect(historyStore.history.length).toBeLessThanOrEqual(100);
  });

  it("clears all entries", () => {
    historyStore.addEntry({
      method: "GET",
      url: "https://example.com",
      statusCode: 200,
      elapsedTime: 50,
    });
    historyStore.clear();
    expect(historyStore.history).toHaveLength(0);
  });

  it("fires onStateChange callback on addEntry", () => {
    const cb = vi.fn();
    historyStore.onStateChange(cb);
    historyStore.addEntry({
      method: "GET",
      url: "https://example.com",
      statusCode: 200,
      elapsedTime: 50,
    });
    expect(cb).toHaveBeenCalledOnce();
  });

  it("fires onStateChange callback on clear", () => {
    const cb = vi.fn();
    historyStore.onStateChange(cb);
    historyStore.clear();
    expect(cb).toHaveBeenCalledOnce();
  });
});
