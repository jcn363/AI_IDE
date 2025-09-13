//! Analysis utilities and helper functions

use serde::{Deserialize, Serialize};

use crate::analysis::types::{AnalysisCategory, Range, Severity};

/// Create a standard analysis finding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisFinding {
    /// The type of the finding
    pub kind:       String,
    /// Description of the finding
    pub message:    String,
    /// The actual finding data as a JSON string
    pub data:       String,
    /// Severity level
    pub severity:   Severity,
    /// Category of the finding
    pub category:   AnalysisCategory,
    /// Location in the source code
    pub location:   String,
    /// Code range where the finding was detected
    pub range:      Range,
    /// Optional suggestion for fixing the issue
    pub suggestion: Option<String>,
    /// Confidence level (0.0 to 1.0)
    pub confidence: f32,
    /// Unique identifier for the rule that generated this finding
    pub rule_id:    String,
}

/// Create a progress bar for analysis operations
pub mod progress {
    use indicatif::{ProgressBar, ProgressStyle};

    pub fn progress_bar(len: u64, message: &str) -> ProgressBar {
        let pb = ProgressBar::new(len);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos:>7}/{len:7} {msg}")
                .expect("Progress bar template should be valid")
                .progress_chars("#>-"),
        );
        pb.set_message(message.to_string());
        pb
    }
}

/// Create a standard analysis finding
pub fn create_finding(
    message: String,
    severity: Severity,
    category: AnalysisCategory,
    range: Range,
    suggestion: Option<String>,
    confidence: f32,
    rule_id: String,
) -> AnalysisFinding {
    let location = format!("{}:{}", range.start_line, range.start_col);

    AnalysisFinding {
        kind: category.to_string().to_lowercase(),
        message,
        data: String::new(),
        severity,
        category,
        location,
        range,
        suggestion,
        confidence: confidence.clamp(0.0, 1.0),
        rule_id,
    }
}

/// Extract line number from syn span
pub fn extract_line_number(span: &proc_macro2::Span) -> u32 {
    let source_text = span.unwrap().source_text().unwrap_or_default();
    source_text.lines().count() as u32
}

/// Create a range from syn span
pub fn span_to_range(span: &proc_macro2::Span) -> Range {
    // Get the source text position from the span
    let source_text = span.unwrap().source_text().unwrap_or_default();

    // For now, return a default range since we can't get accurate positions
    // This will need to be updated if we need accurate positions
    Range {
        start_line: 1,
        start_col:  1,
        end_line:   1,
        end_col:    source_text.len() as u32 + 1, // Approximate end column
    }
}

/// Check if a finding meets the confidence threshold
pub fn meets_confidence_threshold(finding: &AnalysisFinding, threshold: f32) -> bool {
    finding.confidence >= threshold
}

/// Merge overlapping ranges
pub fn merge_ranges(ranges: &[Range]) -> Vec<Range> {
    if ranges.is_empty() {
        return Vec::new();
    }

    let mut sorted_ranges = ranges.to_vec();
    sorted_ranges.sort_by_key(|r| (r.start_line, r.start_col));

    let mut merged = vec![sorted_ranges[0]];

    for range in sorted_ranges.iter().skip(1) {
        let last = merged.last_mut().unwrap();

        // Check if ranges overlap or are adjacent
        if range.start_line <= last.end_line + 1 {
            // Merge ranges
            last.end_line = last.end_line.max(range.end_line);
            last.end_col = if last.end_line == range.end_line {
                last.end_col.max(range.end_col)
            } else {
                range.end_col
            };
        } else {
            // No overlap, add new range
            merged.push(*range);
        }
    }

    merged
}

/// Re-export progress_bar function for easier access
pub use progress::progress_bar;
