//! Tests for layer violation detection

use super::*;
use crate::analysis::{
    architectural::LayerViolationDetector,
    AnalysisType, Severity,
};

#[test]
fn test_proper_layer_architecture() {
    let code = r#"
        // Domain layer (core business logic)
        mod domain {
            pub struct Order {
                pub id: u64,
                pub amount: f64,
            }
            
            pub trait OrderRepository {
                fn save(&self, order: &Order);
            }
            
            pub struct OrderService<T: OrderRepository> {
                repository: T,
            }
            
            impl<T: OrderRepository> OrderService<T> {
                pub fn process_order(&self, order: Order) {
                    // Business logic here
                    self.repository.save(&order);
                }
            }
        }
        
        // Infrastructure layer (implementations, external services)
        mod infrastructure {
            use super::domain::{Order, OrderRepository};
            
            pub struct DatabaseOrderRepository;
            
            impl OrderRepository for DatabaseOrderRepository {
                fn save(&self, _order: &Order) {
                    // Database implementation
                }
            }
        }
        
        // Application layer (coordinating domain objects)
        mod application {
            use super::domain::{Order, OrderService};
            use super::infrastructure::DatabaseOrderRepository;
            
            pub fn process_order() {
                let repo = DatabaseOrderRepository;
                let service = OrderService { repository: repo };
                let order = Order { id: 1, amount: 100.0 };
                service.process_order(order);
            }
        }
    "#;
    
    let registry = create_test_registry();
    let result = registry.analyze_code(code, "test.rs").unwrap();
    
    assert_success(&result);
    assert!(
        !has_findings(&result, AnalysisType::LayerViolation, Severity::Error),
        "Expected no layer violations in properly layered architecture"
    );
}

#[test]
fn test_domain_depending_on_infrastructure() {
    let code = r#"
        mod domain {
            // Domain layer should not depend on infrastructure
            use super::infrastructure::DatabaseOrderRepository;
            
            pub struct OrderService {
                repository: DatabaseOrderRepository,
            }
            
            impl OrderService {
                pub fn process_order(&self) {
                    // ...
                }
            }
        }
        
        mod infrastructure {
            pub struct DatabaseOrderRepository;
        }
    "#;
    
    let registry = create_test_registry();
    let result = registry.analyze_code(code, "test.rs").unwrap();
    
    assert_finding!(
        &result,
        AnalysisType::LayerViolation,
        Severity::Error,
        "Domain layer depends on infrastructure layer"
    );
}

#[test]
fn test_cross_layer_dependencies() {
    let code = r#"
        // Infrastructure layer
        mod infrastructure {
            pub struct DatabaseConnection;
            
            // This is fine - infrastructure can depend on domain
            use super::domain::Order;
            
            impl DatabaseConnection {
                pub fn save_order(&self, _order: &Order) {}
            }
        }
        
        // Domain layer
        mod domain {
            // This is a violation - domain should not depend on infrastructure
            use super::infrastructure::DatabaseConnection;
            
            pub struct Order {
                pub id: u64,
            }
            
            pub struct OrderService {
                // Violation: Domain depends on infrastructure
                db: DatabaseConnection,
            }
            
            impl OrderService {
                pub fn process_order(&self) {
                    // ...
                }
            }
        }
        
        // Application layer
        mod application {
            // This is fine - application can depend on both domain and infrastructure
            use super::domain::OrderService;
            use super::infrastructure::DatabaseConnection;
            
            pub fn run() {
                let db = DatabaseConnection;
                let service = OrderService { db };
                service.process_order();
            }
        }
    "#;
    
    let registry = create_test_registry();
    let result = registry.analyze_code(code, "test.rs").unwrap();
    
    assert_finding!(
        &result,
        AnalysisType::LayerViolation,
        Severity::Error,
        "Domain layer depends on infrastructure layer"
    );
}

#[test]
fn test_cyclic_dependencies_between_layers() {
    let code = r#"
        // Domain layer
        mod domain {
            // Domain depends on application (violation)
            use super::application::ApplicationService;
            
            pub struct Order {
                pub id: u64,
            }
            
            pub struct OrderService {
                app_service: ApplicationService,
            }
        }
        
        // Application layer
        mod application {
            // Application depends on domain (valid)
            use super::domain::Order;
            
            pub struct ApplicationService;
            
            impl ApplicationService {
                pub fn process(&self, _order: &Order) {}
            }
        }
    "#;
    
    let registry = create_test_registry();
    let result = registry.analyze_code(code, "test.rs").unwrap();
    
    assert_finding!(
        &result,
        AnalysisType::LayerViolation,
        Severity::Error,
        "Cyclic dependency between layers detected"
    );
}

// Helper function to check for specific findings
fn has_findings(result: &AnalysisResult, analysis_type: AnalysisType, severity: Severity) -> bool {
    result.findings.iter().any(|f| f.analysis_type == analysis_type && f.severity == severity)
}
