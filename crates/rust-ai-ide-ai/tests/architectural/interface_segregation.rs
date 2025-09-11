//! Tests for interface segregation principle analysis

use super::*;
use crate::analysis::{architectural::InterfaceSegregationAnalyzer, AnalysisType, Severity};

#[test]
fn test_proper_interface_segregation() {
    let code = r#"
        // Well-segregated interfaces
        mod interfaces {
            pub trait Reader {
                fn read(&self) -> String;
            }
            
            pub trait Writer {
                fn write(&mut self, data: &str);
            }
            
            // A type might implement multiple specific interfaces
            pub trait ReadWrite: Reader + Writer {}
        }
        
        // Implementation using only what it needs
        mod client {
            use super::interfaces::Reader;
            
            pub fn process_reader<R: Reader>(reader: &R) -> String {
                reader.read()
            }
        }
    "#;

    let registry = create_test_registry();
    let result = registry.analyze_code(code, "test.rs").unwrap();

    assert_success(&result);
    assert!(
        !has_findings(
            &result,
            AnalysisType::InterfaceSegregation,
            Severity::Warning
        ),
        "Expected no interface segregation violations"
    );
}

#[test]
fn test_violation_large_interface() {
    let code = r#"
        // Large interface that forces implementors to provide many methods
        mod bad_interface {
            pub trait MultiFunctionDevice {
                fn print(&self, document: &str);
                fn scan(&self) -> String;
                fn fax(&self, document: &str);
                fn email(&self, recipient: &str, document: &str);
            }
            
            // Client that only needs to print
            pub struct Client<T: MultiFunctionDevice> {
                device: T,
            }
            
            impl<T: MultiFunctionDevice> Client<T> {
                pub fn print_document(&self, document: &str) {
                    self.device.print(document);
                }
            }
        }
    "#;

    let registry = create_test_registry();
    let result = registry.analyze_code(code, "test.rs").unwrap();

    assert_finding!(
        &result,
        AnalysisType::InterfaceSegregation,
        Severity::Warning,
        "Large interface 'MultiFunctionDevice' with 4 methods may violate interface segregation"
    );
}

#[test]
fn test_violation_forced_unused_methods() {
    let code = r#"
        mod bad_design {
            // Trait with methods that might not be needed by all implementors
            pub trait Worker {
                fn work(&self);
                fn eat(&self);
            }
            
            // Robot doesn't need to eat, but must implement the method
            pub struct Robot;
            
            impl Worker for Robot {
                fn work(&self) {
                    // Working...
                }
                
                fn eat(&self) {
                    // Robot doesn't eat, but must implement this method
                    panic!("Robots don't eat!");
                }
            }
        }
    "#;

    let registry = create_test_registry();
    let result = registry.analyze_code(code, "test.rs").unwrap();

    assert_finding!(
        &result,
        AnalysisType::InterfaceSegregation,
        Severity::Warning,
        "Implementor 'Robot' is forced to implement unused method 'eat'"
    );
}

#[test]
fn test_well_segregated_interfaces() {
    let code = r#"
        // Better design with segregated interfaces
        mod good_design {
            // Separate interfaces for different capabilities
            pub trait Workable {
                fn work(&self);
            }
            
            pub trait Eatable {
                fn eat(&self);
            }
            
            // Types implement only what they need
            pub struct Human;
            
            impl Workable for Human {
                fn work(&self) {
                    // Working...
                }
            }
            
            impl Eatable for Human {
                fn eat(&self) {
                    // Eating...
                }
            }
            
            pub struct Robot;
            
            impl Workable for Robot {
                fn work(&self) {
                    // Working...
                }
            }
            
            // Robot doesn't implement Eatable, which is correct
        }
    "#;

    let registry = create_test_registry();
    let result = registry.analyze_code(code, "test.rs").unwrap();

    assert_success(&result);
    assert!(
        !has_findings(
            &result,
            AnalysisType::InterfaceSegregation,
            Severity::Warning
        ),
        "Expected no interface segregation violations with well-segregated interfaces"
    );
}

// Helper function to check for specific findings
fn has_findings(result: &AnalysisResult, analysis_type: AnalysisType, severity: Severity) -> bool {
    result
        .findings
        .iter()
        .any(|f| f.analysis_type == analysis_type && f.severity == severity)
}
