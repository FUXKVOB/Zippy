
import { signal, effect } from "@zippy/runtime";


export default function ZippyComponent(props) {
  const __onMount = [];
  const __onDestroy = [];
  function onMount(fn) { __onMount.push(fn); }
  function onDestroy(fn) { __onDestroy.push(fn); }

  let show = signal(true);
  const toggle = () => show.val = !show.val;

  const el = document.createElement('div');
  el.setAttribute('data-z-ec6f5aed', '');


  el.innerHTML = `<button data-zippy-evt0>Toggle</button><!--zippy-if-0--><div data-zippy-if="0"><div data-zippy-if-true="0"><p>Visible</p></div><div data-zippy-if-false="0"><p>Hidden</p></div></div>`;

  let __if0;

  const __btn0 = el.querySelector('[data-zippy-evt0]');
  if (__btn0) __btn0.addEventListener('click', toggle);
  effect(() => {
    const __p = el.querySelector('[data-zippy-if="0"]');
    if (!__p) return;
    const __t = __p.querySelector('[data-zippy-if-true="0"]');
    const __f = __p.querySelector('[data-zippy-if-false="0"]');
    if (__t) __t.hidden = !(show.val);
    if (__f) __f.hidden = !!(show.val);
  });


  return {
    el,
    mount(target) { target.appendChild(el); __onMount.forEach(fn => fn()); },
    unmount() { el.remove(); __onDestroy.forEach(fn => fn()); },
    update(newProps) { Object.assign(props, newProps); },
  };
}
