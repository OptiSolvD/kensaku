// @vitest-environment jsdom
import { act } from "react";
import { createRoot } from "react-dom/client";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { useMeasuredHeight } from "./useMeasuredHeight";

/**
 * Height measurement for the search bar.
 *
 * The bar floats over the results and the results are padded by its measured
 * height. 1.0.16 measured that height from an effect with an empty dependency
 * array: the bar is not in the tree on the first render (the setup spinner is),
 * so the effect found no node, returned early, and never ran again. The padding
 * stuck at its 88px default while the bar grew to about 200px once results were
 * on screen, and the first row of thumbnails rendered underneath it.
 *
 * The hook is exercised through a real mounted component, so a regression fails
 * here rather than in someone's window.
 */

class FakeObserver {
  static all: FakeObserver[] = [];
  callback: () => void;
  observed: Element[] = [];
  constructor(callback: () => void) {
    this.callback = callback;
    FakeObserver.all.push(this);
  }
  observe(node: Element) {
    this.observed.push(node);
  }
  disconnect() {
    this.observed = [];
  }
  fire() {
    this.callback();
  }
}

function setHeight(node: HTMLElement, px: number) {
  Object.defineProperty(node, "offsetHeight", { value: px, configurable: true });
}

function Harness({
  fallback,
  onHeight,
}: {
  fallback: number;
  onHeight: (h: number) => void;
}) {
  const [height, ref] = useMeasuredHeight<HTMLDivElement>(fallback);
  onHeight(height);
  return <div ref={ref} id="bar" />;
}

describe("useMeasuredHeight", () => {
  beforeEach(() => {
    FakeObserver.all = [];
    vi.stubGlobal("ResizeObserver", FakeObserver);
  });

  afterEach(() => {
    vi.unstubAllGlobals();
    document.body.innerHTML = "";
  });

  function mount(fallback: number) {
    const host = document.createElement("div");
    document.body.appendChild(host);
    const root = createRoot(host);
    const seen: number[] = [];
    const paint = () => root.render(<Harness fallback={fallback} onHeight={(h) => seen.push(h)} />);

    act(() => paint());

    const node = host.querySelector("#bar") as HTMLElement;
    const latest = () => seen[seen.length - 1];
    const lastObserver = () => FakeObserver.all[FakeObserver.all.length - 1];

    return {
      node,
      seen,
      latest,
      lastObserver,
      repaint: () => act(() => paint()),
      unmount: () => {
        act(() => root.unmount());
        host.remove();
      },
    };
  }

  it("starts at the fallback and measures the node once it attaches", () => {
    const view = mount(88);
    // The value after the callback ref fired: the node existed but had no
    // height set yet, so it measured whatever jsdom reports (0).
    expect(view.seen[0]).toBe(88);

    setHeight(view.node, 214);
    act(() => view.lastObserver().fire());
    expect(view.latest()).toBe(214);
    view.unmount();
  });

  it("follows the bar as it grows when results arrive", () => {
    const view = mount(88);
    setHeight(view.node, 74);
    view.repaint();
    act(() => view.lastObserver().fire());
    expect(view.latest()).toBe(74);

    // Results arrive: interpretation row, chips and toolbar appear.
    setHeight(view.node, 214);
    act(() => view.lastObserver().fire());
    expect(view.latest()).toBe(214);

    // Query cleared: the bar shrinks back.
    setHeight(view.node, 74);
    act(() => view.lastObserver().fire());
    expect(view.latest()).toBe(74);

    view.unmount();
  });

  it("detaches the observer on unmount", () => {
    const view = mount(88);
    const observer = view.lastObserver();
    view.unmount();
    expect(observer.observed).toHaveLength(0);
  });
});
