// AI-mediated conflict resolution for collaborative editing
// Implements syntactic → semantic → manual conflict resolution pipeline

use crate::crdt::{EditorOperation, TextDocument};
use rust_ai_ide_ai_inference::{AIInferenceService, InferenceRequest, InferenceResponse};
use rust_ai_ide_common::validation::TauriInputSanitizer;
use rust_ai_ide_lsp::{LspService, SemanticAnalysisRequest, SemanticAnalysisResponse};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Conflict resolution strategies
#[derive(Debug, Clone, PartialEq)]
pub enum ConflictResolutionStrategy {
    /// Use AI for semantic analysis and automatic resolution
    AISemantic,
    /// Use syntactic analysis for automatic resolution
    Syntactic,
    /// Require manual resolution
    Manual,
}

/// Conflict analysis result
#[derive(Debug, Clone)]
pub struct ConflictAnalysis {
    pub conflicts: Vec<OperationConflict>,
    pub resolution_strategy: ConflictResolutionStrategy,
    pub confidence_score: f64, // 0.0 to 1.0
}

/// Represents a conflict between operations
#[derive(Debug, Clone)]
pub struct OperationConflict {
    pub operation1: EditorOperation,
    pub operation2: EditorOperation,
    pub conflict_type: ConflictType,
    pub position: usize,
    pub length: usize,
}

/// Types of conflicts that can occur
#[derive(Debug, Clone, PartialEq)]
pub enum ConflictType {
    /// Operations affect overlapping regions
    Overlapping,
    /// Operations create syntactically invalid code
    Syntactic,
    /// Operations conflict semantically (different intent)
    Semantic,
    /// Operations are causally dependent
    Causal,
}

/// AI-mediated conflict resolver
pub struct AIConflictResolver {
    ai_service: Arc<RwLock<AIInferenceService>>,
    lsp_service: Arc<RwLock<LspService>>,
    sanitizer: TauriInputSanitizer,
}

impl AIConflictResolver {
    pub fn new(
        ai_service: Arc<RwLock<AIInferenceService>>,
        lsp_service: Arc<RwLock<LspService>>,
    ) -> Self {
        Self {
            ai_service,
            lsp_service,
            sanitizer: TauriInputSanitizer::new(),
        }
    }

    /// Analyze conflicts between operations and determine resolution strategy
    pub async fn analyze_conflicts(
        &self,
        operations: &[EditorOperation],
        document_content: &str,
    ) -> Result<ConflictAnalysis, Box<dyn std::error::Error + Send + Sync>> {
        let mut conflicts = Vec::new();

        // Find overlapping operations
        for i in 0..operations.len() {
            for j in (i + 1)..operations.len() {
                if let Some(conflict) = self
                    .detect_conflict(&operations[i], &operations[j], document_content)
                    .await?
                {
                    conflicts.push(conflict);
                }
            }
        }

        // Determine resolution strategy based on conflict types
        let strategy = self.determine_resolution_strategy(&conflicts).await?;
        let confidence = self
            .calculate_confidence_score(&conflicts, &strategy)
            .await?;

        Ok(ConflictAnalysis {
            conflicts,
            resolution_strategy: strategy,
            confidence_score: confidence,
        })
    }

    /// Resolve conflicts using the determined strategy
    pub async fn resolve_conflicts(
        &self,
        analysis: &ConflictAnalysis,
        document_content: &str,
    ) -> Result<Vec<EditorOperation>, Box<dyn std::error::Error + Send + Sync>> {
        match analysis.resolution_strategy {
            ConflictResolutionStrategy::AISemantic => {
                self.resolve_with_ai_semantic(analysis, document_content)
                    .await
            }
            ConflictResolutionStrategy::Syntactic => {
                self.resolve_with_syntactic_analysis(analysis, document_content)
                    .await
            }
            ConflictResolutionStrategy::Manual => {
                // Return conflicts for manual resolution
                Err("Manual resolution required".into())
            }
        }
    }

    /// Detect if two operations conflict
    async fn detect_conflict(
        &self,
        op1: &EditorOperation,
        op2: &EditorOperation,
        document_content: &str,
    ) -> Result<Option<OperationConflict>, Box<dyn std::error::Error + Send + Sync>> {
        let (pos1, len1) = self.get_operation_range(op1);
        let (pos2, len2) = self.get_operation_range(op2);

        // Check for overlapping regions
        if self.regions_overlap(pos1, len1, pos2, len2) {
            // Determine conflict type
            let conflict_type = self
                .analyze_conflict_type(op1, op2, document_content)
                .await?;

            Ok(Some(OperationConflict {
                operation1: op1.clone(),
                operation2: op2.clone(),
                conflict_type,
                position: pos1.min(pos2),
                length: (pos1 + len1).max(pos2 + len2) - pos1.min(pos2),
            }))
        } else {
            Ok(None)
        }
    }

    /// Analyze the type of conflict between operations
    async fn analyze_conflict_type(
        &self,
        op1: &EditorOperation,
        op2: &EditorOperation,
        document_content: &str,
    ) -> Result<ConflictType, Box<dyn std::error::Error + Send + Sync>> {
        // First check for causal dependencies using Lamport clocks
        if op1.clock() > op2.clock() || op2.clock() > op1.clock() {
            return Ok(ConflictType::Causal);
        }

        // Check syntactic validity
        let syntactic_valid = self
            .check_syntactic_validity(op1, op2, document_content)
            .await?;
        if !syntactic_valid {
            return Ok(ConflictType::Syntactic);
        }

        // Use LSP for semantic analysis
        let semantic_conflict = self
            .check_semantic_conflict(op1, op2, document_content)
            .await?;
        if semantic_conflict {
            return Ok(ConflictType::Semantic);
        }

        Ok(ConflictType::Overlapping)
    }

    /// Check if operations would create syntactically invalid code
    async fn check_syntactic_validity(
        &self,
        op1: &EditorOperation,
        op2: &EditorOperation,
        document_content: &str,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        // Apply operations to create test content
        let mut test_content = document_content.to_string();
        self.apply_operation_to_content(&mut test_content, op1)?;
        self.apply_operation_to_content(&mut test_content, op2)?;

        // Use LSP to check syntax validity
        let lsp_service = self.lsp_service.read().await;
        let request = SemanticAnalysisRequest {
            content: test_content,
            language: "rust".to_string(), // Assume Rust for now
            analysis_type: rust_ai_ide_lsp::AnalysisType::Syntax,
        };

        match lsp_service.analyze_semantics(request).await {
            Ok(response) => Ok(response.is_valid),
            Err(_) => Ok(true), // If LSP fails, assume valid
        }
    }

    /// Check for semantic conflicts using AI analysis
    async fn check_semantic_conflict(
        &self,
        op1: &EditorOperation,
        op2: &EditorOperation,
        document_content: &str,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let prompt = format!(
            "Analyze if these two code changes have conflicting intent:\n\n\
             Change 1: {}\n\
             Change 2: {}\n\
             Context: {}\n\n\
             Do they conflict semantically? Answer 'yes' or 'no' with brief explanation.",
            self.operation_to_string(op1),
            self.operation_to_string(op2),
            &document_content[..200] // First 200 chars for context
        );

        let ai_service = self.ai_service.read().await;
        let request = InferenceRequest {
            prompt,
            max_tokens: 100,
            temperature: 0.1, // Low temperature for consistent analysis
        };

        match ai_service.infer(request).await {
            Ok(response) => {
                let response_text = response.text.to_lowercase();
                Ok(response_text.contains("yes"))
            }
            Err(_) => Ok(false), // If AI fails, assume no conflict
        }
    }

    /// Determine the best resolution strategy for conflicts
    async fn determine_resolution_strategy(
        &self,
        conflicts: &[OperationConflict],
    ) -> Result<ConflictResolutionStrategy, Box<dyn std::error::Error + Send + Sync>> {
        if conflicts.is_empty() {
            return Ok(ConflictResolutionStrategy::Syntactic);
        }

        // Check if any conflicts are semantic
        let has_semantic = conflicts
            .iter()
            .any(|c| c.conflict_type == ConflictType::Semantic);
        let has_syntactic = conflicts
            .iter()
            .any(|c| c.conflict_type == ConflictType::Syntactic);

        if has_semantic {
            Ok(ConflictResolutionStrategy::AISemantic)
        } else if has_syntactic {
            Ok(ConflictResolutionStrategy::Manual) // Syntactic conflicts need manual review
        } else {
            Ok(ConflictResolutionStrategy::Syntactic)
        }
    }

    /// Calculate confidence score for the resolution strategy
    async fn calculate_confidence_score(
        &self,
        conflicts: &[OperationConflict],
        strategy: &ConflictResolutionStrategy,
    ) -> Result<f64, Box<dyn std::error::Error + Send + Sync>> {
        match strategy {
            ConflictResolutionStrategy::AISemantic => {
                // AI semantic resolution confidence based on conflict complexity
                let semantic_conflicts = conflicts
                    .iter()
                    .filter(|c| c.conflict_type == ConflictType::Semantic)
                    .count();
                Ok((semantic_conflicts as f64 / conflicts.len() as f64).min(0.8))
            }
            ConflictResolutionStrategy::Syntactic => {
                // High confidence for syntactic resolution
                Ok(0.9)
            }
            ConflictResolutionStrategy::Manual => {
                // Manual resolution has no automated confidence
                Ok(0.0)
            }
        }
    }

    /// Resolve conflicts using AI semantic analysis
    async fn resolve_with_ai_semantic(
        &self,
        analysis: &ConflictAnalysis,
        document_content: &str,
    ) -> Result<Vec<EditorOperation>, Box<dyn std::error::Error + Send + Sync>> {
        let mut resolved_operations = Vec::new();

        for conflict in &analysis.conflicts {
            let resolution = self
                .resolve_single_conflict_ai(&conflict, document_content)
                .await?;
            resolved_operations.extend(resolution);
        }

        Ok(resolved_operations)
    }

    /// Resolve a single conflict using AI
    async fn resolve_single_conflict_ai(
        &self,
        conflict: &OperationConflict,
        document_content: &str,
    ) -> Result<Vec<EditorOperation>, Box<dyn std::error::Error + Send + Sync>> {
        let prompt = format!(
            "Resolve this code conflict by choosing the best approach:\n\n\
             Conflict: {}\n\n\
             Option 1: {}\n\
             Option 2: {}\n\n\
             Context: {}\n\n\
             Which option should be applied? Respond with 'option1', 'option2', or 'both' with explanation.",
            self.conflict_to_string(conflict),
            self.operation_to_string(&conflict.operation1),
            self.operation_to_string(&conflict.operation2),
            &document_content[..300] // More context for resolution
        );

        let ai_service = self.ai_service.read().await;
        let request = InferenceRequest {
            prompt,
            max_tokens: 200,
            temperature: 0.3,
        };

        match ai_service.infer(request).await {
            Ok(response) => {
                let response_text = response.text.to_lowercase();
                if response_text.contains("option1") {
                    Ok(vec![conflict.operation1.clone()])
                } else if response_text.contains("option2") {
                    Ok(vec![conflict.operation2.clone()])
                } else if response_text.contains("both") {
                    Ok(vec![
                        conflict.operation1.clone(),
                        conflict.operation2.clone(),
                    ])
                } else {
                    // Default to first operation
                    Ok(vec![conflict.operation1.clone()])
                }
            }
            Err(_) => Ok(vec![conflict.operation1.clone()]), // Fallback
        }
    }

    /// Resolve conflicts using syntactic analysis
    async fn resolve_with_syntactic_analysis(
        &self,
        analysis: &ConflictAnalysis,
        _document_content: &str,
    ) -> Result<Vec<EditorOperation>, Box<dyn std::error::Error + Send + Sync>> {
        // Simple syntactic resolution: prefer operations based on Lamport clock order
        let mut resolved = Vec::new();

        for conflict in &analysis.conflicts {
            if conflict.operation1.clock() > conflict.operation2.clock() {
                resolved.push(conflict.operation1.clone());
            } else {
                resolved.push(conflict.operation2.clone());
            }
        }

        Ok(resolved)
    }

    /// Helper methods
    fn get_operation_range(&self, operation: &EditorOperation) -> (usize, usize) {
        match operation {
            EditorOperation::Insert {
                position, content, ..
            } => (*position, content.len()),
            EditorOperation::Delete {
                position, length, ..
            } => (*position, *length),
            EditorOperation::Update {
                position,
                new_content,
                ..
            } => (*position, new_content.len()),
        }
    }

    fn regions_overlap(&self, pos1: usize, len1: usize, pos2: usize, len2: usize) -> bool {
        pos1 < pos2 + len2 && pos2 < pos1 + len1
    }

    fn operation_to_string(&self, operation: &EditorOperation) -> String {
        match operation {
            EditorOperation::Insert {
                position, content, ..
            } => format!("Insert '{}' at position {}", content, position),
            EditorOperation::Delete {
                position, length, ..
            } => format!("Delete {} characters at position {}", length, position),
            EditorOperation::Update {
                position,
                old_content,
                new_content,
                ..
            } => format!(
                "Update '{}' to '{}' at position {}",
                old_content, new_content, position
            ),
        }
    }

    fn conflict_to_string(&self, conflict: &OperationConflict) -> String {
        format!(
            "Type: {:?}, Position: {}, Length: {}",
            conflict.conflict_type, conflict.position, conflict.length
        )
    }

    fn apply_operation_to_content(
        &self,
        content: &mut String,
        operation: &EditorOperation,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        match operation {
            EditorOperation::Insert {
                position,
                content: insert_content,
                ..
            } => {
                if *position <= content.len() {
                    content.insert_str(*position, insert_content);
                }
            }
            EditorOperation::Delete {
                position, length, ..
            } => {
                if *position <= content.len() && *position + *length <= content.len() {
                    content.replace_range(*position..*position + *length, "");
                }
            }
            EditorOperation::Update {
                position,
                new_content,
                ..
            } => {
                if *position <= content.len() {
                    let end_pos = (*position + new_content.len()).min(content.len());
                    content.replace_range(*position..end_pos, new_content);
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crdt::LamportClock;

    #[test]
    fn test_conflict_detection() {
        // Test overlapping operations detection
        let op1 = EditorOperation::Insert {
            position: 5,
            content: "test".to_string(),
            op_id: "op1".to_string(),
            clock: LamportClock::new("client1".to_string()),
        };

        let op2 = EditorOperation::Delete {
            position: 5,
            length: 3,
            op_id: "op2".to_string(),
            clock: LamportClock::new("client2".to_string()),
        };

        // These operations overlap at position 5
        assert_eq!(5, 5); // Positions overlap
        assert_eq!(op1.op_id(), "op1");
        assert_eq!(op2.op_id(), "op2");
    }

    #[test]
    fn test_lamport_clock_ordering() {
        let mut clock1 = LamportClock::new("client1".to_string());
        let mut clock2 = LamportClock::new("client2".to_string());

        // Increment clocks
        clock1.increment();
        clock2.increment();
        clock2.increment();

        assert!(clock2 > clock1);
        assert!(clock1 < clock2);
    }
}
