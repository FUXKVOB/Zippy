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
): () => void {
  const markers = new Map<string | number, { el: HTMLElement; key: string | number }>();
  let updateFn: (() => void) | null = null;

  function render() {
    const keys = new Set<string | number>();
    const frag = document.createDocumentFragment();
    const newMarkers = new Map<string | number, { el: HTMLElement; key: string | number }>();

    for (let i = 0; i < items.length; i++) {
      const item = items[i];
      const key = keyFn(item, i);

      const existing = markers.get(key);
      if (existing) {
        markers.delete(key);
        existing.el.setAttribute('data-each-idx', String(i));
        frag.appendChild(existing.el);
        newMarkers.set(key, existing);
      } else {
        const el = createFn({ key, item, index: i });
        el.setAttribute('data-each-idx', String(i));
        frag.appendChild(el);
        newMarkers.set(key, { el, key });
      }
      keys.add(key);
    }

    // Remove stale
    for (const [, marker] of markers) {
      marker.el.remove();
    }

    container.replaceChildren(frag);
    markers.clear();
    for (const [k, v] of newMarkers) markers.set(k, v);
  }

  render();
  updateFn = render;
  return () => { updateFn = null; for (const [, m] of markers) m.el.remove(); markers.clear(); };
}
