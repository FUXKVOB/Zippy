type Subscriber = () => void;

let currentEffect: Subscriber | null = null;
let currentSubs: Set<Set<Subscriber>> | null = null;

let batchDepth = 0;
let pendingEffects = new Set<Subscriber>();
let isFlushing = false;

function flushEffects() {
  isFlushing = true;
  const effects = pendingEffects;
  pendingEffects = new Set();
  for (const fn of effects) {
    fn();
  }
  isFlushing = false;
}

export class Signal<T> {
  protected _value: T;
  protected subs = new Set<Subscriber>();

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
      this.notify();
    }
  }

  protected notify() {
    if (batchDepth > 0 || isFlushing) {
      for (const fn of this.subs) pendingEffects.add(fn);
    } else {
      for (const fn of this.subs) fn();
    }
  }

  get(): T { return this.val; }
  set(next: T): void { this.val = next; }
  peek(): T { return this._value; }
}

export function signal<T>(initial: T): Signal<T> {
  return new Signal(initial);
}

class Computed<T> extends Signal<T> {
  private _dirty = true;
  private _getter: () => T;
  private _updateEffect: () => void | null = null;

  constructor(fn: () => T) {
    super(undefined as any);
    this._getter = fn;
  }

  override get val(): T {
    if (currentEffect) {
      this.subs.add(currentEffect);
      currentSubs!.add(this.subs);
    }

    if (this._dirty) {
      // 1. Ensure we have an effect to track dependencies.
      if (!this._updateEffect) {
        this._updateEffect = () => {
          this._dirty = true;
          this.notify();
        };
      }

      // 2. Manually trigger dependency tracking by wrapping the getter call
      // in the context of the internal update effect.
      const prevEffect = currentEffect;
      const prevSubs = currentSubs;
      
      currentEffect = this._updateEffect;
      currentSubs = new Set(); 
      
      // This executes the getter and registers this computed as a subscriber
      // to any signals it reads.
      this._value = this._getter();
      
      currentEffect = prevEffect;
      currentSubs = prevSubs;
      this._dirty = false;
    }
    return this._value;
  }
}

export function computed<T>(fn: () => T): Signal<T> {
  return new Computed(fn);
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
  try { 
    fn(); 
  } finally {
    batchDepth--;
    if (batchDepth === 0) {
      queueMicrotask(() => flushEffects());
    }
  }
}
