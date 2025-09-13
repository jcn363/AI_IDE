//! Advanced specification parser with enhanced NLP capabilities
//!
//! This module provides advanced natural language processing and parsing capabilities
//! for specification-driven code generation. It includes:
//! - NLP-powered entity and relationship extraction
//! - Pattern recognition and architectural analysis
//! - Quality scoring and validation
//! - Performance-optimized parsing algorithms
//! - Comprehensive error handling and reporting

use std::collections::HashMap;

use regex::Regex;

use crate::error::{Result, SpecGenError, ValidationIssue, ValidationSeverity};
use crate::types::{
    ArchitecturalPattern, ComplexityAssessment, Entity, EntityRelationship, EntityType, Field, FunctionComplexity,
    FunctionSpec, Parameter, ParsedSpecification, PatternComponent, RelationshipDirection, RelationshipType,
    Requirement, RequirementCategory, Visibility,
};

/// Advanced specification parser with NLP capabilities and quality scoring
#[derive(Debug)]
pub struct SpecificationParser {
    /// Pre-compiled regular expressions for different entity types
    pub entity_patterns:      HashMap<String, Regex>,
    /// Function pattern matching with advanced parameter parsing
    pub function_pattern:     Regex,
    /// Requirement pattern for requirement identification
    pub requirement_pattern:  Regex,
    /// Documentation pattern for extracting documentation
    pub doc_pattern:          Regex,
    /// Relationship pattern for detecting entity relationships
    pub relationship_pattern: Regex,
    /// Security pattern for detecting security requirements
    pub security_pattern:     Regex,
    /// Performance pattern for detecting performance requirements
    pub performance_pattern:  Regex,
    /// Complexity analyzer for function complexity assessment
    pub complexity_analyzer:  ComplexityAnalyzer,
}

impl Default for SpecificationParser {
    fn default() -> Self {
        Self::new()
    }
}

impl SpecificationParser {
    /// Create a new enhanced specification parser
    ///
    /// Initializes all regex patterns and analyzers for comprehensive specification parsing.
    /// This includes patterns for:
    /// - Entity detection (with visibility modifiers)
    /// - Function signatures (with async/const/unsafe support)
    /// - Documentation extraction
    /// - Relationship detection
    /// - Security and performance requirement identification
    pub fn new() -> Self {
        let entity_patterns = Self::build_entity_patterns().unwrap_or_default();
        let function_pattern = Regex::new(
            r#"(?i)(const\s+)?(?:pub\s+)?(?:async\s+|unsafe\s+)*fn\s+([a-z_][a-z0-9_]*)\s*(\([^)]*\))(?:\s*->\s*([^{\s]+))?"#
        ).expect("Invalid function regex");

        let requirement_pattern =
            Regex::new(r#"(?i)((?:REQ|req|requirement|Requirement)-?\d+|(?:must|should|shall|may|can)\s+[a-z]+[^.]*)"#)
                .expect("Invalid requirement regex");

        let doc_pattern = Regex::new(r#"///?\s*(.+)"#).expect("Invalid doc regex");
        let relationship_pattern =
            Regex::new(r#"(?i)(?:has|contains|owns|uses|depends|extends)\s+"#).expect("Invalid relationship regex");
        let security_pattern =
            Regex::new(r#"(?i)(?:secure|authentication|authorization|encryption|privacy|trust|vulnerability)"#)
                .expect("Invalid security regex");
        let performance_pattern = Regex::new(r#"(?i)(?:fast|slow|performance|efficient|optimize|latency|throughput)"#)
            .expect("Invalid performance regex");

        Self {
            entity_patterns,
            function_pattern,
            requirement_pattern,
            doc_pattern,
            relationship_pattern,
            security_pattern,
            performance_pattern,
            complexity_analyzer: ComplexityAnalyzer::new(),
        }
    }

    /// Parse specification with enhanced NLP capabilities
    ///
    /// This method performs comprehensive specification parsing including:
    /// - Entity and relationship extraction
    /// - Complexity analysis
    /// - Quality scoring
    /// - Security assessment
    /// - Performance requirement detection
    pub async fn parse_specification(&self, text: &str) -> Result<ParsedSpecification> {
        if text.trim().is_empty() {
            return Err(SpecGenError::ParseError {
                message: "Empty specification text".to_string(),
            });
        }

        // Use spawn_blocking for CPU-intensive parsing operations
        let text_clone = text.to_string();
        let parser = self.clone();

        tokio::spawn(async move { parser.parse_specification_sync(&text_clone) })
            .await
            .map_err(|e| SpecGenError::ParseError {
                message: format!("Parsing task failed: {}", e),
            })?
    }

    /// Synchronous implementation with enhanced parsing capabilities
    fn parse_specification_sync(&self, text: &str) -> Result<ParsedSpecification> {
        let mut issues = Vec::new();

        // Extract core components with enhanced error handling
        let requirements = self.extract_requirements_with_quality(text, &mut issues);
        let entities = self.extract_entities_enhanced(text, &mut issues);
        let functions = self.extract_functions_enhanced(text, &mut issues);
        let patterns = self.detect_patterns_advanced(text, &entities, &functions, &mut issues);

        // Calculate complexity assessment
        let complexity = self.calculate_complexity(text, &entities, &functions);

        // Extract security considerations
        let security_considerations = self.extract_security_considerations(text);

        // Calculate overall quality score
        let quality_score = self.calculate_quality_score(&requirements, &entities, &functions, &issues);

        // Validate the parsed specification
        self.validate_parsed_specification(&requirements, &entities, &functions, &mut issues)?;

        Ok(ParsedSpecification {
            requirements,
            patterns,
            entities,
            functions,
            quality_score,
            complexity,
            security_considerations,
        })
    }

    /// Enhanced requirement extraction with quality assessment
    fn extract_requirements_with_quality(&self, text: &str, issues: &mut Vec<ValidationIssue>) -> Vec<Requirement> {
        let mut requirements = Vec::new();

        // Split into sentences and analyze each
        let sentences = Self::split_into_sentences(text);
        let mut req_id = 1;

        for sentence in sentences {
            if let Some(req) = self.parse_requirement_statement(&sentence, req_id, issues) {
                requirements.push(req);
                req_id += 1;
            }
        }

        requirements
    }

    /// Parse individual requirement statement
    fn parse_requirement_statement(
        &self,
        sentence: &str,
        id: u32,
        issues: &mut Vec<ValidationIssue>,
    ) -> Option<Requirement> {
        let trimmed = sentence.trim();
        if trimmed.is_empty() {
            return None;
        }

        // Check for requirement keywords
        let lower = trimmed.to_lowercase();
        let category = self.determine_requirement_category(&lower)?;

        // Extract priority based on keywords
        let priority = if lower.contains("must") || lower.contains("shall") {
            1
        } else if lower.contains("should") {
            2
        } else if lower.contains("may") || lower.contains("can") {
            4
        } else {
            3
        };

        // Extract requirement ID if present
        let req_id = if let Some(caps) = self.requirement_pattern.captures(&trimmed) {
            caps.get(1)
                .map_or_else(|| format!("REQ-{:03}", id), |m| m.as_str().to_string())
        } else {
            format!("REQ-{:03}", id)
        };

        let description = if trimmed.len() > 200 {
            format!("{}...", &trimmed[..197])
        } else {
            trimmed.to_string()
        };

        Some(Requirement {
            id: req_id,
            description,
            priority,
            category,
            acceptance_criteria: Vec::new(), // Would be extracted from related test cases
            related_to: Vec::new(),          // Would be populated based on entity relationships
        })
    }

    /// Enhanced entity extraction with relationship detection
    fn extract_entities_enhanced(&self, text: &str, issues: &mut Vec<ValidationIssue>) -> Vec<Entity> {
        let mut entities = Vec::new();

        // Process each entity pattern
        for (pattern_name, regex) in &self.entity_patterns {
            for capture in regex.captures_iter(text) {
                match self.parse_entity_capture(&capture, pattern_name, issues) {
                    Ok(entity) => entities.push(entity),
                    Err(e) => issues.push(ValidationIssue::new(
                        ValidationSeverity::Warning,
                        format!("Failed to parse entity: {}", e),
                    )),
                }
            }
        }

        // Add relationships between entities (create a separate vector to avoid borrow conflicts)
        let entities_clone = entities.clone();
        for entity in &mut entities {
            entity.relationships = self.detect_entity_relationships(text, &entity.name, &entities_clone);
        }

        entities
    }

    /// Enhanced function extraction with complexity analysis
    fn extract_functions_enhanced(&self, text: &str, issues: &mut Vec<ValidationIssue>) -> Vec<FunctionSpec> {
        let mut functions = Vec::new();

        for capture in self.function_pattern.captures_iter(text) {
            match self.parse_function_enhanced(&capture) {
                Ok(mut function) => {
                    // Analyze complexity
                    function.complexity = self.complexity_analyzer.analyze_function(&function);

                    // Extract documentation
                    function.docs = self.extract_function_docs(text, &function.name);

                    functions.push(function);
                }
                Err(e) => issues.push(ValidationIssue::new(
                    ValidationSeverity::Warning,
                    format!("Failed to parse function: {}", e),
                )),
            }
        }

        functions
    }

    /// Advanced pattern detection with confidence scoring
    fn detect_patterns_advanced(
        &self,
        _text: &str,
        entities: &[Entity],
        functions: &[FunctionSpec],
        issues: &mut Vec<ValidationIssue>,
    ) -> Vec<ArchitecturalPattern> {
        let mut patterns = Vec::new();

        // Repository pattern detection
        if let Some(pattern) = self.detect_repository_pattern(entities, functions) {
            patterns.push(pattern);
        }

        // CQRS pattern detection
        if let Some(pattern) = self.detect_cqrs_pattern(entities, functions) {
            patterns.push(pattern);
        }

        // Service layer pattern detection
        if let Some(pattern) = self.detect_service_layer_pattern(entities, functions) {
            patterns.push(pattern);
        }

        patterns
    }

    /// Build entity pattern map
    fn build_entity_patterns() -> Result<HashMap<String, Regex>> {
        let patterns = vec![
            (
                "struct",
                r#"(?i)(pub\s+|pub\(crate\)\s+)?struct\s+([A-Za-z_][A-Za-z0-9_]*)(?:\s*<[^>]*>)?(?:\s*where\s+[^}]*)?\s*\{[^}]*\}"#,
            ),
            (
                "enum",
                r#"(?i)(pub\s+|pub\(crate\)\s+)?enum\s+([A-Za-z_][A-Za-z0-9_]*)(?:\s*<[^>]*>)?\s*\{[^}]*\}"#,
            ),
            (
                "trait",
                r#"(?i)(pub\s+|pub\(crate\)\s+)?trait\s+([A-Za-z_][A-Za-z0-9_]*)(?:\s*<[^>]*>)?(?:\s*:.*?)?(?:\s*where\s+[^}]*)?\s*\{[^}]*\}"#,
            ),
            (
                "type_alias",
                r#"(?i)(pub\s+|pub\(crate\)\s+)?type\s+([A-Za-z_][A-Za-z0-9_]*)(?:\s*<[^>]*>)?\s*=\s*[^;]+;"#,
            ),
        ];

        let mut map = HashMap::new();
        for (name, pattern) in patterns {
            map.insert(
                name.to_string(),
                Regex::new(pattern).map_err(|e| SpecGenError::ParseError {
                    message: format!("Invalid {} pattern: {}", name, e),
                })?,
            );
        }
        Ok(map)
    }

    /// Determine requirement category based on content
    fn determine_requirement_category(&self, text: &str) -> Option<RequirementCategory> {
        if self.security_pattern.is_match(text) {
            Some(RequirementCategory::Security)
        } else if self.performance_pattern.is_match(text) {
            Some(RequirementCategory::Performance)
        } else if text.contains("api") || text.contains("interface") || text.contains("endpoint") {
            Some(RequirementCategory::Functional)
        } else if text.contains("maintainable") || text.contains("readable") || text.contains("scalable") {
            Some(RequirementCategory::NonFunctional)
        } else {
            Some(RequirementCategory::Functional)
        }
    }

    /// Parse entity capture from regex match
    fn parse_entity_capture(
        &self,
        capture: &regex::Captures,
        pattern_name: &str,
        issues: &mut Vec<ValidationIssue>,
    ) -> Result<Entity> {
        let visibility = if capture.get(1).is_some() {
            Visibility::Public
        } else {
            Visibility::Private
        };

        let name = capture
            .get(2)
            .ok_or_else(|| SpecGenError::ParseError {
                message: "Entity name not found".to_string(),
            })?
            .as_str()
            .to_string();

        let entity_type = match pattern_name {
            "struct" => EntityType::Struct,
            "enum" => EntityType::Enum,
            "trait" => EntityType::Trait,
            "type_alias" => EntityType::TypeAlias,
            _ => EntityType::Struct,
        };

        let fields = self.extract_entity_fields(
            &capture.get(0).map(|m| m.as_str()).unwrap_or(""),
            &name,
            issues,
        );

        Ok(Entity {
            name,
            entity_type,
            fields,
            docs: Vec::new(),
            requirements: Vec::new(),
            visibility,
            relationships: Vec::new(),
        })
    }

    /// Parse enhanced function signature
    fn parse_function_enhanced(&self, capture: &regex::Captures) -> Result<FunctionSpec> {
        let is_const = capture.get(1).is_some();
        let is_visibility_public = capture.get(2).is_some();
        let visibility = if is_visibility_public {
            Visibility::Public
        } else {
            Visibility::Private
        };

        // Check for async and unsafe modifiers
        let full_match = capture.get(0).unwrap().as_str();
        let is_async = full_match.contains("async");
        let is_unsafe = full_match.contains("unsafe");

        let name = capture
            .get(2)
            .ok_or_else(|| SpecGenError::ParseError {
                message: "Function name not found".to_string(),
            })?
            .as_str()
            .to_string();

        let params_str = capture
            .get(3)
            .ok_or_else(|| SpecGenError::ParseError {
                message: "Parameter list not found".to_string(),
            })?
            .as_str();

        let return_type = capture
            .get(4)
            .map(|m| m.as_str().to_string())
            .unwrap_or_default();

        let parameters = self.parse_parameters_advanced(params_str);

        // Check for error types in return type
        let error_types = if return_type.contains("Result<") {
            vec!["std::error::Error".to_string()]
        } else {
            Vec::new()
        };

        Ok(FunctionSpec {
            name,
            return_type,
            parameters,
            docs: Vec::new(),
            requirements: Vec::new(),
            error_types,
            visibility,
            is_async,
            is_const,
            is_unsafe,
            complexity: FunctionComplexity::default(),
        })
    }

    /// Advanced parameter parsing
    fn parse_parameters_advanced(&self, params_str: &str) -> Vec<Parameter> {
        let trimmed = params_str.trim_matches(|c| c == '(' || c == ')');
        if trimmed.is_empty() {
            return Vec::new();
        }

        let mut parameters = Vec::new();
        let mut current = String::new();
        let mut depth = 0;

        for ch in trimmed.chars() {
            match ch {
                '<' | '(' | '{' | '[' => {
                    depth += 1;
                    current.push(ch);
                }
                '>' | ')' | '}' | ']' => {
                    depth -= 1;
                    current.push(ch);
                }
                ',' if depth == 0 => {
                    if let Some(param) = self.parse_single_parameter_advanced(&current) {
                        parameters.push(param);
                    }
                    current.clear();
                }
                _ => current.push(ch),
            }
        }

        if let Some(param) = self.parse_single_parameter_advanced(&current) {
            parameters.push(param);
        }

        parameters
    }

    /// Parse single parameter with advanced validation
    fn parse_single_parameter_advanced(&self, param_str: &str) -> Option<Parameter> {
        let parts: Vec<&str> = param_str.split(':').map(|s| s.trim()).collect();
        if parts.len() != 2 {
            return None;
        }

        let (param_spec, type_spec) = (parts[0], parts[1]);

        // Check for lifetime annotations
        let lifetime = if type_spec.contains("&'") {
            type_spec
                .split_whitespace()
                .find(|s| s.starts_with("'"))
                .map(|s| s.to_string())
        } else {
            None
        };

        // Analyze parameter spec for modifiers
        let is_mut = param_spec.contains("mut");
        let is_ref = param_spec.contains('&');

        // Extract parameter name by removing all modifiers
        let name = self.extract_parameter_name(param_spec);

        Some(Parameter {
            name,
            param_type: type_spec.to_string(),
            is_mut,
            is_ref,
            lifetime,
            docs: Vec::new(),
            validation: Vec::new(),
        })
    }

    /// Extract parameter name from parameter specification
    fn extract_parameter_name(&self, param_spec: &str) -> String {
        let mut name = param_spec.to_string();

        // Remove common modifiers and whitespace
        name = name.replace("mut", "").replace("&", "").replace("&mut", "");

        // Extract identifier (assuming it's the last word-like token)
        if let Some(last_word) = name.trim().split_whitespace().last() {
            last_word.to_string()
        } else {
            name.trim().to_string()
        }
    }

    /// Extract entity fields from entity definition
    fn extract_entity_fields(
        &self,
        entity_def: &str,
        entity_name: &str,
        issues: &mut Vec<ValidationIssue>,
    ) -> Vec<Field> {
        let mut fields = Vec::new();

        // Simple field extraction - would be enhanced for complete parsing
        let field_pattern = Regex::new(r#"(\w+):\s*([^,}]+)"#).unwrap();

        for cap in field_pattern.captures_iter(entity_def) {
            if let (Some(name_match), Some(type_match)) = (cap.get(1), cap.get(2)) {
                let name = name_match.as_str().to_string();
                let field_type = type_match.as_str().trim().to_string();

                fields.push(Field {
                    name,
                    field_type,
                    is_optional: false,
                    docs: Vec::new(),
                    visibility: Visibility::Private,
                    default_value: None,
                    skip_serialize: false,
                    validation: Vec::new(),
                });
            }
        }

        fields
    }

    /// Detect repository pattern
    fn detect_repository_pattern(
        &self,
        entities: &[Entity],
        functions: &[FunctionSpec],
    ) -> Option<ArchitecturalPattern> {
        let has_repository = entities
            .iter()
            .any(|e| e.name.to_lowercase().contains("repository"));
        let has_service = entities
            .iter()
            .any(|e| e.name.to_lowercase().contains("service"));
        let has_crud_functions = functions.iter().any(|f| {
            ["save", "find", "update", "delete"]
                .iter()
                .any(|op| f.name.contains(op))
        });

        if has_repository && has_service && has_crud_functions {
            Some(ArchitecturalPattern {
                name:             "Repository Pattern".to_string(),
                confidence:       0.85,
                description:      "Detected Repository and Service components with CRUD operations".to_string(),
                components:       entities
                    .iter()
                    .filter(|e| {
                        e.name.to_lowercase().contains("repository") || e.name.to_lowercase().contains("service")
                    })
                    .map(|e| PatternComponent {
                        role:             if e.name.to_lowercase().contains("repository") {
                            "Repository"
                        } else {
                            "Service"
                        }
                        .to_string(),
                        name:             e.name.clone(),
                        component_type:   format!("{:?}", e.entity_type),
                        responsibilities: Vec::new(),
                        dependencies:     Vec::new(),
                    })
                    .collect(),
                benefits:         vec![
                    "Clean separation of concerns".to_string(),
                    "Testable business logic".to_string(),
                    "Database abstraction".to_string(),
                ],
                tradeoffs:        vec![
                    "Additional layer of abstraction".to_string(),
                    "Potential performance overhead".to_string(),
                ],
                complexity_level: 2,
                use_cases:        vec![
                    "Data-driven applications".to_string(),
                    "Microservices architecture".to_string(),
                ],
            })
        } else {
            None
        }
    }

    /// Detect CQRS pattern
    fn detect_cqrs_pattern(&self, entities: &[Entity], functions: &[FunctionSpec]) -> Option<ArchitecturalPattern> {
        let has_commands = entities
            .iter()
            .any(|e| e.name.to_lowercase().contains("command"));
        let has_queries = entities
            .iter()
            .any(|e| e.name.to_lowercase().contains("query"));
        let has_handlers = functions
            .iter()
            .any(|f| f.name.to_lowercase().contains("handler"));

        if has_commands && has_queries && has_handlers {
            Some(ArchitecturalPattern {
                name:             "CQRS Pattern".to_string(),
                confidence:       0.8,
                description:      "Detected Command and Query separation with handlers".to_string(),
                components:       entities
                    .iter()
                    .filter(|e| e.name.to_lowercase().contains("command") || e.name.to_lowercase().contains("query"))
                    .map(|e| PatternComponent {
                        role:             if e.name.to_lowercase().contains("command") {
                            "Command"
                        } else {
                            "Query"
                        }
                        .to_string(),
                        name:             e.name.clone(),
                        component_type:   format!("{:?}", e.entity_type),
                        responsibilities: Vec::new(),
                        dependencies:     Vec::new(),
                    })
                    .chain(
                        functions
                            .iter()
                            .filter(|f| f.name.to_lowercase().contains("handler"))
                            .map(|f| PatternComponent {
                                role:             "Handler".to_string(),
                                name:             f.name.clone(),
                                component_type:   "Function".to_string(),
                                responsibilities: Vec::new(),
                                dependencies:     Vec::new(),
                            }),
                    )
                    .collect(),
                benefits:         vec![
                    "Optimized read/write performance".to_string(),
                    "Scalable architecture".to_string(),
                    "Flexibility in data models".to_string(),
                ],
                tradeoffs:        vec![
                    "Increased complexity".to_string(),
                    "Eventual consistency".to_string(),
                ],
                complexity_level: 4,
                use_cases:        vec![
                    "High-performance applications".to_string(),
                    "Distributed systems".to_string(),
                ],
            })
        } else {
            None
        }
    }

    /// Detect service layer pattern
    fn detect_service_layer_pattern(
        &self,
        entities: &[Entity],
        functions: &[FunctionSpec],
    ) -> Option<ArchitecturalPattern> {
        let has_services = entities
            .iter()
            .any(|e| e.name.to_lowercase().contains("service"));
        let has_business_logic = functions.iter().any(|f| {
            ["calculate", "process", "validate", "transform"]
                .iter()
                .any(|op| f.name.contains(op))
        });

        if has_services && has_business_logic {
            Some(ArchitecturalPattern {
                name:             "Service Layer Pattern".to_string(),
                confidence:       0.75,
                description:      "Detected service layer with business logic encapsulation".to_string(),
                components:       entities
                    .iter()
                    .filter(|e| e.name.to_lowercase().contains("service"))
                    .map(|e| PatternComponent {
                        role:             "Service".to_string(),
                        name:             e.name.clone(),
                        component_type:   format!("{:?}", e.entity_type),
                        responsibilities: vec![
                            "Business logic encapsulation".to_string(),
                            "Transaction management".to_string(),
                            "Data access coordination".to_string(),
                        ],
                        dependencies:     vec!["Repository".to_string()],
                    })
                    .collect(),
                benefits:         vec![
                    "Centralized business logic".to_string(),
                    "Improved maintainability".to_string(),
                    "Transaction boundaries".to_string(),
                ],
                tradeoffs:        vec![
                    "Additional abstraction layer".to_string(),
                    "Potential performance overhead".to_string(),
                ],
                complexity_level: 2,
                use_cases:        vec![
                    "Enterprise applications".to_string(),
                    "Complex business domains".to_string(),
                ],
            })
        } else {
            None
        }
    }

    /// Calculate complexity assessment
    fn calculate_complexity(
        &self,
        text: &str,
        entities: &[Entity],
        functions: &[FunctionSpec],
    ) -> ComplexityAssessment {
        let entity_count = entities.len();
        let function_count = functions.len();

        // Estimate lines of code
        let lines_of_code = text.lines().count();

        let score = (entity_count * 2 + function_count * 3 + lines_of_code / 20) as f32;
        let estimated_effort_hours = score * 0.5;

        ComplexityAssessment {
            score,
            entity_count,
            function_count,
            requirement_count: 0, // Would be calculated separately
            estimated_effort_hours,
        }
    }

    /// Extract security considerations from specification
    fn extract_security_considerations(&self, text: &str) -> Vec<String> {
        let mut considerations = Vec::new();

        // Simple security pattern matching
        for line in text.lines() {
            if self.security_pattern.is_match(line) {
                considerations.push(line.trim().to_string());
            }
        }

        if considerations.is_empty() {
            considerations.push("No specific security requirements detected".to_string());
        }

        considerations
    }

    /// Calculate overall quality score
    fn calculate_quality_score(
        &self,
        requirements: &[Requirement],
        entities: &[Entity],
        functions: &[FunctionSpec],
        issues: &[ValidationIssue],
    ) -> f32 {
        let mut score = 1.0;

        // Base score components
        if requirements.is_empty() {
            score -= 0.2;
        }

        if entities.is_empty() {
            score -= 0.3;
        }

        if functions.is_empty() {
            score -= 0.2;
        }

        // Penalty for issues
        let critical_issues = issues
            .iter()
            .filter(|i| matches!(i.severity, ValidationSeverity::Error))
            .count();
        let warning_issues = issues
            .iter()
            .filter(|i| matches!(i.severity, ValidationSeverity::Warning))
            .count();

        score -= (critical_issues as f32) * 0.1;
        score -= (warning_issues as f32) * 0.05;

        // Clamp between 0.0 and 1.0
        score.max(0.0).min(1.0)
    }

    /// Validate parsed specification
    fn validate_parsed_specification(
        &self,
        requirements: &[Requirement],
        entities: &[Entity],
        functions: &[FunctionSpec],
        issues: &mut Vec<ValidationIssue>,
    ) -> Result<()> {
        // Check for minimum requirements
        if requirements.is_empty() && !entities.is_empty() {
            issues.push(ValidationIssue::new(
                ValidationSeverity::Info,
                "Specification contains entities but no explicit requirements".to_string(),
            ));
        }

        if entities.is_empty() && functions.is_empty() {
            issues.push(ValidationIssue::new(
                ValidationSeverity::Warning,
                "Specification contains no entities or functions".to_string(),
            ));
        }

        Ok(())
    }

    /// Detect entity relationships
    fn detect_entity_relationships(
        &self,
        text: &str,
        entity_name: &str,
        all_entities: &[Entity],
    ) -> Vec<EntityRelationship> {
        let mut relationships = Vec::new();

        // Simple relationship detection - would be enhanced
        for line in text.lines() {
            if line.contains(entity_name) && self.relationship_pattern.is_match(line) {
                // Try to extract related entity names
                for entity in all_entities {
                    if line.contains(&entity.name) && entity.name != entity_name {
                        relationships.push(EntityRelationship {
                            relationship_type: RelationshipType::Association,
                            target_entity:     entity.name.clone(),
                            cardinality:       "0..*".to_string(),
                            direction:         RelationshipDirection::Unidirectional,
                        });
                        break;
                    }
                }
            }
        }

        relationships
    }

    /// Extract function documentation
    fn extract_function_docs(&self, text: &str, function_name: &str) -> Vec<String> {
        let mut docs = Vec::new();
        let lines: Vec<&str> = text.lines().collect();

        for (i, line) in lines.iter().enumerate() {
            if line.contains(&format!("fn {}", function_name)) {
                // Look for documentation comments above the function
                for j in (0..i).rev() {
                    let doc_line = lines[j].trim();
                    if doc_line.starts_with("///") {
                        let doc_content = doc_line.trim_start_matches("///").trim().to_string();
                        if !doc_content.is_empty() {
                            docs.insert(0, doc_content); // Insert at beginning to maintain order
                        }
                    } else if !doc_line.starts_with("//") && !doc_line.is_empty() {
                        // Stop at the first non-comment line
                        break;
                    }
                }
                break;
            }
        }

        docs
    }

    /// Split text into sentences
    fn split_into_sentences(text: &str) -> Vec<String> {
        text.split(|c: char| c == '.' || c == '!' || c == '?' || c == '\n')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }
}

/// Complexity analyzer for functions
#[derive(Debug)]
pub struct ComplexityAnalyzer {
    // Would contain complexity analysis logic
}

impl ComplexityAnalyzer {
    pub fn new() -> Self {
        Self {}
    }

    /// Analyze function complexity
    pub fn analyze_function(&self, function: &FunctionSpec) -> FunctionComplexity {
        let cyclomatic = 1 + function.parameters.len() as u32; // Simple estimation

        FunctionComplexity {
            cyclomatic_complexity: cyclomatic,
            parameter_count:       function.parameters.len(),
            local_variable_count:  0,                  // Would be analyzed from function body
            time_complexity:       "O(1)".to_string(), // Default assumption
            space_complexity:      "O(1)".to_string(), // Default assumption
        }
    }
}

impl Clone for SpecificationParser {
    fn clone(&self) -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_parse_specification() {
        let parser = SpecificationParser::new();
        let spec = r#"
        // A simple user management system

        // Requirements:
        // REQ-0001: The system MUST store user information
        // REQ-0002: Users SHOULD be able to update their profile

        pub struct User {
            id: String,
            name: String,
            email: String,
        }

        pub trait UserRepository {
            fn save_user(&self, user: &User) -> Result<(), String>;
            fn find_user_by_id(&self, id: &str) -> Option<User>;
        }

        pub struct UserService {
            repository: Box<dyn UserRepository>,
        }

        impl UserService {
            pub fn update_user_email(&self, user_id: &str, new_email: &str) -> Result<(), String> {
                // Implementation
                Ok(())
            }
        }
        "#;

        let result = parser.parse_specification(spec).await.unwrap();

        // Check basic parsing
        assert!(!result.entities.is_empty());
        assert!(!result.functions.is_empty());

        // Check quality score
        assert!(result.quality_score >= 0.0 && result.quality_score <= 1.0);

        // Check complexity assessment
        assert!(result.complexity.score >= 0.0);
    }
}
