import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, cleanup, fireEvent } from "@testing-library/svelte";
import type { RequestCollection } from "../../lib/types";

vi.mock("../../lib/stores/app.svelte", () => ({
  appStore: {
    selectedRequestId: null as string | null,
    selectRequest: vi.fn(),
    reorderRequest: vi.fn(),
  },
}));

vi.mock("../../lib/stores/editor.svelte", () => ({
  editorStore: {
    loadFrom: vi.fn(),
  },
}));

// Mock RequestRow child component
function createMockComponent() {
  function MockComponent() {}
  (MockComponent as any).$$typeof = Symbol.for("svelte.component");
  return MockComponent;
}
vi.mock("./RequestRow.svelte", () => ({ default: createMockComponent() }));

import { appStore } from "../../lib/stores/app.svelte";
import CollectionItem from "./CollectionItem.svelte";

function makeCollection(overrides: Partial<RequestCollection> = {}): RequestCollection {
  return {
    id: "col-1",
    name: "My Collection",
    requests: [
      {
        id: "req-1",
        name: "Get Users",
        method: "GET",
        url: "https://api.example.com/users",
        query_params: [],
        headers: [],
        body: { type: "none" },
        auth: { type: "none" },
        sort_order: 0,
        created_at: "2024-01-01",
        updated_at: "2024-01-01",
      },
      {
        id: "req-2",
        name: "Create User",
        method: "POST",
        url: "https://api.example.com/users",
        query_params: [],
        headers: [],
        body: { type: "json", content: "{}" },
        auth: { type: "none" },
        sort_order: 1,
        created_at: "2024-01-01",
        updated_at: "2024-01-01",
      },
    ],
    ...overrides,
  } as RequestCollection;
}

describe("CollectionItem", () => {
  beforeEach(() => {
    cleanup();
    vi.clearAllMocks();
    (appStore as any).selectedRequestId = null;
  });

  it("renders collection name and request count", () => {
    const collection = makeCollection();
    render(CollectionItem, { props: { collection } });
    expect(screen.getByText("My Collection")).toBeInTheDocument();
    expect(screen.getByText("2")).toBeInTheDocument();
  });

  it("starts expanded by default", () => {
    const collection = makeCollection();
    const { container } = render(CollectionItem, { props: { collection } });
    expect(container.querySelector(".requests")).toBeInTheDocument();
  });

  it("collapses when header is clicked", async () => {
    const collection = makeCollection();
    const { container } = render(CollectionItem, { props: { collection } });

    const header = container.querySelector(".header")!;
    await fireEvent.click(header);

    expect(container.querySelector(".requests")).not.toBeInTheDocument();
  });

  it("expands again when header is clicked twice", async () => {
    const collection = makeCollection();
    const { container } = render(CollectionItem, { props: { collection } });

    const header = container.querySelector(".header")!;
    await fireEvent.click(header);
    await fireEvent.click(header);

    expect(container.querySelector(".requests")).toBeInTheDocument();
  });

  it("renders correct number of drag-wrapper items", () => {
    const collection = makeCollection();
    const { container } = render(CollectionItem, { props: { collection } });
    const wrappers = container.querySelectorAll(".drag-wrapper");
    expect(wrappers).toHaveLength(2);
  });

  it("shows empty requests area for empty collection", () => {
    const collection = makeCollection({ requests: [] });
    const { container } = render(CollectionItem, { props: { collection } });
    expect(screen.getByText("0")).toBeInTheDocument();
    const wrappers = container.querySelectorAll(".drag-wrapper");
    expect(wrappers).toHaveLength(0);
  });
});
