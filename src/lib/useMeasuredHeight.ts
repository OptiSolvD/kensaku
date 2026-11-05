import { useCallback, useEffect, useRef, useState } from "react";

/**
 * Measures an element's height and keeps the value current.
 *
 * Exists because the naive version of this broke the layout: an effect with an
 * empty dependency array measured on mount, found no node (the element was not
 * rendered yet), gave up, and never measured again. The caller's padding stayed
 * at its default while the real element was three times taller, so content
 * rendered underneath the floating search bar.
 *
 * A callback ref is the right primitive: it fires when the node attaches, not
 * when the component mounts, and the ResizeObserver keeps the value honest as
 * the element grows and shrinks afterwards.
 *
 * `fallback` is returned until the first measurement.
 */
export function useMeasuredHeight<T extends HTMLElement>(
  fallback: number,
): [number, (node: T | null) => void] {
  const [height, setHeight] = useState(fallback);
  const observer = useRef<ResizeObserver | null>(null);

  const ref = useCallback((node: T | null) => {
    observer.current?.disconnect();
    observer.current = null;
    if (!node) return;

    const measure = () => setHeight(node.offsetHeight);
    // ResizeObserver is absent in some embedded runtimes; a one-off measurement
    // is better than none.
    if (typeof ResizeObserver === "undefined") {
      measure();
      return;
    }

    const instance = new ResizeObserver(measure);
    instance.observe(node);
    observer.current = instance;
    measure();
  }, []);

  useEffect(() => () => observer.current?.disconnect(), []);

  return [height, ref];
}
