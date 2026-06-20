export function createElement(tag: string): HTMLElement {
  return document.createElement(tag);
}

export function setText(el: HTMLElement, text: string): void {
  el.textContent = text;
}

export function setAttr(el: HTMLElement, name: string, value: string): void {
  el.setAttribute(name, value);
}

export function listen(
  el: HTMLElement,
  event: string,
  handler: EventListener,
): () => void {
  el.addEventListener(event, handler);
  return () => el.removeEventListener(event, handler);
}

export function insertAfter(anchor: Node, node: Node): void {
  anchor.parentNode?.insertBefore(node, anchor.nextSibling);
}

export function removeAll(parent: Node): void {
  while (parent.firstChild) parent.removeChild(parent.firstChild);
}
