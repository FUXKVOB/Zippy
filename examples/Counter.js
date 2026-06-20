
import { signal, effect } from "@zippy/runtime";

export default function ZippyComponent(props) {
  let count = signal(props.start ?? 0);
  const increment = () => count.set(count.get() + 1);

  const el = document.createElement('div');
  el.setAttribute('data-z-2e55c838', '');
  const __style = document.createElement('style');
  __style.textContent = `[data-z-2e55c838] button{
    padding: 8px 16px;
    font-size: 16px;
  }
    [data-z-2e55c838] .counter{
    font-size: 24px;
    font-weight: bold;
  }
`;
  document.head.append(__style);

  el.innerHTML = `<div class="counter"><button data-zippy-evt0>Count: <span data-zippy-expr="0"></span></button></div>`;


  const __btn0 = el.querySelector('[data-zippy-evt0]');
  if (__btn0) __btn0.addEventListener('click', increment);
  effect(() => {
    const __n = el.querySelector('[data-zippy-expr="0"]');
    if (__n) __n.textContent = count.val;
  });


  return {
    el,
    mount(target) { target.appendChild(el); },
    unmount() { el.remove();
    __style.remove(); },
    update(newProps) { Object.assign(props, newProps); },
  };
}
