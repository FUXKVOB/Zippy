
import { signal, effect } from "@zippy/runtime";


export default function ZippyComponent(props) {
  const __onMount = [];
  const __onDestroy = [];
  function onMount(fn) { __onMount.push(fn); }
  function onDestroy(fn) { __onDestroy.push(fn); }

  // This component has a slot

  const el = document.createElement('div');
  el.setAttribute('data-z-b7b36509', '');
  const __style = document.createElement('style');
  __style.textContent = `[data-z-b7b36509] .box { border: 1px solid #ccc; padding: 16px; }
`;
  document.head.append(__style);

  el.innerHTML = `<div class="box"><div data-zippy-slot></div></div>`;




  return {
    el,
    mount(target) { target.appendChild(el); __onMount.forEach(fn => fn()); },
    unmount() { el.remove(); __onDestroy.forEach(fn => fn());
    __style.remove(); },
    update(newProps) { Object.assign(props, newProps); },
  };
}
