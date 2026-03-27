import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/svelte";
import EmptyState from "./EmptyState.svelte";

describe("EmptyState", () => {
  it("renders title and message", () => {
    render(EmptyState, { props: { title: "No Data", message: "Nothing here" } });
    expect(screen.getByText("No Data")).toBeInTheDocument();
    expect(screen.getByText("Nothing here")).toBeInTheDocument();
  });

  it("renders default icon", () => {
    const { container } = render(EmptyState, { props: { title: "Test", message: "msg" } });
    const icon = container.querySelector(".icon");
    expect(icon).toBeInTheDocument();
    expect(icon?.textContent).toContain("📭");
  });

  it("renders custom icon", () => {
    const { container } = render(EmptyState, { props: { title: "Test", message: "msg", icon: "🔥" } });
    const icon = container.querySelector(".icon");
    expect(icon?.textContent).toContain("🔥");
  });
});
