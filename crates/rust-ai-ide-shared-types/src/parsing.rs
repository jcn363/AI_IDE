//! AST parsing and type extraction from Rust code
//!
//! This module provides comprehensive parsing capabilities to extract
//! type information from Rust source files using the syn crate.

use crate::errors::TypeGenerationError;
use crate::types::*;
use std::collections::HashMap;
use syn::{Attribute, Fields, File, Item, Type, Variant as SynVariant, Visibility};

/// Parser for extracting types from Rust source code
#[derive(Debug, Clone)]
pub struct TypeParser {
    /// Configuration for parsing behavior
    pub config: ParserConfig,
}

/// Configuration for the type parser
#[derive(Debug, Clone)]
pub struct ParserConfig {
    /// Include private types
    pub include_private: bool,

    /// Include documentation
    pub include_docs: bool,

    /// Skip types with certain attributes
    pub skip_attributes: Vec<String>,

    /// Custom parsing rules
    pub custom_rules: HashMap<String, String>,
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            include_private: false,
            include_docs: true,
            skip_attributes: vec!["skip_typescript".to_string()],
            custom_rules: HashMap::new(),
        }
    }
}

impl TypeParser {
    /// Create a new type parser with default configuration
    pub fn new() -> Self {
        Self {
            config: ParserConfig::default(),
        }
    }

    /// Create a new type parser with custom configuration
    pub fn with_config(config: ParserConfig) -> Self {
        Self { config }
    }

    /// Parse a Rust source file and extract type information
    pub fn parse_file(
        &self,
        source: &str,
        file_path: &str,
    ) -> Result<Vec<ParsedType>, TypeGenerationError> {
        let ast: File = syn::parse_str(source).map_err(|e| {
            TypeGenerationError::AnalysisError(format!("Failed to parse Rust file: {}", e))
        })?;

        let mut types = Vec::new();

        for item in &ast.items {
            if let Some(parsed_type) = self.parse_item(item, file_path)? {
                types.push(parsed_type);
            }
        }

        Ok(types)
    }

    /// Parse a single item from the AST
    fn parse_item(
        &self,
        item: &Item,
        file_path: &str,
    ) -> Result<Option<ParsedType>, TypeGenerationError> {
        match item {
            Item::Struct(item_struct) => {
                if self.should_skip_item(&item_struct.attrs) {
                    return Ok(None);
                }
                Ok(Some(self.parse_struct(item_struct, file_path)))
            }
            Item::Enum(item_enum) => {
                if self.should_skip_item(&item_enum.attrs) {
                    return Ok(None);
                }
                Ok(Some(self.parse_enum(item_enum, file_path)))
            }
            Item::Type(item_type) => {
                if self.should_skip_item(&item_type.attrs) {
                    return Ok(None);
                }
                Ok(Some(self.parse_type_alias(item_type, file_path)))
            }
            Item::Union(item_union) => {
                if self.should_skip_item(&item_union.attrs) {
                    return Ok(None);
                }
                Ok(Some(self.parse_union(item_union, file_path)))
            }
            _ => Ok(None), // Skip other items
        }
    }

    /// Parse a struct definition
    fn parse_struct(&self, item_struct: &syn::ItemStruct, file_path: &str) -> ParsedType {
        let name = item_struct.ident.to_string();
        let visibility = self.convert_visibility(&item_struct.vis);
        let generics = self.parse_generics(&item_struct.generics);
        let fields = self.parse_struct_fields(&item_struct.fields);
        let attributes = self.parse_attributes(&item_struct.attrs);

        ParsedType {
            name,
            kind: TypeKind::Struct,
            documentation: self.extract_documentation(&item_struct.attrs),
            visibility,
            generics,
            fields,
            variants: vec![],
            associated_items: vec![],
            attributes,
            source_location: self.extract_source_location(file_path, item_struct.struct_token.span),
            dependencies: self.extract_dependencies(&item_struct.fields),
            metadata: Some(TypeMetadata::default()),
        }
    }

    /// Parse an enum definition
    fn parse_enum(&self, item_enum: &syn::ItemEnum, file_path: &str) -> ParsedType {
        let name = item_enum.ident.to_string();
        let visibility = self.convert_visibility(&item_enum.vis);
        let generics = self.parse_generics(&item_enum.generics);
        let variants = self.parse_enum_variants(&item_enum.variants);
        let attributes = self.parse_attributes(&item_enum.attrs);

        ParsedType {
            name,
            kind: TypeKind::Enum,
            documentation: self.extract_documentation(&item_enum.attrs),
            visibility,
            generics,
            fields: vec![],
            variants,
            associated_items: vec![],
            attributes,
            source_location: self.extract_source_location(file_path, item_enum.enum_token.span),
            dependencies: self.extract_dependencies_from_variants(&item_enum.variants),
            metadata: Some(TypeMetadata::default()),
        }
    }

    /// Parse a type alias
    fn parse_type_alias(&self, item_type: &syn::ItemType, file_path: &str) -> ParsedType {
        let name = item_type.ident.to_string();
        let visibility = self.convert_visibility(&item_type.vis);
        let generics = self.parse_generics(&item_type.generics);
        let attributes = self.parse_attributes(&item_type.attrs);

        ParsedType {
            name,
            kind: TypeKind::TypeAlias,
            documentation: self.extract_documentation(&item_type.attrs),
            visibility,
            generics,
            fields: vec![],
            variants: vec![],
            associated_items: vec![],
            attributes,
            source_location: self.extract_source_location(file_path, item_type.type_token.span),
            dependencies: vec![self.type_to_string(&item_type.ty)],
            metadata: Some(TypeMetadata::default()),
        }
    }

    /// Parse a union definition
    fn parse_union(&self, item_union: &syn::ItemUnion, file_path: &str) -> ParsedType {
        let name = item_union.ident.to_string();
        let visibility = self.convert_visibility(&item_union.vis);
        let generics = self.parse_generics(&item_union.generics);
        let fields = self.parse_struct_fields(&Fields::Named(item_union.fields.clone()));
        let attributes = self.parse_attributes(&item_union.attrs);

        ParsedType {
            name,
            kind: TypeKind::Union,
            documentation: self.extract_documentation(&item_union.attrs),
            visibility,
            generics,
            fields,
            variants: vec![],
            associated_items: vec![],
            attributes,
            source_location: self.extract_source_location(file_path, item_union.union_token.span),
            dependencies: self.extract_dependencies(&Fields::Named(item_union.fields.clone())),
            metadata: Some(TypeMetadata::default()),
        }
    }

    /// Check if an item should be skipped based on attributes
    fn should_skip_item(&self, attrs: &[Attribute]) -> bool {
        for attr in attrs {
            let attr_name = attr
                .path()
                .segments
                .iter()
                .map(|seg| seg.ident.to_string())
                .collect::<Vec<_>>()
                .join("::");

            if self.config.skip_attributes.contains(&attr_name) {
                return true;
            }
        }
        false
    }

    /// Convert syn visibility to our Visibility enum
    fn convert_visibility(&self, vis: &Visibility) -> crate::types::Visibility {
        use crate::types::Visibility as TypesVisibility;
        match vis {
            Visibility::Public(_) => TypesVisibility::Public,
            Visibility::Restricted(restricted) => {
                // Check if it's crate-level restricted visibility
                if restricted.path.segments.len() == 1
                    && restricted.path.segments[0].ident == "crate"
                {
                    TypesVisibility::Crate
                } else {
                    TypesVisibility::Module
                }
            }
            Visibility::Inherited => TypesVisibility::Private,
        }
    }

    /// Parse generic parameters
    fn parse_generics(&self, generics: &syn::Generics) -> Vec<String> {
        generics
            .params
            .iter()
            .filter_map(|param| match param {
                syn::GenericParam::Type(type_param) => Some(type_param.ident.to_string()),
                syn::GenericParam::Const(const_param) => Some(const_param.ident.to_string()),
                syn::GenericParam::Lifetime(lifetime_param) => {
                    Some(lifetime_param.lifetime.ident.to_string())
                }
            })
            .collect()
    }

    /// Parse struct fields
    fn parse_struct_fields(&self, fields: &Fields) -> Vec<Field> {
        match fields {
            Fields::Named(fields_named) => {
                fields_named
                    .named
                    .iter()
                    .map(|field| {
                        let type_str = self.type_to_string(&field.ty);
                        Field {
                            name: field.ident.as_ref().unwrap().to_string(),
                            ty: type_str.clone(),
                            field_type: type_str,
                            documentation: self.extract_documentation(&field.attrs),
                            visibility: self.convert_visibility(&field.vis),
                            is_mutable: false, // Can't determine from struct fields
                            attributes: self.parse_attributes(&field.attrs),
                        }
                    })
                    .collect()
            }
            Fields::Unnamed(fields_unnamed) => fields_unnamed
                .unnamed
                .iter()
                .enumerate()
                .map(|(index, field)| {
                    let type_str = self.type_to_string(&field.ty);
                    Field {
                        name: format!("field_{}", index),
                        ty: type_str.clone(),
                        field_type: type_str,
                        documentation: self.extract_documentation(&field.attrs),
                        visibility: self.convert_visibility(&field.vis),
                        is_mutable: false,
                        attributes: self.parse_attributes(&field.attrs),
                    }
                })
                .collect(),
            Fields::Unit => vec![],
        }
    }

    /// Parse enum variants
    fn parse_enum_variants(
        &self,
        variants: &syn::punctuated::Punctuated<SynVariant, syn::Token![,]>,
    ) -> Vec<Variant> {
        variants
            .iter()
            .map(|variant| {
                let fields = match &variant.fields {
                    Fields::Named(fields_named) => fields_named
                        .named
                        .iter()
                        .map(|field| {
                            let type_str = self.type_to_string(&field.ty);
                            VariantField {
                                name: field.ident.as_ref().map(|id| id.to_string()),
                                ty: type_str.clone(),
                                field_type: type_str,
                            }
                        })
                        .collect(),
                    Fields::Unnamed(fields_unnamed) => fields_unnamed
                        .unnamed
                        .iter()
                        .map(|field| {
                            let type_str = self.type_to_string(&field.ty);
                            VariantField {
                                name: None,
                                ty: type_str.clone(),
                                field_type: type_str,
                            }
                        })
                        .collect(),
                    Fields::Unit => vec![],
                };

                Variant {
                    name: variant.ident.to_string(),
                    documentation: self.extract_documentation(&variant.attrs),
                    fields,
                    discriminant: variant
                        .discriminant
                        .as_ref()
                        .map(|(_, expr)| self.expr_to_string(expr)),
                    attributes: self.parse_attributes(&variant.attrs),
                }
            })
            .collect()
    }

    /// Extract documentation from attributes
    fn extract_documentation(&self, attrs: &[Attribute]) -> Option<String> {
        if !self.config.include_docs {
            return None;
        }

        let mut docs = Vec::new();

        for attr in attrs {
            if attr.path().is_ident("doc") {
                if let syn::Meta::NameValue(meta) = &attr.meta {
                    if let syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(lit_str),
                        ..
                    }) = &meta.value
                    {
                        docs.push(lit_str.value());
                    }
                }
            }
        }

        if docs.is_empty() {
            None
        } else {
            Some(docs.join("\n"))
        }
    }

    /// Parse attributes into strings
    fn parse_attributes(&self, attrs: &[Attribute]) -> Vec<String> {
        attrs
            .iter()
            .map(|attr| {
                attr.path()
                    .segments
                    .iter()
                    .map(|seg| seg.ident.to_string())
                    .collect::<Vec<_>>()
                    .join("::")
            })
            .collect()
    }

    /// Extract source location information
    fn extract_source_location(&self, file_path: &str, span: proc_macro2::Span) -> SourceLocation {
        // Note: In a real implementation, you'd use span information to get exact line/column
        // For now, we'll use placeholder values based on the file path
        SourceLocation {
            file: file_path.to_string(),
            file_path: file_path.to_string(),
            line: 0, // proc_macro2::Span doesn't have start() in the same way syn spans do
            column: 0,
            module_path: vec![], // Would need more context to determine module path
        }
    }

    /// Extract dependencies from fields
    fn extract_dependencies(&self, fields: &Fields) -> Vec<String> {
        let mut deps = Vec::new();

        match fields {
            Fields::Named(fields_named) => {
                for field in &fields_named.named {
                    deps.extend(self.extract_types_from_type(&field.ty));
                }
            }
            Fields::Unnamed(fields_unnamed) => {
                for field in &fields_unnamed.unnamed {
                    deps.extend(self.extract_types_from_type(&field.ty));
                }
            }
            Fields::Unit => {}
        }

        deps
    }

    /// Extract dependencies from enum variants
    fn extract_dependencies_from_variants(
        &self,
        variants: &syn::punctuated::Punctuated<SynVariant, syn::Token![,]>,
    ) -> Vec<String> {
        let mut deps = Vec::new();

        for variant in variants {
            match &variant.fields {
                Fields::Named(fields_named) => {
                    for field in &fields_named.named {
                        deps.extend(self.extract_types_from_type(&field.ty));
                    }
                }
                Fields::Unnamed(fields_unnamed) => {
                    for field in &fields_unnamed.unnamed {
                        deps.extend(self.extract_types_from_type(&field.ty));
                    }
                }
                Fields::Unit => {}
            }
        }

        deps
    }

    /// Extract type names from a type
    fn extract_types_from_type(&self, ty: &Type) -> Vec<String> {
        match ty {
            Type::Path(type_path) => {
                if let Some(segment) = type_path.path.segments.last() {
                    vec![segment.ident.to_string()]
                } else {
                    vec![]
                }
            }
            Type::Array(type_array) => self.extract_types_from_type(&type_array.elem),
            Type::Slice(type_slice) => self.extract_types_from_type(&type_slice.elem),
            Type::Ptr(type_ptr) => self.extract_types_from_type(&type_ptr.elem),
            Type::Reference(type_ref) => self.extract_types_from_type(&type_ref.elem),
            _ => vec![], // Skip complex types for now
        }
    }

    /// Convert a type to string representation
    fn type_to_string(&self, ty: &Type) -> String {
        quote::quote!(#ty).to_string()
    }

    /// Convert a bound to string representation
    fn bound_to_string(&self, bound: &syn::TypeParamBound) -> String {
        match bound {
            syn::TypeParamBound::Trait(trait_bound) => {
                // Convert path to type-like structure for type_to_string
                let path_type = Type::Path(syn::TypePath {
                    qself: None,
                    path: trait_bound.path.clone(),
                });
                self.type_to_string(&path_type)
            }
            syn::TypeParamBound::Lifetime(lifetime) => lifetime.ident.to_string(),
            syn::TypeParamBound::PreciseCapture(_) => "precise".to_string(), // Placeholder
            _ => "unknown".to_string(), // Handle non-exhaustive pattern
        }
    }

    /// Convert an expression to string representation
    fn expr_to_string(&self, expr: &syn::Expr) -> String {
        quote::quote!(#expr).to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_struct() {
        let source = r#"
            /// A simple test struct
            pub struct TestStruct {
                /// The name field
                pub name: String,
                /// The age field
                pub age: i32,
            }
        "#;

        let parser = TypeParser::new();
        let types = parser.parse_file(source, "test.rs").unwrap();

        assert_eq!(types.len(), 1);
        let ty = &types[0];
        assert_eq!(ty.name, "TestStruct");
        assert_eq!(ty.kind, TypeKind::Struct);
        assert_eq!(ty.fields.len(), 2);
        assert_eq!(ty.fields[0].name, "name");
        assert_eq!(ty.fields[1].name, "age");
    }

    #[test]
    fn test_parse_enum() {
        let source = r#"
            /// Test enum
            pub enum TestEnum {
                /// Variant A
                VariantA,
                /// Variant B with data
                VariantB(String),
                /// Variant C with named fields
                VariantC {
                    /// Field documentation
                    field: i32
                },
            }
        "#;

        let parser = TypeParser::new();
        let types = parser.parse_file(source, "test.rs").unwrap();

        assert_eq!(types.len(), 1);
        let ty = &types[0];
        assert_eq!(ty.name, "TestEnum");
        assert_eq!(ty.kind, TypeKind::Enum);
        assert_eq!(ty.variants.len(), 3);
    }

    #[test]
    fn test_skip_attribute() {
        let source = r#"
            #[skip_typescript]
            pub struct SkipMe {
                pub field: String,
            }
        "#;

        let parser = TypeParser::new();
        let types = parser.parse_file(source, "test.rs").unwrap();

        assert_eq!(types.len(), 0);
    }

    #[test]
    fn test_extract_documentation() {
        let source = r#"
            /// This is a test struct
            /// with multiple lines
            pub struct TestStruct {
                pub field: String,
            }
        "#;

        let parser = TypeParser::new();
        let types = parser.parse_file(source, "test.rs").unwrap();

        assert_eq!(types.len(), 1);
        let ty = &types[0];
        assert!(ty
            .documentation
            .as_ref()
            .unwrap()
            .contains("This is a test struct"));
    }

    #[test]
    fn test_parser_config() {
        let config = ParserConfig {
            include_private: true,
            include_docs: false,
            skip_attributes: vec!["custom_skip".to_string()],
            custom_rules: HashMap::new(),
        };

        let parser = TypeParser::with_config(config);
        assert!(!parser.config.include_docs);
    }
}
