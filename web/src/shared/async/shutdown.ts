type CleanupFunction = () => void | Promise<void>;
type AsyncOperation = Promise<any>;

class GracefulShutdown {
  private shutdownHandler?: () => void;
  private cleanupFunctions: CleanupFunction[] = [];
  private activeOperations: Set<AsyncOperation> = new Set();
  private isShuttingDown = false;

  constructor() {
    // Register signal handlers
    process.on('SIGINT', () => this.initiateShutdown());
    process.on('SIGTERM', () => this.initiateShutdown());

    // Handle uncaught exceptions
    process.on('uncaughtException', (error) => {
      console.error('Uncaught Exception:', error);
      this.initiateShutdown();
    });

    // Handle unhandled promise rejections
    process.on('unhandledRejection', (reason, promise) => {
      console.error('Unhandled Rejection at:', promise, 'reason:', reason);
      this.initiateShutdown();
    });
  }

  /**
   * Register a cleanup function to be called during shutdown
   */
  registerCleanup(cleanup: CleanupFunction): void {
    this.cleanupFunctions.push(cleanup);
  }

  /**
   * Track an async operation
   */
  trackOperation(operation: AsyncOperation): void {
    if (this.isShuttingDown) {
      console.warn('Operation tracked during shutdown - may not complete');
    }
    this.activeOperations.add(operation);

    operation.finally(() => {
      this.activeOperations.delete(operation);
    });
  }

  /**
   * Check if shutdown has been initiated
   */
  isShutdown(): boolean {
    return this.isShuttingDown;
  }

  /**
   * Manually initiate shutdown
   */
  async shutdown(): Promise<void> {
    if (this.isShuttingDown) return;
    this.isShuttingDown = true;

    console.log('Initiating graceful shutdown...');

    // Stop accepting new operations
    // Call all cleanup functions
    const cleanupPromises = this.cleanupFunctions.map(async (cleanup) => {
      try {
        await cleanup();
      } catch (error) {
        console.error('Error during cleanup:', error);
      }
    });

    // Wait for all cleanup functions to complete
    await Promise.all(cleanupPromises);

    console.log(`Waiting for ${this.activeOperations.size} active operations to complete...`);

    // Wait for all active operations to complete with a timeout
    const operationsTimeout = new Promise((reject) => {
      setTimeout(() => reject(new Error('Active operations timeout')), 30000); // 30 seconds timeout
    });

    try {
      await Promise.race([Promise.all(Array.from(this.activeOperations)), operationsTimeout]);
      console.log('All operations completed');
    } catch (error) {
      console.warn('Some operations did not complete within timeout:', error);
    }

    console.log('Graceful shutdown completed');
    process.exit(0);
  }

  private initiateShutdown(): void {
    if (this.isShuttingDown) return;

    this.shutdown().catch((error) => {
      console.error('Error during shutdown:', error);
      process.exit(1);
    });
  }
}

export const shutdownManager = new GracefulShutdown();

export function onShutdown(cleanup: CleanupFunction): void {
  shutdownManager.registerCleanup(cleanup);
}

export function trackAsyncOperation(operation: AsyncOperation): void {
  shutdownManager.trackOperation(operation);
}

export function isShuttingDown(): boolean {
  return shutdownManager.isShutdown();
}

export function shutdown(): Promise<void> {
  return shutdownManager.shutdown();
}
