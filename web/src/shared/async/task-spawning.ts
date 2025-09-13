// Standardized task spawning for background operations
// Eliminates inconsistencies in task management across the codebase

export interface BackgroundTask {
  id: string;
  name: string;
  promise: Promise<any>;
  abortController: AbortController;
  cleanup?: () => void | Promise<void>;
}

export class BackgroundTaskManager {
  private tasks = new Map<string, BackgroundTask>();
  private shutdownInitiated = false;

  /**
   * Spawn a background task with proper naming and cleanup
   */
  spawn<T>(
    operation: () => Promise<T>,
    name: string,
    onComplete?: (result: T) => void,
    onError?: (error: Error) => void
  ): string {
    if (this.shutdownInitiated) {
      throw new Error('Task manager is shutting down');
    }

    const id = `${name}-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
    const abortController = new AbortController();

    const taskPromise = this.executeTask(operation, name, abortController.signal);

    const task: BackgroundTask = {
      id,
      name,
      promise: taskPromise,
      abortController,
    };

    this.tasks.set(id, task);

    // Set up completion handler
    taskPromise
      .then((result) => {
        if (!this.shutdownInitiated) {
          onComplete?.(result);
        }
      })
      .catch((error) => {
        if (!this.shutdownInitiated) {
          onError?.(error);
        }
      })
      .finally(() => {
        this.tasks.delete(id);
      });

    console.debug(`[TaskManager] Spawned task: ${name} (${id})`);
    return id;
  }

  /**
   * Cancel a specific task
   */
  cancel(taskId: string): void {
    const task = this.tasks.get(taskId);
    if (task) {
      console.debug(`[TaskManager] Cancelling task: ${task.name} (${taskId})`);
      task.abortController.abort();
      if (task.cleanup) {
        Promise.resolve(task.cleanup()).catch((err: any) =>
          console.error(`[TaskManager] Cleanup failed for ${task.name}:`, err)
        );
      }
    }
  }

  /**
   * Cancel all active tasks
   */
  cancelAll(): void {
    const taskIds = Array.from(this.tasks.keys());
    taskIds.forEach((id) => this.cancel(id));
  }

  /**
   * Wait for all tasks to complete (with timeout)
   */
  async waitForAll(timeoutMs = 10000): Promise<void> {
    const activeTasks = Array.from(this.tasks.values());

    if (activeTasks.length === 0) {
      return;
    }

    console.debug(`[TaskManager] Waiting for ${activeTasks.length} tasks to complete`);

    const promises = activeTasks.map((task) => task.promise);

    await Promise.race([
      Promise.all(promises),
      new Promise<void>((_, reject) => {
        setTimeout(() => reject(new Error('Timeout waiting for tasks to complete')), timeoutMs);
      }),
    ]);
  }

  /**
   * Graceful shutdown - cancel all tasks and wait for completion
   */
  async shutdown(timeoutMs = 5000): Promise<void> {
    if (this.shutdownInitiated) {
      return;
    }

    console.debug('[TaskManager] Initiating graceful shutdown');
    this.shutdownInitiated = true;

    const activeTasks = Array.from(this.tasks.values());
    if (activeTasks.length > 0) {
      console.debug(`[TaskManager] Cancelling ${activeTasks.length} active tasks`);

      // Cancel all tasks
      this.cancelAll();

      // Wait for tasks to finish or timeout
      try {
        await this.waitForAll(timeoutMs);
        console.debug('[TaskManager] All tasks completed gracefully');
      } catch (error) {
        console.warn(`[TaskManager] Forced shutdown after timeout: ${error}`);
      }
    } else {
      console.debug('[TaskManager] No active tasks to wait for');
    }
  }

  /**
   * Get information about active tasks
   */
  getActiveTasks(): Array<{ id: string; name: string }> {
    return Array.from(this.tasks.values()).map((task) => ({
      id: task.id,
      name: task.name,
    }));
  }

  /**
   * Execute task with proper error handling and abort support
   */
  private async executeTask<T>(
    operation: () => Promise<T>,
    name: string,
    signal: AbortSignal
  ): Promise<T> {
    // Check if already cancelled
    if (signal.aborted) {
      console.debug(`[TaskManager] Task ${name} cancelled before execution`);
      throw new Error(`Task ${name} cancelled`);
    }

    try {
      const result = await operation();
      console.debug(`[TaskManager] Task ${name} completed successfully`);
      return result;
    } catch (error) {
      console.error(`[TaskManager] Task ${name} failed:`, error);
      throw error;
    }
  }
}

// Global instance for application-wide background task management
export const globalTaskManager = new BackgroundTaskManager();
