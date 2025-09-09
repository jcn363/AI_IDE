// Unified cancellation handling for TypeScript async operations

export class CancellationError extends Error {
  constructor(message = 'Operation cancelled') {
    super(message);
    this.name = 'CancellationError';
  }
}

export class CancellationController {
  private cancelled = false;
  private onCancel: ((reason?: string) => void) | null = null;

  cancel(reason?: string) {
    if (!this.cancelled) {
      this.cancelled = true;
      if (this.onCancel) {
        this.onCancel(reason);
      }
    }
  }

  isCancelled(): boolean {
    return this.cancelled;
  }

  onCancelled(callback: (reason?: string) => void) {
    this.onCancel = callback;
  }

  signal(): AbortSignal {
    const controller = new AbortController();
    this.onCancelled(() => controller.abort());
    return controller.signal;
  }

  child(): CancellationController {
    const child = new CancellationController();
    this.onCancelled(() => child.cancel());
    return child;
  }
}

export interface CancellableOperation<T> {
  promise: Promise<T>;
  cancel(): void;
  isCancelled(): boolean;
}

export function createCancellable<T>(
  operation: (signal: AbortSignal) => Promise<T>
): CancellableOperation<T> {
  const controller = new CancellationController();
  const signal = controller.signal();
  const promise = operation(signal);

  return {
    promise,
    cancel: () => controller.cancel(),
    isCancelled: () => controller.isCancelled()
  };
}