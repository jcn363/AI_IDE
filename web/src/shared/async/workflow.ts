import { performance } from 'node:perf_hooks';

interface WorkflowTask<T = any, R = any> {
  id: string;
  name: string;
  execute: (input: T) => Promise<R>;
  rollback?: (input: T) => Promise<void>;
  dependsOn?: string[];
  retryAttempts?: number;
  retryDelay?: number;
}

interface WorkflowState {
  id: string;
  name: string;
  status: 'pending' | 'running' | 'completed' | 'failed' | 'rollback' | 'retrying';
  input?: any;
  result?: any;
  error?: Error;
  startTime?: number;
  endTime?: number;
  retryCount: number;
  maxRetries: number;
}

interface WorkflowOptions {
  maxConcurrency?: number;
  enableRetry?: boolean;
  enableRollback?: boolean;
  timeout?: number;
  onStateChange?: (state: WorkflowState) => void;
  onError?: (error: Error, task: WorkflowTask) => void;
  onRecovery?: (task: WorkflowTask) => void;
}

interface WorkflowResult<T = any> {
  success: boolean;
  result?: T;
  error?: Error;
  duration: number;
  taskStates: Map<string, WorkflowState>;
}

/**
 * Workflow execution engine with task chaining, error recovery, and state tracking
 */
export class WorkflowEngine<T = any> {
  private tasks = new Map<string, WorkflowTask>();
  private state = new Map<string, WorkflowState>();
  private options: WorkflowOptions;
  private isRunning = false;

  constructor(options: WorkflowOptions = {}) {
    this.options = {
      maxConcurrency: 1,
      enableRetry: true,
      enableRollback: true,
      timeout: 300000, // 5 minutes
      ...options,
    };
  }

  /**
   * Add a task to the workflow
   */
  addTask(task: WorkflowTask): void {
    if (this.isRunning) {
      throw new Error('Cannot add tasks while workflow is running');
    }

    this.tasks.set(task.id, task);

    // Initialize task state
    const state: WorkflowState = {
      id: task.id,
      name: task.name,
      status: 'pending',
      retryCount: 0,
      maxRetries: task.retryAttempts || 3,
    };

    this.state.set(task.id, state);
  }

  /**
   * Add multiple tasks to the workflow
   */
  addTasks(tasks: WorkflowTask[]): void {
    tasks.forEach((task) => this.addTask(task));
  }

  /**
   * Execute the workflow
   */
  async execute(input?: T): Promise<WorkflowResult> {
    if (this.isRunning) {
      throw new Error('Workflow is already running');
    }

    this.isRunning = true;
    const startTime = performance.now();

    try {
      await this.validateWorkflow();

      const executionOrder = this.calculateExecutionOrder();
      const results = new Map<string, any>();

      for (const taskId of executionOrder) {
        const task = this.tasks.get(taskId)!;
        const taskState = this.state.get(taskId)!;

        // Wait for dependencies to complete
        await this.waitForDependencies(taskId);

        // Execute task
        const taskInput = this.resolveTaskInput(taskId, results, input);

        try {
          const result = await this.executeTask(task, taskInput, taskState);
          results.set(taskId, result);

          this.notifyStateChange(taskState);
        } catch (error) {
          const taskError = error instanceof Error ? error : new Error(String(error));

          if (this.options.enableRetry && taskState.retryCount < taskState.maxRetries) {
            await this.retryTask(task, taskInput, taskState);
          } else {
            await this.handleTaskFailure(task, taskError, taskState);
            throw taskError;
          }
        }
      }

      return {
        success: true,
        result: results.get(this.getTerminalTask()), // Return result of terminal task
        duration: performance.now() - startTime,
        taskStates: this.state,
      };
    } catch (error) {
      const workflowError = error instanceof Error ? error : new Error(String(error));

      const result: WorkflowResult = {
        success: false,
        error: workflowError,
        duration: performance.now() - startTime,
        taskStates: this.state,
      };

      this.notifyStateChange({
        id: 'workflow',
        name: 'workflow',
        status: 'failed',
        retryCount: 0,
        maxRetries: 0,
        error: workflowError,
      });

      return result;
    } finally {
      this.isRunning = false;
    }
  }

  /**
   * Get workflow state
   */
  getWorkflowState(): Map<string, WorkflowState> {
    return new Map(this.state);
  }

  /**
   * Get state of specific task
   */
  getTaskState(taskId: string): WorkflowState | null {
    return this.state.get(taskId) || null;
  }

  /**
   * Stop workflow execution
   */
  async stop(): Promise<void> {
    this.isRunning = false;
    // Note: In a real implementation, you'd need to handle ongoing task cancellation
  }

  private async validateWorkflow(): Promise<void> {
    // Validate task dependencies exist
    for (const task of this.tasks.values()) {
      if (task.dependsOn) {
        for (const depId of task.dependsOn) {
          if (!this.tasks.has(depId)) {
            throw new Error(`Task ${task.id} depends on non-existent task ${depId}`);
          }
        }
      }
    }

    // Detect circular dependencies
    const visited = new Set<string>();
    const recursionStack = new Set<string>();

    const hasCycle = (taskId: string): boolean => {
      visited.add(taskId);
      recursionStack.add(taskId);

      const task = this.tasks.get(taskId)!;
      if (task.dependsOn) {
        for (const depId of task.dependsOn) {
          if (!visited.has(depId) && hasCycle(depId)) {
            return true;
          } else if (recursionStack.has(depId)) {
            return true;
          }
        }
      }

      recursionStack.delete(taskId);
      return false;
    };

    for (const taskId of this.tasks.keys()) {
      if (!visited.has(taskId) && hasCycle(taskId)) {
        throw new Error(`Circular dependency detected in workflow`);
      }
    }
  }

  private calculateExecutionOrder(): string[] {
    const executionOrder: string[] = [];
    const visited = new Set<string>();

    const visit = (taskId: string) => {
      if (visited.has(taskId)) return;
      visited.add(taskId);

      const task = this.tasks.get(taskId)!;
      if (task.dependsOn) {
        for (const depId of task.dependsOn) {
          visit(depId);
        }
      }

      executionOrder.push(taskId);
    };

    for (const taskId of this.tasks.keys()) {
      visit(taskId);
    }

    return executionOrder;
  }

  private async waitForDependencies(taskId: string): Promise<void> {
    const task = this.tasks.get(taskId)!;

    if (!task.dependsOn) return;

    const promises = task.dependsOn.map((depId) => {
      return new Promise<void>((resolve, reject) => {
        const checkDependency = () => {
          const depState = this.state.get(depId)!;
          if (depState.status === 'completed') {
            resolve();
          } else if (depState.status === 'failed') {
            reject(new Error(`Dependency ${depId} failed`));
          } else {
            setTimeout(checkDependency, 100); // Poll every 100ms
          }
        };
        checkDependency();
      });
    });

    await Promise.all(promises);
  }

  private resolveTaskInput(taskId: string, results: Map<string, any>, initialInput?: T): any {
    const task = this.tasks.get(taskId)!;

    if (!task.dependsOn || task.dependsOn.length === 0) {
      return initialInput;
    }

    // Return results from dependencies
    if (task.dependsOn.length === 1) {
      return results.get(task.dependsOn[0]);
    }

    // Return array of dependency results
    return task.dependsOn.map((depId) => results.get(depId));
  }

  private async executeTask(task: WorkflowTask, input: any, state: WorkflowState): Promise<any> {
    state.status = 'running';
    state.startTime = performance.now();
    state.input = input;

    this.notifyStateChange(state);

    try {
      const result = await this.withTimeout(task.execute(input), this.options.timeout!);

      state.status = 'completed';
      state.result = result;
      state.endTime = performance.now();

      this.notifyStateChange(state);

      return result;
    } catch (error) {
      state.status = 'failed';
      state.error = error instanceof Error ? error : new Error(String(error));
      state.endTime = performance.now();

      this.notifyStateChange(state);
      throw error;
    }
  }

  private async retryTask(task: WorkflowTask, input: any, state: WorkflowState): Promise<any> {
    state.status = 'retrying';
    state.retryCount++;
    state.error = undefined;
    state.result = undefined;

    this.notifyStateChange(state);

    if (this.options.onRecovery) {
      this.options.onRecovery(task);
    }

    // Exponential backoff
    const delay = task.retryDelay || Math.pow(2, state.retryCount) * 1000;
    await this.delay(delay);

    return this.executeTask(task, input, state);
  }

  private async handleTaskFailure(
    task: WorkflowTask,
    error: Error,
    state: WorkflowState
  ): Promise<void> {
    if (this.options.onError) {
      this.options.onError(error, task);
    }

    if (this.options.enableRollback && task.rollback) {
      try {
        state.status = 'rollback';
        this.notifyStateChange(state);

        await task.rollback(state.input);
      } catch (rollbackError) {
        console.error(`Rollback failed for task ${task.id}:`, rollbackError);
      }
    }
  }

  private getTerminalTask(): string {
    // Return task with no dependents (terminal task)
    const hasDependents = new Set<string>();

    for (const task of this.tasks.values()) {
      if (task.dependsOn) {
        for (const depId of task.dependsOn) {
          hasDependents.add(depId);
        }
      }
    }

    for (const taskId of this.tasks.keys()) {
      if (!hasDependents.has(taskId)) {
        return taskId;
      }
    }

    // If no clear terminal task, return the last task added
    return Array.from(this.tasks.keys()).pop()!;
  }

  private notifyStateChange(state: WorkflowState): void {
    if (this.options.onStateChange) {
      this.options.onStateChange(state);
    }
  }

  private withTimeout<T>(promise: Promise<T>, timeout: number): Promise<T> {
    return new Promise((resolve, reject) => {
      const timer = setTimeout(() => {
        reject(new Error(`Task timed out after ${timeout}ms`));
      }, timeout);

      promise
        .then(resolve)
        .catch(reject)
        .finally(() => clearTimeout(timer));
    });
  }

  private delay(ms: number): Promise<void> {
    return new Promise((resolve) => setTimeout(resolve, ms));
  }
}

/**
 * Create a simple workflow from a task chain
 */
export function createWorkflow<T>(
  name: string,
  tasks: Array<Omit<WorkflowTask, 'id'>>,
  options?: WorkflowOptions
): WorkflowEngine<T> {
  const workflow = new WorkflowEngine<T>(options);

  // Set up dependencies automatically for simple chaining
  for (let i = 0; i < tasks.length; i++) {
    const task = tasks[i];
    const taskId = `task-${i}`;

    workflow.addTask({
      ...task,
      id: taskId,
      dependsOn: i > 0 ? [`task-${i - 1}`] : undefined,
    });
  }

  return workflow;
}

export type { WorkflowTask, WorkflowState, WorkflowOptions, WorkflowResult };
