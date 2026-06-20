
import { signal, effect } from "@zippy/runtime";


export default function ZippyComponent(props) {
  const __onMount = [];
  const __onDestroy = [];
  function onMount(fn) { __onMount.push(fn); }
  function onDestroy(fn) { __onDestroy.push(fn); }

  let active = signal(false);
  const toggle = () => active.val = !active.val;

  const el = document.createElement('div');
  el.setAttribute('data-z-bbb223eb', '');
  const __style = document.createElement('style');
  __style.textContent = `[data-z-bbb223eb] .highlight { background: yellow; }
`;
  document.head.append(__style);

  el.innerHTML = `<button data-zippy-evt0 data-zippy-toggle="0"><span data-zippy-expr="0"></span></button>`;


  const __btn0 = el.querySelector('[data-zippy-evt0]');
  if (__btn0) __btn0.addEventListener('click', toggle);
  effect(() => {
    const __n = el.querySelector('[data-zippy-expr="0"]');
    if (__n) __n.textContent = active ? 'ON' : 'OFF';
  });
  effect(() => {
    const __n = el.querySelector('[data-zippy-toggle="0"]');
    if (__n) __n.classList.toggle('highlight', active.val);
  });


  return {
    el,
    mount(target) { target.appendChild(el); __onMount.forEach(fn => fn()); },
    unmount() { el.remove(); __onDestroy.forEach(fn => fn());
    __style.remove(); },
    update(newProps) { Object.assign(props, newProps); },
  };
}
