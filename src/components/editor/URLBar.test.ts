import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, cleanup } from "@testing-library/svelte";

const mockStore = vi.hoisted(() => ({
  method: "GET" as string,
  url: "",
  isUrlValid: true,
  isLoading: false,
  _canSend: false,
  get canSend() {
    return this._canSend;
  },
  timeoutSecs: 30,
  send: vi.fn(),
  cancel: vi.fn(),
  markDirty: vi.fn(),
}));

vi.mock("../../lib/stores/editor.svelte", () => ({
  editorStore: mockStore,
}));

vi.mock("../../lib/types", async (importOriginal) => {
  const actual = await importOriginal<Record<string, unknown>>();
  return {
    ...actual,
    HTTP_METHODS: ["GET", "POST", "PUT", "PATCH", "DELETE"],
    METHOD_COLORS: {
      GET: "#22c55e",
      POST: "#f97316",
      PUT: "#3b82f6",
      PATCH: "#a855f7",
      DELETE: "#ef4444",
    },
  };
});

import URLBar from "./URLBar.svelte";

describe("URLBar", () => {
  beforeEach(() => {
    cleanup();
    vi.clearAllMocks();
    mockStore.method = "GET";
    mockStore.url = "";
    mockStore.isUrlValid = true;
    mockStore.isLoading = false;
    mockStore._canSend = false;
    mockStore.timeoutSecs = 30;
  });

  it("renders method selector with all HTTP methods", () => {
    render(URLBar);
    const select = screen.getByRole("combobox");
    const options = select.querySelectorAll("option");
    expect(options).toHaveLength(5);
    expect(options[0].textContent).toBe("GET");
    expect(options[1].textContent).toBe("POST");
  });

  it("renders URL input with placeholder", () => {
    render(URLBar);
    const input = screen.getByPlaceholderText("Enter URL (e.g. https://api.example.com)");
    expect(input).toBeInTheDocument();
  });

  it("shows Send button when not loading", () => {
    render(URLBar);
    expect(screen.getByText("Send")).toBeInTheDocument();
    expect(screen.queryByText("Cancel")).not.toBeInTheDocument();
  });

  it("shows Cancel button when loading", () => {
    mockStore.isLoading = true;
    render(URLBar);
    expect(screen.getByText("Cancel")).toBeInTheDocument();
    expect(screen.queryByText("Send")).not.toBeInTheDocument();
  });

  it("Send button is disabled when canSend is false", () => {
    mockStore._canSend = false;
    render(URLBar);
    const btn = screen.getByText("Send");
    expect(btn).toBeDisabled();
  });

  it("Send button is enabled when canSend is true", () => {
    mockStore._canSend = true;
    render(URLBar);
    const btn = screen.getByText("Send");
    expect(btn).not.toBeDisabled();
  });

  it("renders timeout input", () => {
    render(URLBar);
    const input = screen.getByTitle("Request timeout (seconds)");
    expect(input).toBeInTheDocument();
  });
});
