export interface EachEntry<T> {
  key: string | number;
  item: T;
  index: number;
}

export function reconcileEach<T>(
  container: HTMLElement,
  items: T[],
  keyFn: (item: T, index: number) => string | number,
  createFn: (entry: EachEntry<T>) => HTMLElement,
  initFn?: (el: HTMLElement, entry: EachEntry<T>) => (() => void) | void,
): () => void {
  const markers = new Map<string | number, { el: HTMLElement; key: string | number; destroy?: () => void }>();
  let updateFn: (() => void) | null = null;

  function render() {
    const keys = new Set<string | number>();
    const frag = document.createDocumentFragment();
    const newMarkers = new Map<string | number, { el: HTMLElement; key: string | number; destroy?: () => void }>();

    for (let i = 0; i < items.length; i++) {
      const item = items[i];
      const key = keyFn(item, i);
      const entry = { key, item, index: i };

      const existing = markers.get(key);
      if (existing) {
        markers.delete(key);
        existing.el.setAttribute('data-each-idx', String(i));
        frag.appendChild(existing.el);
        newMarkers.set(key, existing);
      } else {
        const el = createFn(entry);
        el.setAttribute('data-each-idx', String(i));
        const destroy = initFn?.(el, entry) as (() => void) | undefined;
        frag.appendChild(el);
        newMarkers.set(key, { el, key, destroy });
      }
      keys.add(key);
    }

    // Remove stale — call per-item destroy
    for (const [, marker] of markers) {
      marker.destroy?.();
      marker.el.remove();
    }

    container.replaceChildren(frag);
    markers.clear();
    for (const [k, v] of newMarkers) markers.set(k, v);
  }

  render();
  updateFn = render;
  return () => {
    updateFn = null;
    for (const [, m] of markers) { m.destroy?.(); m.el.remove(); }
    markers.clear();
  };
}
