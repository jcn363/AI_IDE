use crate::error::TestError;
use std::future::Future;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::timeout;

/// Executes a future with a timeout
pub async fn with_timeout<T>(
    future: impl Future<Output = T>,
    duration: Duration,
) -> Result<T, TestError> {
    timeout(duration, future)
        .await
        .map_err(|_| TestError::Timeout(format!("Operation timed out after {:?}", duration)))
}

/// Runs a task with timeout, returning the result or error
pub async fn timeout_task<T>(
    task: impl Future<Output = Result<T, TestError>> + Send + 'static,
    duration: Duration,
) -> Result<T, TestError> {
    with_timeout(task, duration).await?
}

/// Runs multiple tasks concurrently and returns the first successful result
pub async fn run_concurrent<T, Fut>(futures: Vec<Fut>) -> Result<T, TestError>
where
    Fut: Future<Output = Result<T, TestError>> + Send + 'static,
    T: Send + 'static,
{
    if futures.is_empty() {
        return Err(TestError::Async("No tasks provided".to_string()));
    }

    // Create a stream of all futures
    let results: Vec<Result<T, TestError>> = futures::future::join_all(futures).await;

    // Return the first successful result
    for result in results {
        if result.is_ok() {
            return result;
        }
    }

    // If no successful results, return a generic error
    Err(TestError::Async(
        "No successful results from concurrent tasks".to_string(),
    ))
}

/// Waits for all tasks to complete with a timeout
pub async fn wait_all_timeout<T>(
    tasks: Vec<impl Future<Output = T> + Send + 'static>,
    duration: Duration,
) -> Result<Vec<T>, TestError> {
    with_timeout(futures::future::join_all(tasks), duration).await
}

/// Utility for managing asynchronous test setup
pub struct AsyncTestHelper;

impl AsyncTestHelper {
    /// Creates a simple test task that sleeps for a given duration
    pub async fn sleep_task(duration: Duration) -> () {
        tokio::time::sleep(duration).await
    }

    /// Creates a failing task for testing error handling
    pub async fn failing_task<T>(message: &str) -> Result<T, TestError> {
        Err(TestError::Async(message.to_string()))
    }

    /// Creates a task that returns a value
    pub async fn value_task<T>(value: T) -> T {
        value
    }

    /// Races two tasks and returns the result of whichever completes first
    pub async fn race_tasks<T1, T2>(
        task1: impl Future<Output = Result<T1, TestError>> + Send + 'static,
        task2: impl Future<Output = Result<T2, TestError>> + Send + 'static,
    ) -> Result<futures::future::Either<T1, T2>, TestError> {
        match futures::future::select(Box::pin(task1), Box::pin(task2)).await {
            futures::future::Either::Left((result1, _)) => {
                Ok(futures::future::Either::Left(result1?))
            }
            futures::future::Either::Right((result2, _)) => {
                Ok(futures::future::Either::Right(result2?))
            }
        }
    }
}

/// Context for controlling execution of asynchronous tests
#[derive(Clone)]
pub struct AsyncContext {
    default_timeout: Duration,
}

impl Default for AsyncContext {
    fn default() -> Self {
        AsyncContext {
            default_timeout: Duration::from_secs(30),
        }
    }
}

impl AsyncContext {
    /// Creates a new context with custom timeout
    pub fn with_timeout(timeout: Duration) -> Self {
        AsyncContext {
            default_timeout: timeout,
        }
    }

    /// Executes a test task with the default timeout
    pub async fn execute<T>(&self, task: impl Future<Output = T>) -> Result<T, TestError> {
        with_timeout(task, self.default_timeout).await
    }

    /// Executes a concurrent test scenario
    pub async fn execute_concurrent<T>(
        &self,
        tasks: Vec<impl Future<Output = T> + Send + 'static>,
        concurrent_timeout: Option<Duration>,
    ) -> Result<Vec<T>, TestError>
    where
        T: Send + 'static,
    {
        let timeout = concurrent_timeout.unwrap_or(self.default_timeout);
        with_timeout(futures::future::join_all(tasks), timeout).await
    }
}

/// Macros for testing async operations
#[macro_export]
macro_rules! assert_async_timeout {
    ($future:expr, $timeout:expr) => {
        $crate::async_utils::with_timeout($future, $timeout)
            .await
            .expect_test(concat!("Async operation timed out in test: ", line!()));
    };
}

#[macro_export]
macro_rules! assert_async_fails {
    ($future:expr) => {
        assert!($future.await.is_err(), "Expected async operation to fail");
    };
}

/// Tokio test utilities
#[cfg(feature = "tokio")]
#[derive(Debug)]
pub struct TokioTestUtils;

#[cfg(feature = "tokio")]
impl TokioTestUtils {
    /// Sets up tokio runtime for tests
    pub fn init_runtime() -> Result<tokio::runtime::Runtime, TestError> {
        tokio::runtime::Runtime::new().map_err(|e| TestError::Async(e.to_string()))
    }
}

/// Advanced concurrency testing utilities
pub struct ConcurrencyTester {
    thread_count: usize,
}

impl ConcurrencyTester {
    pub fn new() -> Self {
        Self {
            thread_count: num_cpus::get(),
        }
    }

    pub fn with_thread_count(mut self, count: usize) -> Self {
        self.thread_count = count;
        self
    }

    /// Tests concurrent execution of a function
    pub async fn test_concurrent<F, Fut, T>(
        &self,
        test_fn: F,
        iterations: usize,
    ) -> Result<Vec<T>, TestError>
    where
        F: Fn() -> Fut + Send + Sync + Clone + 'static,
        Fut: Future<Output = Result<T, TestError>> + Send + 'static,
        T: Send + 'static,
    {
        let mut tasks = Vec::new();
        for _ in 0..iterations.min(self.thread_count * 10) {
            let test_fn = test_fn.clone();
            tasks.push(tokio::spawn(async move { test_fn().await }));
        }

        let results = futures::future::join_all(tasks).await;
        let mut final_results = Vec::new();

        for result in results {
            match result {
                Ok(Ok(value)) => final_results.push(value),
                Ok(Err(e)) => return Err(e),
                Err(e) => return Err(TestError::Async(format!("Task join error: {:?}", e))),
            }
        }

        Ok(final_results)
    }

    /// Tests for race conditions
    pub async fn test_race_conditions<F, Fut>(
        &self,
        setup: F,
        operations: Vec<Box<dyn FnOnce() -> Fut + Send + 'static>>,
    ) -> Result<(), TestError>
    where
        F: FnOnce() + Send + 'static,
        Fut: Future<Output = Result<(), TestError>> + Send + 'static,
    {
        let setup_result = tokio::spawn(async move { setup() }).await;
        if let Err(e) = setup_result {
            return Err(TestError::Async(format!("Setup failed: {:?}", e)));
        }

        let mut tasks = Vec::new();
        for operation in operations {
            tasks.push(tokio::spawn(async move { operation().await }));
        }

        let results = futures::future::join_all(tasks).await;
        for result in results {
            match result {
                Ok(Ok(_)) => continue,
                Ok(Err(e)) => return Err(e),
                Err(e) => {
                    return Err(TestError::Async(format!(
                        "Race condition test failed: {:?}",
                        e
                    )))
                }
            }
        }

        Ok(())
    }
}

impl Default for ConcurrencyTester {
    fn default() -> Self {
        Self::new()
    }
}

/// Async task scheduler for controlling execution timing in tests
pub struct AsyncScheduler {
    tasks: Vec<(
        String,
        Box<
            dyn FnOnce() -> std::pin::Pin<Box<dyn Future<Output = Result<(), TestError>> + Send>>
                + Send,
        >,
    )>,
}

impl AsyncScheduler {
    pub fn new() -> Self {
        Self { tasks: Vec::new() }
    }

    /// Add a task to be executed at a specific order
    pub fn add_task<F, Fut>(mut self, name: &str, task: F) -> Self
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = Result<(), TestError>> + Send + 'static,
    {
        self.tasks
            .push((name.to_string(), Box::new(move || Box::pin(task()))));
        self
    }

    /// Execute all tasks in sequence
    pub async fn execute_sequential(mut self) -> Result<Vec<String>, TestError> {
        let mut executed = Vec::new();

        for (name, task_fn) in self.tasks.drain(..) {
            task_fn().await?;
            executed.push(name);
        }

        Ok(executed)
    }

    /// Execute all tasks concurrently
    pub async fn execute_concurrent(mut self) -> Result<Vec<String>, TestError> {
        let mut task_handles = Vec::new();

        for (name, task_fn) in self.tasks.drain(..) {
            let handle = tokio::spawn(async move {
                task_fn().await?;
                Ok(name)
            });
            task_handles.push(handle);
        }

        let results = futures::future::join_all(task_handles).await;
        let mut executed = Vec::new();

        for result in results {
            match result {
                Ok(Ok(name)) => executed.push(name),
                Ok(Err(e)) => return Err(e),
                Err(e) => return Err(TestError::Async(format!("Task execution failed: {:?}", e))),
            }
        }

        Ok(executed)
    }
}

impl Default for AsyncScheduler {
    fn default() -> Self {
        Self::new()
    }
}

/// Stress testing utilities for async operations
pub struct AsyncStressTester {
    concurrency_level: usize,
    iterations: usize,
    timeout: Duration,
}

impl AsyncStressTester {
    pub fn new() -> Self {
        Self {
            concurrency_level: 10,
            iterations: 100,
            timeout: Duration::from_secs(30),
        }
    }

    pub fn with_concurrency(mut self, level: usize) -> Self {
        self.concurrency_level = level;
        self
    }

    pub fn with_iterations(mut self, iterations: usize) -> Self {
        self.iterations = iterations;
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Run stress test with the given async operation
    pub async fn run<F, Fut, T>(&self, operation: F) -> Result<StressTestResults, TestError>
    where
        F: Fn() -> Fut + Send + Sync + Clone + 'static,
        Fut: Future<Output = Result<T, TestError>> + Send + 'static,
        T: Send + 'static,
    {
        let start_time = std::time::Instant::now();
        let mut successful_operations = 0;
        let mut failed_operations = 0;
        let mut errors = Vec::new();

        // Use semaphore to control concurrency
        let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(self.concurrency_level));
        let mut tasks = Vec::new();

        for i in 0..self.iterations {
            let operation = operation.clone();
            let permit = semaphore
                .clone()
                .acquire_owned()
                .await
                .map_err(|_| TestError::Async("Semaphore acquire failed".to_string()))?;
            let task = tokio::spawn(async move {
                let _permit = permit; // Hold permit until task completes
                operation().await
            });
            tasks.push((i, task));
        }

        let results = futures::future::join_all(tasks.into_iter().map(|(i, task)| async move {
            let timeout_result = tokio::time::timeout(self.timeout, task).await;
            (i, timeout_result)
        }))
        .await;

        for (iteration, result) in results {
            match result {
                Ok(Ok(Ok(_))) => successful_operations += 1,
                Ok(Ok(Err(e))) => {
                    failed_operations += 1;
                    errors.push(format!("Iteration {}: {}", iteration, e));
                }
                Ok(Err(e)) => {
                    failed_operations += 1;
                    errors.push(format!("Iteration {}: Task panicked: {:?}", iteration, e));
                }
                Err(_) => {
                    failed_operations += 1;
                    errors.push(format!("Iteration {}: Timeout", iteration));
                }
            }
        }

        let total_time = start_time.elapsed();

        Ok(StressTestResults {
            total_operations: self.iterations,
            successful_operations,
            failed_operations,
            total_time,
            operations_per_second: self.iterations as f64 / total_time.as_secs_f64(),
            errors,
        })
    }
}

impl Default for AsyncStressTester {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct StressTestResults {
    pub total_operations: usize,
    pub successful_operations: usize,
    pub failed_operations: usize,
    pub total_time: Duration,
    pub operations_per_second: f64,
    pub errors: Vec<String>,
}

/// Async test hooks for setup and teardown
#[derive(Default)]
pub struct AsyncTestHooks {
    before_each: Vec<
        Box<
            dyn Fn() -> std::pin::Pin<Box<dyn Future<Output = Result<(), TestError>> + Send>>
                + Send
                + Sync,
        >,
    >,
    after_each: Vec<
        Box<
            dyn Fn() -> std::pin::Pin<Box<dyn Future<Output = Result<(), TestError>> + Send>>
                + Send
                + Sync,
        >,
    >,
}

impl AsyncTestHooks {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a before-each hook
    pub fn before_each<F, Fut>(mut self, hook: F) -> Self
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<(), TestError>> + Send + 'static,
    {
        self.before_each.push(Box::new(move || Box::pin(hook())));
        self
    }

    /// Add an after-each hook
    pub fn after_each<F, Fut>(mut self, hook: F) -> Self
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<(), TestError>> + Send + 'static,
    {
        self.after_each.push(Box::new(move || Box::pin(hook())));
        self
    }

    /// Execute all before-each hooks
    pub async fn run_before_each(&self) -> Result<(), TestError> {
        for hook in &self.before_each {
            hook().await?;
        }
        Ok(())
    }

    /// Execute all after-each hooks
    pub async fn run_after_each(&self) -> Result<(), TestError> {
        for hook in &self.after_each {
            hook().await?;
        }
        Ok(())
    }
}

/// Deadlock detection utilities
pub struct DeadlockDetector {
    active_tasks: Arc<Mutex<Vec<String>>>,
}

impl DeadlockDetector {
    pub fn new() -> Self {
        Self {
            active_tasks: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Register a task start
    pub fn task_started(&self, task_name: &str) {
        self.active_tasks
            .lock()
            .unwrap()
            .push(task_name.to_string());
    }

    /// Register a task completion
    pub fn task_completed(&self, task_name: &str) {
        let mut tasks = self.active_tasks.lock().unwrap();
        if let Some(pos) = tasks.iter().position(|t| t == task_name) {
            tasks.remove(pos);
        }
    }

    /// Check for potential deadlocks
    pub fn check_for_deadlocks(&self, max_wait: Duration) -> Result<(), TestError> {
        let start = std::time::Instant::now();
        while start.elapsed() < max_wait {
            if self.active_tasks.lock().unwrap().is_empty() {
                return Ok(());
            }
            std::thread::sleep(Duration::from_millis(10));
        }

        let active = self.active_tasks.lock().unwrap();
        Err(TestError::Async(format!(
            "Potential deadlock detected. Active tasks: {:?}",
            *active
        )))
    }

    /// Get currently active tasks
    pub fn active_tasks(&self) -> Vec<String> {
        self.active_tasks.lock().unwrap().clone()
    }
}

impl Default for DeadlockDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_concurrency_tester() {
        let tester = ConcurrencyTester::new().with_thread_count(2);

        let counter = Arc::new(Mutex::new(0));
        let counter_clone = counter.clone();

        let result = tester
            .test_concurrent(
                move || {
                    let counter = counter_clone.clone();
                    async move {
                        let mut val = counter.lock().unwrap();
                        *val += 1;
                        Ok(())
                    }
                },
                10,
            )
            .await;

        assert!(result.is_ok());
        assert_eq!(*counter.lock().unwrap(), 10);
    }

    #[tokio::test]
    async fn test_async_scheduler() {
        let scheduler = AsyncScheduler::new()
            .add_task("task1", || async { Ok(()) })
            .add_task("task2", || async { Ok(()) });

        let executed = scheduler.execute_sequential().await.unwrap();
        assert_eq!(executed, vec!["task1", "task2"]);
    }

    #[tokio::test]
    async fn test_deadlock_detector() {
        let detector = DeadlockDetector::new();

        detector.task_started("test_task");
        assert_eq!(detector.active_tasks(), vec!["test_task"]);

        detector.task_completed("test_task");
        assert_eq!(detector.active_tasks(), Vec::<String>::new());
    }

    #[tokio::test]
    async fn test_stress_tester() {
        let tester = AsyncStressTester::new()
            .with_iterations(5)
            .with_concurrency(2);

        let results = tester.run(|| async { Ok(()) }).await.unwrap();

        assert_eq!(results.total_operations, 5);
        assert_eq!(results.successful_operations, 5);
        assert_eq!(results.failed_operations, 0);
    }
}
