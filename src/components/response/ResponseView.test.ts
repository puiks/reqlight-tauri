import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen } from "@testing-library/svelte";

// Mock editorStore before importing the component
vi.mock("../../lib/stores/editor.svelte", () => {
  return {
    editorStore: {
      isLoading: false,
      errorMessage: null as string | null,
      response: null as any,
      activeResponseTab: "body" as string,
    },
  };
});

// Svelte 5 component mocks need a valid component shape
function createMockComponent() {
  // Svelte 5 compiled components are functions
  function MockComponent() {}
  // Tag it so svelte recognizes it
  (MockComponent as any).$$typeof = Symbol.for("svelte.component");
  return MockComponent;
}

vi.mock("./StatusBar.svelte", () => ({ default: createMockComponent() }));
vi.mock("./ResponseBody.svelte", () => ({ default: createMockComponent() }));
vi.mock("./ResponseHeaders.svelte", () => ({ default: createMockComponent() }));

import { editorStore } from "../../lib/stores/editor.svelte";
import ResponseView from "./ResponseView.svelte";

describe("ResponseView", () => {
  beforeEach(() => {
    editorStore.isLoading = false;
    editorStore.errorMessage = null;
    editorStore.response = null;
    editorStore.activeResponseTab = "body";
  });

  it("shows empty state when no response", () => {
    render(ResponseView);
    expect(screen.getByText("No Response")).toBeInTheDocument();
    expect(screen.getByText("Send a request to see the response here.")).toBeInTheDocument();
  });

  it("shows loading spinner when isLoading", () => {
    editorStore.isLoading = true;
    render(ResponseView);
    expect(screen.getByText("Sending request...")).toBeInTheDocument();
  });

  it("shows error state with message", () => {
    editorStore.errorMessage = "Connection refused";
    render(ResponseView);
    expect(screen.getByText("Request Failed")).toBeInTheDocument();
    expect(screen.getByText("Connection refused")).toBeInTheDocument();
  });

  it("shows response tabs when response exists", () => {
    editorStore.response = {
      statusCode: 200,
      headers: [],
      bodyString: '{"ok":true}',
      elapsedTime: 123,
      bodySize: 11,
      isJson: true,
      isTruncated: false,
    };
    render(ResponseView);
    expect(screen.getByText("Body")).toBeInTheDocument();
    expect(screen.getByText("Headers")).toBeInTheDocument();
  });
});
