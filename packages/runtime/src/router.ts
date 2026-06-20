import type { ComponentFactory, ZippyComponent } from './component';

interface RouteConfig {
  [path: string]: ComponentFactory;
}

export function createRouter(routes: RouteConfig) {
  let current: ZippyComponent | null = null;
  let outlet: HTMLElement | null = null;

  function render() {
    const hash = location.hash.slice(1) || "/";
    if (current) current.unmount();
    if (!outlet) return;
    outlet.innerHTML = "";
    const factory = routes[hash];
    if (factory) {
      current = factory({});
      current.mount(outlet);
    }
  }

  const onHash = () => render();

  return {
    mount(el: HTMLElement) {
      outlet = el;
      window.addEventListener("hashchange", onHash);
      render();
    },
    unmount() {
      window.removeEventListener("hashchange", onHash);
      if (current) current.unmount();
      current = null;
      outlet = null;
    },
  };
}
