import { effect } from './reactivity';

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
  const disposes: (() => void)[] = [];
  for (const [key, getter] of dynamicProps) {
    disposes.push(effect(() => {
      instance.update({ [key]: getter() });
    }));
  }
  instance.mount(slot);
  const origUnmount = instance.unmount.bind(instance);
  instance.unmount = () => {
    origUnmount();
    for (const d of disposes) d();
  };
  return instance;
}
