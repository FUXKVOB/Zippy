import { effect } from './reactivity.js';

export interface ZippyComponent {
  el: HTMLElement;
  mount(target: HTMLElement): void;
  unmount(): void;
  update(props: Record<string, unknown>): void;
}

export type ComponentFactory = (props: Record<string, unknown>) => ZippyComponent;

export function createComponent(
  factory: ComponentFactory,
  props: Record<string, unknown>,
  dynamicProps: [string, () => unknown][],
  slot: HTMLElement,
): ZippyComponent {
  const instance = factory(props);
  instance.mount(slot);

  for (const [key, getter] of dynamicProps) {
    effect(() => {
      instance.update({ [key]: getter() });
    });
  }

  return instance;
}
