// Custom type declarations for DOM APIs
interface Element {
  // Basic properties
  textContent: string | null;

  // Methods
  getAttribute(name: string): string | null;
  querySelectorAll<E extends Element = Element>(selectors: string): NodeListOf<E>;
  contains(other: Node | null): boolean;
  appendChild<T extends Node>(node: T): T;
  removeChild<T extends Node>(child: T): T;
  addEventListener(
    type: string,
    listener: EventListenerOrEventListenerObject,
    options?: boolean | AddEventListenerOptions
  ): void;
  removeEventListener(
    type: string,
    listener: EventListenerOrEventListenerObject,
    options?: boolean | EventListenerOptions
  ): void;
}

interface HTMLElement extends Element {
  // Basic properties
  style: CSSStyleDeclaration;

  // Methods
  getBoundingClientRect(): DOMRect;
}

interface Document {
  body: HTMLElement;
  createElement(tagName: string, options?: ElementCreationOptions): HTMLElement;
  querySelectorAll<E extends Element = Element>(selectors: string): NodeListOf<E>;
  querySelector<E extends Element = Element>(selectors: string): E | null;
  getElementById(elementId: string): HTMLElement | null;
}

interface CSSStyleDeclaration {
  [key: string]: string | null;
}

interface Window {
  innerWidth: number;
  innerHeight: number;
  clientX: number;
  clientY: number;
}

declare const document: Document;
declare const window: Window;
