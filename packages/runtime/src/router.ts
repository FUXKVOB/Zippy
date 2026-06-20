import type { ComponentFactory, ZippyComponent } from './component';

interface RouteConfig {
  [path: string]: ComponentFactory;
}

export interface RouterInstance {
  mount(el: HTMLElement): void;
  unmount(): void;
  navigate(path: string, replace?: boolean): void;
  back(): void;
  forward(): void;
  current(): string;
}

function matchPath(pattern: string, path: string): { match: boolean; params: Record<string, string> } {
  const patternParts = pattern.split('/').filter(Boolean);
  const pathParts = path.split('/').filter(Boolean);
  if (patternParts.length !== pathParts.length) return { match: false, params: {} };

  const params: Record<string, string> = {};
  for (let i = 0; i < patternParts.length; i++) {
    const p = patternParts[i];
    const v = pathParts[i];
    if (p.startsWith(':')) {
      params[p.slice(1)] = decodeURIComponent(v);
    } else if (p !== v) {
      return { match: false, params: {} };
    }
  }
  return { match: true, params };
}

export function createRouter(routes: RouteConfig): RouterInstance {
  let current: ZippyComponent | null = null;
  let outlet: HTMLElement | null = null;
  let mounted = false;

  function render() {
    if (!outlet || !mounted) return;
    const path = location.pathname || '/';

    if (current) {
      current.unmount();
      current = null;
    }
    outlet.innerHTML = '';

    for (const pattern in routes) {
      const { match, params } = matchPath(pattern, path);
      if (match) {
        const factory = routes[pattern];
        current = factory({ ...params, __params: params });
        current.mount(outlet);
        return;
      }
    }

    const notFound = routes['/404'] || routes['*'];
    if (notFound) {
      current = notFound({});
      current.mount(outlet);
    }
  }

  function onPopState() {
    render();
  }

  return {
    mount(el: HTMLElement) {
      outlet = el;
      mounted = true;
      window.addEventListener('popstate', onPopState);
      render();
    },
    unmount() {
      mounted = false;
      window.removeEventListener('popstate', onPopState);
      if (current) current.unmount();
      current = null;
      outlet = null;
    },
    navigate(path: string, replace = false) {
      if (replace) {
        history.replaceState({}, '', path);
      } else {
        history.pushState({}, '', path);
      }
      render();
    },
    back() {
      history.back();
    },
    forward() {
      history.forward();
    },
    current() {
      return location.pathname || '/';
    },
  };
}
