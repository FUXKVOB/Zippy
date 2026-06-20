
import { signal, effect, on, reconcileEach } from "@zippy/runtime";

export default function ZippyComponent(props) {
  const __onMount = [];
  const __onDestroy = [];
  function onMount(fn) { __onMount.push(fn); }
  function onDestroy(fn) { __onDestroy.push(fn); }

  let items = signal(["a", "b", "c"]);
  const addItem = () => items.val = [...items.val, "x"];

  const el = document.createElement('div');
  el.setAttribute('data-z-57b91740', '');


  el.innerHTML = `<button data-zippy-evt0>Add</button><ul><!--zippy-each-0--><div data-zippy-each="0"></div></ul>`;

  let __each0;

  const __btn0 = el.querySelector('[data-zippy-evt0]');
  if (__btn0) on(__btn0, 'click', addItem, __onDestroy);
  let __eachDispose0;
  effect(() => {
    if (__eachDispose0) __eachDispose0();
    const __c0 = el.querySelector('[data-zippy-each="0"]');
    if (!__c0) return;
    __eachDispose0 = reconcileEach(__c0, items.val, (item, i) => i, ({ item: item, index: i }) => {
    const __e = document.createElement('div');
    __e.innerHTML = `<li>${i}: ${item}</li>`;
    return __e.firstElementChild || __e;
    });
  });
  onDestroy(() => { if (__eachDispose0) __eachDispose0(); });


  return {
    el,
    mount(target) { target.appendChild(el); __onMount.forEach(fn => fn()); },
    unmount() { el.remove(); __onDestroy.forEach(fn => fn()); },
    update(newProps) { Object.assign(props, newProps); },
  };
}
