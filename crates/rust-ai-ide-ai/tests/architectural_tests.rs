//! Tests for architectural analysis

use rust_ai_ide_ai::analysis::architectural::*;
use rust_ai_ide_ai::analysis::CodeLocation;
use syn::{parse_quote, File};

#[test]
fn test_complexity_analysis() {
    let analyzer = ArchitecturalAnalyzer::with_all_checks()
        .with_max_cyclomatic_complexity(5)
        .with_max_trait_methods(3);

    // Simple function with low complexity
    let simple_fn: File = parse_quote! {
        fn simple() {
            if true {
                println!("Hello");
            }
        }
    };

    // Complex function with high complexity
    let complex_fn: File = parse_quote! {
        fn complex() {
            if true {
                for i in 0..10 {
                    match i {
                        0 => println!("Zero"),
                        1..=5 => println!("Low"),
                        _ => println!("High"),
                    }
                }
            }
            while true {
                break;
            }
        }
    };

    // Test simple function
    let findings = analyzer.analyze(
        &simple_fn,
        "test.rs",
        "fn simple() { if true { println!(\"Hello\"); } }",
    );
    assert!(
        findings.is_empty(),
        "Simple function should have no findings"
    );

    // Test complex function
    let findings = analyzer.analyze(
        &complex_fn,
        "test.rs",
        "fn complex() { if true { for i in 0..10 { match i { 0 => println!(\"Zero\"), 1..=5 => println!(\"Low\"), _ \
         => println!(\"High\"), } } while true { break; } }",
    );
    assert!(
        !findings.is_empty(),
        "Complex function should have findings"
    );
    assert!(
        findings.iter().any(|f| f.id == "high-complexity-method"),
        "Should find high complexity method"
    );
}

#[test]
fn test_dependency_inversion() {
    let analyzer = ArchitecturalAnalyzer::with_all_checks().with_allowed_concrete_dependency("ConcreteType");

    // Violation: Direct concrete type dependency
    let violation: File = parse_quote! {
        struct ConcreteType {
            value: i32
        }

        impl ConcreteType {
            fn new() -> Self { Self { value: 0 } }
        }

        struct BadStruct {
            bad: ConcreteType,
        }

        impl BadStruct {
            fn new() -> Self {
                Self { bad: ConcreteType::new() }
            }
        }
    };

    // No violation: Using a trait
    let no_violation: File = parse_quote! {
        trait GoodTrait {}

        struct GoodStruct<T: GoodTrait> {
            good: T,
        }
    };

    // Test violation
    let findings = analyzer.analyze(
        &violation,
        "test.rs",
        "struct ConcreteType { value: i32 } impl ConcreteType { fn new() -> Self { Self { value: 0 } } }",
    );
    assert!(
        !findings.is_empty(),
        "Should find dependency inversion violation"
    );

    // Test no violation
    let findings = analyzer.analyze(
        &no_violation,
        "test.rs",
        "trait GoodTrait {} struct GoodStruct<T: GoodTrait> { good: T }",
    );
    assert!(findings.is_empty(), "Should not find any violations");
}

#[test]
fn test_interface_segregation() {
    let analyzer = ArchitecturalAnalyzer::with_all_checks().with_max_trait_methods(2);

    // Violation: Too many methods in trait
    let violation: File = parse_quote! {
        trait TooBigTrait {
            fn method1(&self);
            fn method2(&self);
            fn method3(&self);
        }
    };

    // No violation: Small, focused trait
    let no_violation: File = parse_quote! {
        trait GoodTrait {
            fn do_one_thing(&self);
        }
    };

    // Test violation
    let findings = analyzer.analyze(
        &violation,
        "test.rs",
        "trait TooBigTrait { fn method1(&self); fn method2(&self); fn method3(&self); }",
    );
    assert!(
        !findings.is_empty(),
        "Should find interface segregation violation"
    );

    // Test no violation
    let findings = analyzer.analyze(
        &no_violation,
        "test.rs",
        "trait GoodTrait { fn do_one_thing(&self); }",
    );
    assert!(
        findings.is_empty(),
        "Should not find any violations: {:?}",
        findings
    );
}

#[test]
fn test_module_size_validation() {
    let analyzer = ArchitecturalAnalyzer::with_all_checks().with_max_module_size(5);

    // Create a file with too many lines
    let code = "// Line 1\n// Line 2\n// Line 3\n// Line 4\n// Line 5\n// Line 6";

    let ast = syn::parse_file(code).unwrap();
    let findings = analyzer.analyze(&ast, "test.rs", code);

    assert!(!findings.is_empty(), "Should find module size violation");
    assert!(findings.iter().any(|f| f.id == "module-too-large"));
}

#[test]
fn test_public_items_validation() {
    let analyzer = ArchitecturalAnalyzer::with_all_checks().with_max_public_items(2);

    // File with too many public items
    let code = "
        pub struct A;
        pub struct B;
        pub struct C;
    ";

    let ast = syn::parse_file(code).unwrap();
    let findings = analyzer.analyze(&ast, "test.rs", code);

    assert!(!findings.is_empty(), "Should find too many public items");
    assert!(findings.iter().any(|f| f.id == "too-many-public-items"));
}
