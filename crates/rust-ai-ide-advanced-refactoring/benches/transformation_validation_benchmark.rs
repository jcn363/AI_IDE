use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use rust_ai_ide_advanced_refactoring::*;
use std::time::Duration;

// Helper function to generate test transformations
fn generate_test_transformations(count: usize) -> Vec<CodeTransformation> {
    (0..count)
        .map(|i| {
            let transform_type = match i % 6 {
                0 => TransformationType::ExtractMethod,
                1 => TransformationType::InlineVariable,
                2 => TransformationType::RenameSymbol,
                3 => TransformationType::ChangeSignature,
                4 => TransformationType::MoveMethod,
                _ => TransformationType::Other,
            };

            let complexity = (i % 10) + 1; // 1-10 complexity

            CodeTransformation {
                id: i.to_string(),
                transform_type,
                source: format!("source_{}.rs", i % 10),
                target: format!("target_{}.rs", i % 10),
                complexity,
                dependencies: (0..(i % 5)).map(|d| d.to_string()).collect(),
            }
        })
        .collect()
}

fn transformation_validation_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("transformation_validation");

    // Test with different numbers of transformations
    let transformation_counts = [
        ("small", 100),    // 100 transformations
        ("medium", 1_000), // 1,000 transformations
        ("large", 10_000), // 10,000 transformations
    ];

    for (size_name, count) in &transformation_counts {
        let transformations = generate_test_transformations(*count);
        let validator = TransformationValidator::new();

        // Benchmark validation of all transformations
        group.bench_with_input(
            BenchmarkId::new("validate_all", size_name),
            &transformations,
            |b, transforms| {
                b.iter(|| {
                    let results = validator.validate_all(transforms);
                    black_box(results);
                });
            },
        );

        // Benchmark individual validation
        group.bench_with_input(
            BenchmarkId::new("validate_each", size_name),
            &transformations,
            |b, transforms| {
                b.iter(|| {
                    let results: Vec<_> =
                        transforms.iter().map(|t| validator.validate(t)).collect();
                    black_box(results);
                });
            },
        );
    }

    // Test with different validation rules
    let rule_sets = [("basic", 1), ("standard", 3), ("strict", 5)];

    for (rules_name, rule_level) in &rule_sets {
        group.bench_with_input(
            BenchmarkId::new("validation_rules", rules_name),
            rule_level,
            |b, &level| {
                let transforms = generate_test_transformations(1000);
                let validator = TransformationValidator::with_rule_level(level);

                b.iter(|| {
                    let results = validator.validate_all(&transforms);
                    black_box(results);
                });
            },
        );
    }

    // Benchmark complex validation scenario
    group.bench_function("complex_validation_scenario", |b| {
        let transforms = generate_test_transformations(1000);
        let validator = TransformationValidator::new();

        b.iter(|| {
            // 1. Filter valid transformations
            let valid_transforms: Vec<_> = transforms
                .iter()
                .filter(|t| validator.validate(t).is_ok())
                .collect();

            // 2. Group by target file
            let mut targets = std::collections::HashMap::new();
            for t in valid_transforms {
                targets.entry(&t.target).or_insert_with(Vec::new).push(t);
            }

            // 3. Validate each target's transformations together
            let results: Vec<_> = targets
                .values()
                .map(|ts| validator.validate_batch(ts))
                .collect();

            black_box(results);
        });
    });

    group.finish();
}

// Mock implementations for the benchmark
#[derive(Debug, Clone, PartialEq)]
pub enum TransformationType {
    ExtractMethod,
    InlineVariable,
    RenameSymbol,
    ChangeSignature,
    MoveMethod,
    Other,
}

#[derive(Debug, Clone)]
pub struct CodeTransformation {
    pub id: String,
    pub transform_type: TransformationType,
    pub source: String,
    pub target: String,
    pub complexity: u8,
    pub dependencies: Vec<String>,
}

#[derive(Debug)]
pub enum ValidationError {
    InvalidTransformation(String),
    DependencyConflict(String),
    ComplexityTooHigh(u8),
}

pub struct TransformationValidator {
    max_complexity: u8,
    rule_level: u8,
}

impl TransformationValidator {
    pub fn new() -> Self {
        Self {
            max_complexity: 8,
            rule_level: 3,
        }
    }

    pub fn with_rule_level(level: u8) -> Self {
        Self {
            max_complexity: 10 - (level / 2), // Stricter rules = lower max complexity
            rule_level: level,
        }
    }

    pub fn validate_all(
        &self,
        transforms: &[CodeTransformation],
    ) -> Vec<Result<(), ValidationError>> {
        transforms.iter().map(|t| self.validate(t)).collect()
    }

    pub fn validate(&self, transform: &CodeTransformation) -> Result<(), ValidationError> {
        // Check complexity
        if transform.complexity > self.max_complexity {
            return Err(ValidationError::ComplexityTooHigh(transform.complexity));
        }

        // Check for required fields
        if transform.source.is_empty() || transform.target.is_empty() {
            return Err(ValidationError::InvalidTransformation(
                "Source and target must be specified".to_string(),
            ));
        }

        // Additional checks based on rule level
        if self.rule_level >= 2 && transform.dependencies.len() > 10 {
            return Err(ValidationError::DependencyConflict(
                "Too many dependencies".to_string(),
            ));
        }

        if self.rule_level >= 4 && transform.complexity > 5 {
            return Err(ValidationError::ComplexityTooHigh(transform.complexity));
        }

        Ok(())
    }

    pub fn validate_batch(
        &self,
        transforms: &[&CodeTransformation],
    ) -> Vec<Result<(), ValidationError>> {
        let mut results = Vec::with_capacity(transforms.len());

        for (i, &t) in transforms.iter().enumerate() {
            // Check for conflicts with previous transforms in the batch
            if self.rule_level >= 3 {
                for prev_t in &transforms[..i] {
                    if self.have_conflict(t, prev_t) {
                        results.push(Err(ValidationError::DependencyConflict(format!(
                            "Conflict with transformation {}",
                            prev_t.id
                        ))));
                        continue;
                    }
                }
            }

            // Validate the individual transform
            results.push(self.validate(t));
        }

        results
    }

    fn have_conflict(&self, a: &CodeTransformation, b: &CodeTransformation) -> bool {
        // Two transformations conflict if they modify the same file
        // and have overlapping dependencies or high complexity
        a.target == b.target
            && (a.complexity + b.complexity > 10
                || a.dependencies.iter().any(|d| b.dependencies.contains(d)))
    }
}

criterion_group!(
    name = benches;
    config = Criterion::default()
        .sample_size(10)
        .measurement_time(std::time::Duration::from_secs(10));
    targets = transformation_validation_benchmark
);
criterion_main!(benches);
