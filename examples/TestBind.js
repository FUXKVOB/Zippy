
import { signal, effect } from "@zippy/runtime";

export default function ZippyComponent(props) {
  let name = signal("Zippy");

  const el = document.createElement('div');
  el.setAttribute('data-z-ce8f42cf', '');


  el.innerHTML = `<input data-zippy-bind="0" /><p>Hello, <span data-zippy-expr="0"></span>!</p>`;


  effect(() => {
    const __n = el.querySelector('[data-zippy-expr="0"]');
    if (__n) __n.textContent = name.val;
  });
  const __bind0 = el.querySelector('[data-zippy-bind="0"]');
  if (__bind0) {
    __bind0.value = name.val;
    __bind0.addEventListener('input', () => { name.val = __bind0.value; });
    effect(() => { __bind0.value = name.val; });
  }


  return {
    el,
    mount(target) { target.appendChild(el); },
    unmount() { el.remove(); },
    update(newProps) { Object.assign(props, newProps); },
  };
}
