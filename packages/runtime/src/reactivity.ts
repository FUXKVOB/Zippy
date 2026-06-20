type Subscriber = () => void;

let currentEffect: Subscriber | null = null;
let currentSubs: Set<Set<Subscriber>> | null = null;

let batchDepth = 0;
let pendingEffects = new Set<Subscriber>();

export class Signal<T> {
  private _value: T;
  private subs = new Set<Subscriber>();

  constructor(initial: T) {
    this._value = initial;
  }

  get val(): T {
    if (currentEffect) {
      this.subs.add(currentEffect);
      currentSubs!.add(this.subs);
    }
    return this._value;
  }

  set val(next: T) {
    if (next !== this._value) {
      this._value = next;
      if (batchDepth > 0) {
        for (const fn of this.subs) pendingEffects.add(fn);
      } else {
        for (const fn of this.subs) fn();
      }
    }
  }

  get(): T { return this.val; }
  set(next: T): void { this.val = next; }
  peek(): T { return this._value; }
}

export function signal<T>(initial: T): Signal<T> {
  return new Signal(initial);
}

export function computed<T>(fn: () => T): Signal<T> {
  const s = signal(fn());
  effect(() => { s.val = fn(); });
  return s;
}

export function effect(fn: () => void): () => void {
  const prev = currentEffect;
  const prevSubs = currentSubs;
  currentEffect = fn;
  currentSubs = new Set();
  fn();
  currentEffect = prev;
  const mySubs = currentSubs;
  currentSubs = prevSubs;
  return () => {
    for (const s of mySubs) s.delete(fn);
    mySubs.clear();
  };
}

export function batch(fn: () => void): void {
  batchDepth++;
  try { fn(); }
  finally {
    batchDepth--;
    if (batchDepth === 0) {
      const pending = pendingEffects;
      pendingEffects = new Set();
      for (const fn of pending) fn();
    }
  }
}
