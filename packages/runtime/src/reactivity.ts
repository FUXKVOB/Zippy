type Subscriber = () => void;

let currentEffect: Subscriber | null = null;
const cleanupStack: (() => void)[] = [];

export class Signal<T> {
  private _value: T;
  private subs = new Set<Subscriber>();

  constructor(initial: T) {
    this._value = initial;
  }

  get val(): T {
    if (currentEffect) this.subs.add(currentEffect);
    return this._value;
  }

  set val(next: T) {
    if (next !== this._value) {
      this._value = next;
      for (const fn of this.subs) fn();
    }
  }

  get(): T {
    return this.val;
  }

  set(next: T): void {
    this.val = next;
  }

  peek(): T {
    return this._value;
  }
}

export function signal<T>(initial: T): Signal<T> {
  return new Signal(initial);
}

export function computed<T>(fn: () => T): Signal<T> {
  const s = new Signal(undefined as T);
  effect(() => { s.val = fn(); });
  return s;
}

export function effect(fn: () => void): () => void {
  const prev = currentEffect;
  currentEffect = fn;
  const cleanup = fn();
  currentEffect = prev;
  if (typeof cleanup === 'function') cleanupStack.push(cleanup);
  return () => {};
}

export function batch(fn: () => void): void {
  fn();
}
