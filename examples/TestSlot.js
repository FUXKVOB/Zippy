import Box from "./Box.js";
import { signal, effect } from "@zippy/runtime";


export default function ZippyComponent(props) {
  const __onMount = [];
  const __onDestroy = [];
  function onMount(fn) { __onMount.push(fn); }
  function onDestroy(fn) { __onDestroy.push(fn); }

  

  const el = document.createElement('div');
  el.setAttribute('data-z-6b07d5f', '');


  el.innerHTML = `<div data-zippy-cmp="0"></div>`;

  let __cmp0;

  const __host0 = el.querySelector('[data-zippy-cmp="0"]');
  if (__host0) {
    __cmp0 = Box({ children: `<p>This is slotted content</p>` });
    __cmp0.mount(__host0);
    (__cmp0.el.querySelector('[data-zippy-slot]') || {}).innerHTML = `<p>This is slotted content</p>`;
  }


  return {
    el,
    mount(target) { target.appendChild(el); __onMount.forEach(fn => fn()); },
    unmount() { el.remove(); __onDestroy.forEach(fn => fn());
    if (__cmp0) __cmp0.unmount(); },
    update(newProps) { Object.assign(props, newProps); },
  };
}
