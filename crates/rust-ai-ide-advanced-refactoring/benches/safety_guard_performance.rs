use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use rust_ai_ide_advanced_refactoring::*;
use std::time::Duration;

// Helper function to generate test data
fn generate_test_data(size: usize) -> Vec<CodeChange> {
    (0..size)
        .map(|i| CodeChange {
            id: i.to_string(),
            change_type: match i % 5 {
                0 => ChangeType::FunctionModification,
                1 => ChangeType::VariableRename,
                2 => ChangeType::StructureChange,
                3 => ChangeType::DependencyUpdate,
                _ => ChangeType::Other,
            },
            risk_level: (i % 10) + 1, // 1-10 scale
            affected_files: (i % 5) + 1,
            complexity: (i % 10) + 1, // 1-10 scale
        })
        .collect()
}

fn safety_guard_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("safety_guard");

    // Test with different numbers of changes
    let change_counts = [
        ("small", 100),    // 100 changes
        ("medium", 1_000), // 1,000 changes
        ("large", 10_000), // 10,000 changes
    ];

    for (size_name, count) in &change_counts {
        let changes = generate_test_data(*count);
        let guard = SafetyGuard::new();

        // Benchmark safety validation
        group.bench_with_input(
            BenchmarkId::new("validate_changes", size_name),
            &changes,
            |b, changes| {
                b.iter(|| {
                    let result = guard.validate_changes(changes);
                    black_box(result);
                });
            },
        );

        // Benchmark risk assessment
        group.bench_with_input(
            BenchmarkId::new("assess_risk", size_name),
            &changes,
            |b, changes| {
                b.iter(|| {
                    let risks: Vec<_> = changes.iter().map(|c| guard.assess_risk(c)).collect();
                    black_box(risks);
                });
            },
        );
    }

    // Test with different risk thresholds
    let risk_thresholds = [3, 6, 9];

    for &threshold in &risk_thresholds {
        group.bench_with_input(
            BenchmarkId::new("apply_risk_threshold", threshold),
            &threshold,
            |b, &threshold| {
                let changes = generate_test_data(1000);
                let guard = SafetyGuard::with_threshold(threshold);

                b.iter(|| {
                    let risky_changes: Vec<_> =
                        changes.iter().filter(|c| guard.is_risky(c)).count();
                    black_box(risky_changes);
                });
            },
        );
    }

    // Benchmark complex validation scenario
    group.bench_function("complex_validation_scenario", |b| {
        let changes = generate_test_data(1000);
        let guard = SafetyGuard::new();

        b.iter(|| {
            // 1. Filter out high-risk changes
            let safe_changes: Vec<_> = changes.iter().filter(|c| !guard.is_high_risk(c)).collect();

            // 2. Validate remaining changes
            let validation_results: Vec<_> = safe_changes
                .iter()
                .map(|c| (c, guard.validate_change(c)))
                .collect();

            // 3. Count valid changes
            let valid_count = validation_results
                .iter()
                .filter(|(_, valid)| *valid)
                .count();

            black_box(valid_count);
        });
    });

    group.finish();
}

// Mock implementations for the benchmark
#[derive(Debug, Clone)]
pub enum ChangeType {
    FunctionModification,
    VariableRename,
    StructureChange,
    DependencyUpdate,
    Other,
}

#[derive(Debug, Clone)]
pub struct CodeChange {
    pub id: String,
    pub change_type: ChangeType,
    pub risk_level: u8,
    pub affected_files: usize,
    pub complexity: u8,
}

pub struct SafetyGuard {
    risk_threshold: u8,
}

impl SafetyGuard {
    pub fn new() -> Self {
        Self { risk_threshold: 5 }
    }

    pub fn with_threshold(risk_threshold: u8) -> Self {
        Self { risk_threshold }
    }

    pub fn validate_changes(&self, changes: &[CodeChange]) -> bool {
        changes.iter().all(|c| self.validate_change(c))
    }

    pub fn validate_change(&self, change: &CodeChange) -> bool {
        // Simple validation logic
        change.risk_level <= 8 && change.complexity <= 9
    }

    pub fn assess_risk(&self, change: &CodeChange) -> u8 {
        // Simple risk assessment
        let mut risk = change.risk_level;

        // Increase risk based on change type
        match change.change_type {
            ChangeType::StructureChange => risk = risk.saturating_add(2),
            ChangeType::DependencyUpdate => risk = risk.saturating_add(1),
            _ => {}
        }

        // Increase risk based on complexity
        if change.complexity > 7 {
            risk = risk.saturating_add(1);
        }

        risk.min(10) // Cap at 10
    }

    pub fn is_risky(&self, change: &CodeChange) -> bool {
        self.assess_risk(change) >= self.risk_threshold
    }

    pub fn is_high_risk(&self, change: &CodeChange) -> bool {
        self.assess_risk(change) >= 8
    }
}

criterion_group!(
    name = benches;
    config = Criterion::default()
        .sample_size(10)
        .measurement_time(std::time::Duration::from_secs(10));
    targets = safety_guard_benchmark
);
criterion_main!(benches);
