import Counter from "./Counter.js";
import { signal, effect } from "@zippy/runtime";

export default function ZippyComponent(props) {
    const name = signal("Zippy");

  const items = signal(["a", "b", "c"]);
  const show = signal(true);

  const el = document.createElement('div');
  el.setAttribute('data-z-ddc7aa89', '');
  const __style = document.createElement('style');
  __style.textContent = `[data-z-ddc7aa89] h1 { color: #42b883; }
`;
  document.head.append(__style);

  el.innerHTML = `<div><h1>Hello, <span data-zippy-expr="0"></span>!</h1><div data-zippy-cmp="0"></div><!--zippy-if-0--><div data-zippy-if="0"><p>Visible section</p></div><ul><!--zippy-each-0--><div data-zippy-each="0"></div></ul></div>`;

  let __cmp0;
  let __if0;
  let __each0;

  effect(() => {
    const __n = el.querySelector('[data-zippy-expr="0"]');
    if (__n) __n.textContent = name.val;
  });
  const __slot0 = el.querySelector('[data-zippy-cmp="0"]');
  if (__slot0) {
    __cmp0 = Counter({ start: 5 });
    __cmp0.mount(__slot0);
  }
  effect(() => {
    if (__cmp0) __cmp0.update({ start: 5 });
  });
  const __ifAnchor0 = el.querySelector('[data-zippy-if="0"]');
  if (__ifAnchor0) {
    effect(() => {
      __ifAnchor0.hidden = !(show.val);
    });
  }
  const __each_0 = () => {
    const __list = items.val;
    return __list.map((item) => `<li>${item.val}</li>`).join('');
  };
  effect(() => {
    const __parent = el.querySelector('[data-zippy-each="0"]');
    if (!__parent) return;
    __parent.innerHTML = __each_0();
  });


  return {
    el,
    mount(target) { target.appendChild(el); },
    unmount() { el.remove();
    if (__cmp0) __cmp0.unmount();
    __style.remove(); },
    update(newProps) { Object.assign(props, newProps); },
  };
}
