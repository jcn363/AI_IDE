//! Tests for dependency inversion principle analysis

use super::*;
use crate::analysis::{architectural::DependencyInversionAnalyzer, AnalysisType, Severity};

#[test]
fn test_proper_dependency_inversion() {
    let code = r#"
        // Abstract module defining the interface
        mod interfaces {
            pub trait Service {
                fn do_something(&self);
            }
        }
        
        // High-level module depending on abstraction
        mod client {
            use super::interfaces::Service;
            
            pub struct Client<T: Service> {
                service: T,
            }
            
            impl<T: Service> Client<T> {
                pub fn new(service: T) -> Self {
                    Client { service }
                }
                
                pub fn execute(&self) {
                    self.service.do_something();
                }
            }
        }
        
        // Low-level implementation
        mod service_impl {
            use super::interfaces::Service;
            
            pub struct ServiceImpl;
            
            impl Service for ServiceImpl {
                fn do_something(&self) {
                    // Implementation
                }
            }
        }
    "#;

    let registry = create_test_registry();
    let result = registry.analyze_code(code, "test.rs").unwrap();

    assert_success(&result);
    assert!(
        !has_findings(
            &result,
            AnalysisType::DependencyInversion,
            Severity::Warning
        ),
        "Expected no dependency inversion violations"
    );
}

#[test]
fn test_violation_direct_concrete_dependency() {
    let code = r#"
        // High-level module directly depending on low-level module
        mod client {
            use super::concrete_service::ConcreteService;
            
            pub struct Client {
                service: ConcreteService,
            }
            
            impl Client {
                pub fn new() -> Self {
                    Client { 
                        service: ConcreteService::new() 
                    }
                }
                
                pub fn execute(&self) {
                    self.service.do_something();
                }
            }
        }
        
        // Low-level module
        mod concrete_service {
            pub struct ConcreteService;
            
            impl ConcreteService {
                pub fn new() -> Self {
                    ConcreteService
                }
                
                pub fn do_something(&self) {
                    // Implementation
                }
            }
        }
    "#;

    let registry = create_test_registry();
    let result = registry.analyze_code(code, "test.rs").unwrap();

    assert_finding!(
        &result,
        AnalysisType::DependencyInversion,
        Severity::Warning,
        "High-level module 'client' directly depends on low-level module 'concrete_service'"
    );
}

#[test]
fn test_violation_concrete_dependency_in_method() {
    let code = r#"
        mod client {
            pub fn do_work() {
                let service = super::concrete::ConcreteService::new();
                service.do_something();
            }
        }
        
        mod concrete {
            pub struct ConcreteService;
            
            impl ConcreteService {
                pub fn new() -> Self {
                    ConcreteService
                }
                
                pub fn do_something(&self) {}
            }
        }
    "#;

    let registry = create_test_registry();
    let result = registry.analyze_code(code, "test.rs").unwrap();

    assert_finding!(
        &result,
        AnalysisType::DependencyInversion,
        Severity::Warning,
        "Direct dependency on concrete implementation 'ConcreteService'"
    );
}

#[test]
fn test_dependency_inversion_with_generics() {
    let code = r#"
        // Trait defining the interface
        pub trait Storage {
            fn save(&self, data: &str) -> Result<(), String>;
        }
        
        // High-level module depending on abstraction
        pub struct DataManager<T: Storage> {
            storage: T,
        }
        
        impl<T: Storage> DataManager<T> {
            pub fn new(storage: T) -> Self {
                DataManager { storage }
            }
            
            pub fn process_data(&self, data: &str) -> Result<(), String> {
                // Process data...
                self.storage.save(data)
            }
        }
        
        // Low-level implementation
        pub struct FileStorage;
        
        impl Storage for FileStorage {
            fn save(&self, _data: &str) -> Result<(), String> {
                // Implementation for file storage
                Ok(())
            }
        }
    "#;

    let registry = create_test_registry();
    let result = registry.analyze_code(code, "test.rs").unwrap();

    assert_success(&result);
    assert!(
        !has_findings(
            &result,
            AnalysisType::DependencyInversion,
            Severity::Warning
        ),
        "Expected no dependency inversion violations with proper generic usage"
    );
}

// Helper function to check for specific findings
fn has_findings(result: &AnalysisResult, analysis_type: AnalysisType, severity: Severity) -> bool {
    result
        .findings
        .iter()
        .any(|f| f.analysis_type == analysis_type && f.severity == severity)
}
