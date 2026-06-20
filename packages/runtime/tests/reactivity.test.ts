import { describe, it, expect } from "bun:test";
import { signal, computed, effect, batch } from "../src/reactivity";

describe("Reactivity System", () => {
  it("should update signal values", () => {
    const s = signal(0);
    s.val = 1;
    expect(s.val).toBe(1);
  });

  it("should trigger effects", () => {
    const s = signal(0);
    let count = 0;
    effect(() => {
      count = s.val;
    });
    expect(count).toBe(0);
    s.val = 1;
    expect(count).toBe(1);
  });

  it("should implement lazy computed signals", () => {
    const s = signal(10);
    let evaluations = 0;
    const c = computed(() => {
      evaluations++;
      return s.val * 2;
    });

    // Should not evaluate yet (lazy)
    expect(evaluations).toBe(0);

    // First access: evaluate
    expect(c.val).toBe(20);
    expect(evaluations).toBe(1);

    // Second access: use cached value
    expect(c.val).toBe(20);
    expect(evaluations).toBe(1);

    // Change dependency: mark dirty
    s.val = 20;
    // Should still be 1 because we haven't accessed c.val yet
    expect(evaluations).toBe(1);

    // Third access: re-evaluate
    expect(c.val).toBe(40);
    expect(evaluations).toBe(2);
  });

  it("should batch updates", async () => {
    const s1 = signal(0);
    const s2 = signal(0);
    let updates = 0;
    effect(() => {
      updates++;
      return s1.val + s2.val;
    });

    // Initial effect run
    updates = 0;

    // Update without batch: 2 updates
    s1.val = 1;
    s2.val = 1;
    expect(updates).toBe(2);

    updates = 0;

    // Update with batch: 1 update
    batch(() => {
      s1.val = 2;
      s2.val = 2;
    });
    
    // Wait for microtasks to flush
    await new Promise(resolve => queueMicrotask(resolve));
    
    expect(updates).toBe(1);
  });
});
