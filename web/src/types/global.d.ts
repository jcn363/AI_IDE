// Extend the Window interface to include missing browser APIs
interface Window extends globalThis.Window {
  // Add any custom window properties here
  XMLSerializer: {
    new (): XMLSerializer;
    prototype: XMLSerializer;
  };
  URL: {
    createObjectURL(blob: Blob): string;
    revokeObjectURL(url: string): void;
  };
  webkitURL: {
    createObjectURL(blob: Blob): string;
    revokeObjectURL(url: string): void;
  };
}

// Extend HTMLAnchorElement with anchor-specific properties and methods
interface HTMLAnchorElement extends HTMLElement {
  // Standard properties
  download: string;
  href: string;
  hreflang: string;
  media: string;
  referrerPolicy: string;
  rel: string;
  target: string;
  type: string;
  
  // Standard methods
  click(): void;
  toString(): string;
  
  // HTML5 extensions
  relList: DOMTokenList;
  
  // Non-standard but widely supported
  ping: string;
  text: string;
  
  // Deprecated but still in use
  charset: string;
  coords: string;
  name: string;
  rev: string;
  shape: string;
}

// Extend Document to include createEvent and other DOM methods
interface Document extends globalThis.Document {
  createEvent(eventInterface: 'MouseEvents'): MouseEvent;
}

// Extend HTMLElement to include common DOM methods
interface HTMLElement {
  // Standard DOM methods
  click(): void;
  setAttribute(name: string, value: string): void;
  getAttribute(name: string): string | null;
  removeAttribute(name: string): void;
  hasAttribute(name: string): boolean;
  
  // Style manipulation
  style: CSSStyleDeclaration;
  
  // Class manipulation
  classList: DOMTokenList;
  className: string;
  
  // Common properties
  id: string;
  title: string;
  hidden: boolean;
  tabIndex: number;
  
  // Event handlers
  addEventListener(type: string, listener: EventListenerOrEventListenerObject, options?: boolean | AddEventListenerOptions): void;
  removeEventListener(type: string, listener: EventListenerOrEventListenerObject, options?: boolean | EventListenerOptions): void;
  dispatchEvent(event: Event): boolean;
}

// CSS Modules
declare module '*.module.css' {
  const content: { [className: string]: string };
  export default content;
}
