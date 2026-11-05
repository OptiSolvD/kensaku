// @vitest-environment jsdom
import { act } from "react";
import { createRoot } from "react-dom/client";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

/* Mounting the sort control must never move focus.
 *
 * The control is rendered only once a query has results, so it mounts on the
 * first keystroke. A focus() on mount therefore lands between character one
 * and two: the first character is in the box, then the caret is on the sort
 * button and everything typed after it goes nowhere. That shipped in 1.0.16
 * and this test exists so it cannot come back. */

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

import { ResultControls } from "./ResultControls";

function renderControls() {
  const container = document.createElement("div");
  document.body.appendChild(container);
  const root = createRoot(container);

  act(() => {
    root.render(
      <div>
        <input id="search" />
        <ResultControls
          sort="best"
          onSort={() => undefined}
          view="grid"
          onView={() => undefined}
          onSave={() => undefined}
          canSave={true}
        />
      </div>,
    );
  });

  const cleanup = () => {
    act(() => root.unmount());
    container.remove();
  };
  return { container, cleanup };
}

describe("ResultControls", () => {
  beforeEach(() => {
    document.body.innerHTML = "";
  });

  afterEach(() => {
    document.body.innerHTML = "";
  });

  it("does not take focus when it mounts", () => {
    const search = document.createElement("input");
    search.id = "search-before";
    document.body.appendChild(search);
    search.focus();
    expect(document.activeElement).toBe(search);

    const { cleanup } = renderControls();

    expect(document.activeElement).toBe(search);
    cleanup();
  });

  it("keeps focus in the search box across a re-render", () => {
    const search = document.createElement("input");
    document.body.appendChild(search);
    search.focus();

    const host = document.createElement("div");
    document.body.appendChild(host);
    const root = createRoot(host);

    const render = (view: "grid" | "list") =>
      act(() => {
        root.render(
          <div>
            <input id="search" />
            <ResultControls
              sort="best"
              onSort={() => undefined}
              view={view}
              onView={() => undefined}
              onSave={() => undefined}
              canSave={true}
            />
          </div>,
        );
      });

    render("grid");
    const input = host.querySelector("input") as HTMLInputElement;
    input.focus();
    expect(document.activeElement).toBe(input);

    render("list");
    expect(document.activeElement).toBe(input);

    act(() => root.unmount());
    host.remove();
  });

  it("returns focus to the trigger only after the list closes", () => {
    const { container, cleanup } = renderControls();

    const trigger = container.querySelector(
      'button[aria-haspopup="listbox"]',
    ) as HTMLButtonElement;
    expect(trigger).toBeTruthy();

    act(() => trigger.click());
    expect(trigger.getAttribute("aria-expanded")).toBe("true");

    // Escape closes it, and focus should land back on the trigger.
    const list = container.querySelector('[role="listbox"]') as HTMLElement;
    act(() => {
      list.dispatchEvent(
        new KeyboardEvent("keydown", { key: "Escape", bubbles: true }),
      );
    });

    expect(trigger.getAttribute("aria-expanded")).toBe("false");
    expect(document.activeElement).toBe(trigger);

    cleanup();
  });
});
