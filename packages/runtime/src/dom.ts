export function createElement(tag: string): HTMLElement {
  return document.createElement(tag);
}

export function createComment(text: string): Comment {
  return document.createComment(text);
}

export function setText(el: HTMLElement, text: string): void {
  el.textContent = text;
}

export function clearAfter(anchor: Node): void {
  let next = anchor.nextSibling;
  while (next) {
    const toRemove = next;
    next = next.nextSibling;
    toRemove.remove();
  }
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

export function on(
  el: HTMLElement,
  event: string,
  handler: EventListener,
  cleanup: Set<() => void>,
): void {
  el.addEventListener(event, handler);
  cleanup.add(() => el.removeEventListener(event, handler));
}

export function teleportTo(target: Node, content: Node): void {
  target.appendChild(content);
}
