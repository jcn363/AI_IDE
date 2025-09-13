//! DSL Parser implementation using pest

use pest::Parser;
use pest_derive::Parser;

use crate::ast::*;
use crate::error::{DslError, DslResult};

/// Convert serde_json::Value to MetadataValue
fn convert_json_value(value: serde_json::Value) -> MetadataValue {
    match value {
        serde_json::Value::String(s) => MetadataValue::String(s),
        serde_json::Value::Number(n) => MetadataValue::Number(n.as_f64().unwrap_or(0.0)),
        serde_json::Value::Bool(b) => MetadataValue::Boolean(b),
        serde_json::Value::Array(arr) => MetadataValue::Array(arr.into_iter().map(convert_json_value).collect()),
        serde_json::Value::Object(obj) => {
            let mut map = std::collections::HashMap::new();
            for (k, v) in obj {
                map.insert(k, convert_json_value(v));
            }
            MetadataValue::Object(map)
        }
        serde_json::Value::Null => MetadataValue::String(String::new()), // Default to string for null
    }
}

#[derive(Parser)]
#[grammar_inline = r#"
// DSL Grammar for Code Generation Templates
//
// This grammar supports:
// - Template definitions with parameters and guards
// - Template content with interpolation and conditionals
// - Metadata attachments and pattern matching
//
// Example DSL:
//
// template FunctionTemplate {
//     name: "simple_function"
//     description: "Generate a simple function"
//
//     parameters: {
//         name: String!
//         return_type: String!
//         params: [String]!
//     }
//
//     guard: "{{if language == ProgrammingLanguage::Rust}}"
//
//     generate: {
//         content: """
// fn {{name}}({{params.join(", ")}}) -> {{return_type}} {
//     // Generated function
// }
//         """
//     }
// }

// document = { SOI ~ (template | metadata | NEWLINE)* ~ EOI }
document = { (template | metadata | NEWLINE_LINE)* }

// ===== Templates ===================================================================
template = {
    "template" ~ identifier ~ "{" ~
        (template_property | NEWLINE_LINE)* ~
    "}"
}

template_property = {
    name_property |
    description_property |
    parameters_property |
    guards_property |
    generate_property |
    patterns_property |
    metadata_property
}

name_property = { "name:" ~ quoted_string }
description_property = { "description:" ~ quoted_string }

// ===== Parameters ==================================================================
parameters_property = { "parameters:" ~ "{" ~ (parameter_definition ~ (","? ~ NEWLINE_LINE)?)* ~ "}" }

parameter_definition = {
    identifier ~ ":" ~
    parameter_type ~
    parameter_required? ~
    parameter_description?
}

parameter_type = {
    "String" |
    "Integer" |
    "Boolean" |
    "Float" |
    array_type |
    custom_type
}

array_type = { "[" ~ parameter_type ~ "]" }
custom_type = { identifier }

parameter_required = { "!" }
parameter_description = { "//" ~ quoted_string }

// ===== Guards =======================================================================
guards_property = { "guard:" ~ template_string }

// ===== Generation ===================================================================
generate_property = {
    "generate:" ~ "{" ~
        (generate_property_item | NEWLINE_LINE)* ~
    "}"
}

generate_property_item = {
    generate_content |
    generate_validation
}

generate_content = { "content:" ~ template_string }
generate_validation = { "validation:" ~ validation_rule }

validation_rule = {
    identifier ~ ":" ~ quoted_string
}

// ===== Template Content =============================================================
template_string = { triple_quoted_string | quoted_string }
triple_quoted_string = { "\"\"\"" ~ (!"\"\"\"" ~ ANY)* ~ "\"\"\"" }
quoted_string = { "\"" ~ (!"\"" ~ ANY)* ~ "\"" }

// ===== Identifiers =================================================================
identifier = { ASCII_ALPHA ~ (ASCII_ALPHANUMERIC | "_")* }

// ===== Metadata ====================================================================
metadata = { "@" ~ identifier ~ literal }
metadata_property = { "@" ~ identifier ~ literal }
literal = { quoted_string | number_literal | boolean_literal | "null" }
number_literal = { ASCII_DIGIT+ ~ ("." ~ ASCII_DIGIT+)? }
boolean_literal = { "true" | "false" }

// ===== Patterns (extended later) ===================================================
patterns_property = { "patterns:" ~ quoted_string }

// ===== Comments and Whitespace ======================================================
WHITESPACE = _{ " " | "\t" | "\r" | "\n" }
NEWLINE_LINE = _{ "\n" | "\r\n" | "\r" }

"#]
struct DslParser;

/// Main parser struct for DSL documents
pub struct DslDocumentParser;

impl DslDocumentParser {
    /// Parse a DSL document from a string
    pub fn parse(input: &str) -> DslResult<DslDocument> {
        let pairs = DslParser::parse(Rule::document, input).map_err(|e| {
            DslError::parse(
                format!("Parse error: {}", e),
                1, // Default line
                1, // Default column
                "".to_string(),
            )
        })?;

        let mut document = DslDocument::default();

        for pair in pairs {
            match pair.as_rule() {
                Rule::template => {
                    document.templates.push(self::parse_template(pair)?);
                }
                Rule::metadata => {
                    document.metadata.push(self::parse_metadata(pair)?);
                }
                _ => {} // Ignore other rules at document level
            }
        }

        Ok(document)
    }

    /// Parse a single template from the DSL
    pub fn parse_template_str(input: &str) -> DslResult<Template> {
        let pairs = DslParser::parse(Rule::template, input).map_err(|e| {
            DslError::parse(
                format!("Template parse error: {}", e),
                1, // Default line
                1, // Default column
                "".to_string(),
            )
        })?;

        // Should have exactly one template
        let template_pair = pairs
            .find_first_tagged("template")
            .ok_or_else(|| DslError::parse("No template found", 1, 1, input))?;

        self::parse_template(template_pair)
    }
}

/// Parse a complete template from a pest pair
fn parse_template(pair: pest::iterators::Pair<Rule>) -> DslResult<Template> {
    let mut template = Template::new("");
    let position = parse_position(&pair);

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::identifier => {
                template.name = inner.as_str().to_string();
            }
            Rule::template_property => {
                parse_template_property(inner, &mut template)?;
            }
            _ => {}
        }
    }

    template.location = position;
    Ok(template)
}

/// Parse individual template properties
fn parse_template_property(pair: pest::iterators::Pair<Rule>, template: &mut Template) -> DslResult<()> {
    match pair.as_rule() {
        Rule::name_property =>
            for inner in pair.into_inner() {
                if inner.as_rule() == Rule::quoted_string {
                    template.name = parse_quoted_string(inner)?;
                }
            },
        Rule::description_property =>
            for inner in pair.into_inner() {
                if inner.as_rule() == Rule::quoted_string {
                    template.description = Some(parse_quoted_string(inner)?);
                }
            },
        Rule::parameters_property => {
            template.parameters = parse_parameters(pair)?;
        }
        Rule::guards_property => {
            template.guards = vec![Guard {
                condition: Expression::Literal(Literal::String("true".to_string())), // Simplified
                location:  None,
            }];
        }
        Rule::generate_property => {
            template.generate.content.parts = vec![ContentPart::Literal("template_generation".to_string())];
        }
        Rule::patterns_property => {
            template.patterns = vec!["default".to_string()];
        }
        Rule::metadata_property => {
            template.metadata.push(Metadata {
                key:   "default".to_string(),
                value: MetadataValue::String("value".to_string()),
            });
        }
        _ => {}
    }

    Ok(())
}

/// Parse parameters block - simplified for basic functionality
fn parse_parameters(pair: pest::iterators::Pair<Rule>) -> DslResult<Vec<Parameter>> {
    let mut parameters = Vec::new();

    for param in pair.into_inner() {
        if param.as_rule() == Rule::parameter_definition {
            parameters.push(parse_parameter(param)?);
        }
    }

    Ok(parameters)
}

/// Parse a single parameter - simplified
fn parse_parameter(pair: pest::iterators::Pair<Rule>) -> DslResult<Parameter> {
    let mut param = Parameter {
        name:          String::new(),
        param_type:    ParameterType::String,
        required:      false,
        default_value: None,
        description:   None,
    };

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::identifier => {
                param.name = inner.as_str().to_string();
            }
            Rule::parameter_type => {
                param.param_type = match inner.as_str() {
                    "String" => ParameterType::String,
                    "Integer" => ParameterType::Integer,
                    "Boolean" => ParameterType::Boolean,
                    "Float" => ParameterType::Float,
                    s if s.starts_with('[') && s.ends_with(']') => {
                        let inner_type = s.trim_matches(|c| c == '[' || c == ']');
                        match inner_type {
                            "String" => ParameterType::Array(Box::new(ParameterType::String)),
                            _ => ParameterType::Array(Box::new(ParameterType::String)), // Default
                        }
                    }
                    custom => ParameterType::Custom(custom.to_string()),
                };
            }
            Rule::parameter_required => {
                param.required = true;
            }
            _ => {}
        }
    }

    Ok(param)
}

/// Parse guards - simplified
fn parse_guards(_pair: pest::iterators::Pair<Rule>) -> DslResult<Vec<Guard>> {
    Ok(vec![Guard {
        condition: Expression::Literal(Literal::Boolean(true)),
        location:  None,
    }])
}

/// Parse generate block - simplified
fn parse_generate_block(pair: pest::iterators::Pair<Rule>) -> DslResult<GenerateBlock> {
    let mut block = GenerateBlock::default();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::generate_content =>
                for content_inner in inner.into_inner() {
                    if content_inner.as_rule() == Rule::quoted_string {
                        block.content.parts = vec![ContentPart::Literal(parse_quoted_string(content_inner)?)];
                    }
                },
            _ => {}
        }
    }

    Ok(block)
}

/// Parse validation rule - simplified
fn parse_validation_rule(pair: pest::iterators::Pair<Rule>) -> DslResult<ValidationRule> {
    use crate::ast::{ValidationRule, ValidationSeverity};

    let parts: Vec<_> = pair.into_inner().collect();
    let name = parts
        .get(0)
        .and_then(|p| {
            if p.as_rule() == Rule::identifier {
                Some(p.as_str())
            } else {
                None
            }
        })
        .unwrap_or("unknown");
    let rule = parts
        .get(1)
        .and_then(|p| {
            if p.as_rule() == Rule::quoted_string {
                Some(parse_quoted_string(p.clone()).unwrap())
            } else {
                None
            }
        })
        .unwrap_or_default();

    Ok(ValidationRule {
        name: name.to_string(),
        rule,
        severity: ValidationSeverity::Warning,
    })
}

/// Parse template string - simplified
fn parse_template_string(pair: pest::iterators::Pair<Rule>) -> DslResult<String> {
    let result = pair.as_str().to_string();
    for inner in pair.into_inner() {
        if matches!(
            inner.as_rule(),
            Rule::triple_quoted_string | Rule::quoted_string
        ) {
            return parse_quoted_string(inner);
        }
    }
    Ok(result)
}

/// Parse metadata - simplified
fn parse_metadata(pair: pest::iterators::Pair<Rule>) -> DslResult<Metadata> {
    let mut key = "default".to_string();
    let mut value = MetadataValue::String("default".to_string());

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::identifier => {
                key = inner.as_str().to_string();
            }
            Rule::literal => {
                value = MetadataValue::String(inner.as_str().to_string());
            }
            _ => {}
        }
    }

    Ok(Metadata { key, value })
}

/// Parse quoted string
fn parse_quoted_string(pair: pest::iterators::Pair<Rule>) -> DslResult<String> {
    let raw = pair.as_str();

    // Handle both triple and single quoted strings
    if raw.starts_with("\"\"\"") && raw.ends_with("\"\"\"") {
        Ok(raw.trim_matches('\"').to_string())
    } else if raw.starts_with('"') && raw.ends_with('"') {
        Ok(raw.trim_matches('"').to_string())
    } else {
        Ok(raw.to_string())
    }
}

/// Parse position information - simplified
fn parse_position(_pair: &pest::iterators::Pair<Rule>) -> Option<Location> {
    Some(Location {
        line:   1,
        column: 1,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_template_parsing() {
        let dsl = r#"
            template SimpleFunction {
                name: "test_function"
                description: "A simple test function"

                generate: {
                    content: "fn {{name}}() { println!(\"Hello\"); }"
                }
            }
        "#;

        let result = DslDocumentParser::parse(dsl);
        assert!(result.is_ok(), "Should parse basic template");

        let doc = result.unwrap();
        assert_eq!(doc.templates.len(), 1);
        assert_eq!(doc.templates[0].name, "test_function");
    }
}
