//! Lazy loading infrastructure for components

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock, Semaphore};
use tokio::time::{timeout, Duration};
use async_trait::async_trait;
use std::any::Any;

use crate::{LazyComponent, LazyLoadingConfig, LazyLoadingError, LazyResult};

/// Manager for lazy loading components
pub struct LazyLoader {
    registry: Arc<RwLock<HashMap<String, Box<dyn LazyComponent>>>>,
    config: LazyLoadingConfig,
    semaphore: Arc<Semaphore>,
    loaded_components: Arc<RwLock<HashMap<String, Arc<dyn Any + Send + Sync>>>>,
}

impl LazyLoader {
    /// Create a new lazy loader with the given configuration
    pub fn new(config: LazyLoadingConfig) -> Self {
        Self {
            registry: Arc::new(RwLock::new(HashMap::new())),
            config,
            semaphore: Arc::new(Semaphore::new(config.max_concurrent_loads)),
            loaded_components: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a lazy component
    pub async fn register_component(
        &self,
        component: Box<dyn LazyComponent>,
    ) -> LazyResult<()> {
        let name = component.name().to_string();
        let mut registry = self.registry.write().await;

        if registry.contains_key(&name) {
            return Err(LazyLoadingError::invalid_configuration(
                "component_name",
                format!("Component '{}' already registered", name),
            ));
        }

        registry.insert(name, component);
        Ok(())
    }

    /// Get a component, loading it if necessary
    pub async fn get_component<T: 'static + Send + Sync>(
        &self,
        name: &str,
    ) -> LazyResult<Arc<T>> {
        // Check if component is already loaded
        {
            let loaded = self.loaded_components.read().await;
            if let Some(component) = loaded.get(name) {
                if let Ok(typed_component) = component.clone().downcast::<T>() {
                    return Ok(typed_component);
                }
            }
        }

        // Acquire semaphore permit for concurrent load limiting
        let _permit = self.semaphore.acquire().await.map_err(|_| {
            LazyLoadingError::concurrent_load_limit_exceeded(
                self.config.max_concurrent_loads,
                self.config.max_concurrent_loads,
            )
        })?;

        // Load the component with timeout
        let load_future = self.load_component_inner(name);
        let component = timeout(
            Duration::from_secs(self.config.load_timeout_seconds),
            load_future,
        )
        .await
        .map_err(|_| LazyLoadingError::loading_timeout(name.to_string(), self.config.load_timeout_seconds))?
        .map_err(|e| {
            LazyLoadingError::initialization_failed(name.to_string(), e.to_string())
        })?;

        // Try to downcast to the requested type
        let typed_component = component.downcast::<T>().map_err(|_| {
            LazyLoadingError::internal(format!(
                "Component '{}' could not be downcast to requested type",
                name
            ))
        })?;

        Ok(typed_component)
    }

    /// Load a component internally
    async fn load_component_inner(&self, name: &str) -> LazyResult<Arc<dyn Any + Send + Sync>> {
        let registry = self.registry.read().await;
        let mut component = registry
            .get(name)
            .ok_or_else(|| LazyLoadingError::component_not_found(name.to_string()))?
            .as_ref()
            .clone_box();

        // Check if already loaded (double-check locking pattern)
        if !component.is_loaded() {
            component.load().await?;
        }

        // Create an Arc wrapper for the loaded component
        // Note: This assumes the component implements Clone or we need to wrap it
        // For now, we'll create a simple wrapper
        let component_arc = Arc::new(component);

        // Store in loaded components cache
        let mut loaded = self.loaded_components.write().await;
        loaded.insert(name.to_string(), component_arc.clone());

        Ok(component_arc)
    }

    /// Check if a component is loaded
    pub async fn is_component_loaded(&self, name: &str) -> bool {
        let loaded = self.loaded_components.read().await;
        loaded.contains_key(name)
    }

    /// Unload a component to free memory
    pub async fn unload_component(&self, name: &str) -> LazyResult<()> {
        let mut loaded = self.loaded_components.write().await;

        if let Some(_component) = loaded.remove(name) {
            // The component will be dropped when all Arc references are gone
            Ok(())
        } else {
            Err(LazyLoadingError::component_not_found(name.to_string()))
        }
    }

    /// Get memory usage of all loaded components
    pub async fn get_total_memory_usage(&self) -> usize {
        let registry = self.registry.read().await;
        let mut total = 0;

        for (name, component) in registry.iter() {
            if self.is_component_loaded(name).await {
                total += component.memory_usage();
            }
        }

        total
    }

    /// Get list of registered component names
    pub async fn get_registered_components(&self) -> Vec<String> {
        let registry = self.registry.read().await;
        registry.keys().cloned().collect()
    }

    /// Get list of loaded component names
    pub async fn get_loaded_components(&self) -> Vec<String> {
        let loaded = self.loaded_components.read().await;
        loaded.keys().cloned().collect()
    }

    /// Preload multiple components concurrently
    pub async fn preload_components(&self, component_names: &[&str]) -> LazyResult<()> {
        let mut futures = Vec::new();

        for &name in component_names {
            let loader = self.clone();
            let name = name.to_string();

            let future = tokio::spawn(async move {
                loader.get_component::<Box<dyn Any + Send + Sync>>(&name).await?;
                Ok::<_, LazyLoadingError>(())
            });

            futures.push(future);
        }

        // Wait for all preload operations to complete
        for future in futures {
            future.await.map_err(|e| {
                LazyLoadingError::internal(format!("Preload task failed: {}", e))
            })??;
        }

        Ok(())
    }
}

impl Clone for LazyLoader {
    fn clone(&self) -> Self {
        Self {
            registry: self.registry.clone(),
            config: self.config.clone(),
            semaphore: self.semaphore.clone(),
            loaded_components: self.loaded_components.clone(),
        }
    }
}

/// Helper trait for cloning boxed components
trait CloneBox {
    fn clone_box(&self) -> Box<dyn LazyComponent>;
}

impl<T> CloneBox for T
where
    T: 'static + LazyComponent + Clone,
{
    fn clone_box(&self) -> Box<dyn LazyComponent> {
        Box::new(self.clone())
    }
}

/// Lazy-loaded component wrapper for simple types
pub struct SimpleLazyComponent<T, F>
where
    T: Send + Sync + 'static,
    F: Fn() -> LazyResult<T> + Send + Sync + 'static,
{
    name: String,
    initializer: F,
    loaded: Mutex<Option<Arc<T>>>,
}

impl<T, F> SimpleLazyComponent<T, F>
where
    T: Send + Sync + 'static,
    F: Fn() -> LazyResult<T> + Send + Sync + 'static,
{
    /// Create a new simple lazy component
    pub fn new(name: impl Into<String>, initializer: F) -> Self {
        Self {
            name: name.into(),
            initializer,
            loaded: Mutex::new(None),
        }
    }
}

#[async_trait]
impl<T, F> LazyComponent for SimpleLazyComponent<T, F>
where
    T: Send + Sync + 'static,
    F: Fn() -> LazyResult<T> + Send + Sync + 'static,
{
    fn name(&self) -> &str {
        &self.name
    }

    fn is_loaded(&self) -> bool {
        self.loaded.try_lock().map(|l| l.is_some()).unwrap_or(false)
    }

    async fn load(&mut self) -> LazyResult<()> {
        let mut loaded = self.loaded.lock().await;
        if loaded.is_none() {
            let component = (self.initializer)()?;
            *loaded = Some(Arc::new(component));
        }
        Ok(())
    }

    fn memory_usage(&self) -> usize {
        // Estimate memory usage - this is a rough approximation
        std::mem::size_of::<T>()
    }

    async fn unload(&mut self) -> LazyResult<()> {
        let mut loaded = self.loaded.lock().await;
        *loaded = None;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    fn create_test_component() -> LazyResult<String> {
        COUNTER.fetch_add(1, Ordering::SeqCst);
        Ok(format!("component-{}", COUNTER.load(Ordering::SeqCst)))
    }

    #[tokio::test]
    async fn test_simple_lazy_component() {
        let mut component = SimpleLazyComponent::new("test", create_test_component);

        assert_eq!(component.name(), "test");
        assert!(!component.is_loaded());

        component.load().await.unwrap();
        assert!(component.is_loaded());
        assert_eq!(component.memory_usage(), std::mem::size_of::<String>());
    }

    #[tokio::test]
    async fn test_lazy_loader_registration() {
        let loader = LazyLoader::new(LazyLoadingConfig::default());
        let component = SimpleLazyComponent::new("test", create_test_component);

        loader.register_component(Box::new(component)).await.unwrap();

        let components = loader.get_registered_components().await;
        assert!(components.contains(&"test".to_string()));
    }

    #[tokio::test]
    async fn test_component_not_found() {
        let loader = LazyLoader::new(LazyLoadingConfig::default());

        let result = loader.get_component::<String>("nonexistent").await;
        assert!(matches!(result, Err(LazyLoadingError::ComponentNotFound(_))));
    }
}