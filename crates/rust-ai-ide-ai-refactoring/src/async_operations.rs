use std::fs;

use async_trait::async_trait;
use prettyplease;
use syn::visit_mut::VisitMut;
use syn::Ident;

use crate::ast_utils::*;
use crate::types::*;
use crate::utils::*;
use crate::RefactoringOperation;

/// Convert to Async operation - converts a function to async
pub struct ConvertToAsyncOperation;

/// Analyzed function information for async conversion
struct AsyncConversionInfo {
    function_name: String,
    function_span: proc_macro2::Span,
    is_already_async: bool,
    return_type: Option<syn::Type>,
    calls_awaitable: bool,
    callees: Vec<String>,
}

impl ConvertToAsyncOperation {
    /// Analyze a function to determine if it should be converted to async
    fn analyze_function_for_async(
        &self,
        syntax: &syn::File,
        function_name: &str,
    ) -> Result<AsyncConversionInfo, Box<dyn std::error::Error + Send + Sync>> {
        let mut is_already_async = false;
        let mut return_type = None;
        let mut calls_awaitable = false;
        let mut callees = Vec::new();
        let mut function_span = proc_macro2::Span::call_site();

        for item in &syntax.items {
            if let syn::Item::Fn(item_fn) = item {
                if item_fn.sig.ident == function_name {
                    function_span = item_fn.sig.ident.span();
                    is_already_async = item_fn.sig.asyncness.is_some();
                    return_type = match &item_fn.sig.output {
                        syn::ReturnType::Default => None,
                        syn::ReturnType::Type(_, ty) => Some(*ty.clone()),
                    };

                    // Analyze function body for async operations
                    self.analyze_block_for_async(
                        &item_fn.block,
                        &mut calls_awaitable,
                        &mut callees,
                    );

                    break;
                }
            } else if let syn::Item::Impl(impl_block) = item {
                for impl_item in &impl_block.items {
                    if let syn::ImplItem::Fn(method) = impl_item {
                        if method.sig.ident == function_name {
                            function_span = method.sig.ident.span();
                            is_already_async = method.sig.asyncness.is_some();
                            return_type = match &method.sig.output {
                                syn::ReturnType::Default => None,
                                syn::ReturnType::Type(_, ty) => Some(*ty.clone()),
                            };

                            self.analyze_block_for_async(
                                &method.block,
                                &mut calls_awaitable,
                                &mut callees,
                            );

                            break;
                        }
                    }
                }
            }
        }

        Ok(AsyncConversionInfo {
            function_name: function_name.to_string(),
            function_span,
            is_already_async,
            return_type,
            calls_awaitable,
            callees,
        })
    }

    /// Analyze function body for async operations
    fn analyze_block_for_async(
        &self,
        block: &syn::Block,
        calls_awaitable: &mut bool,
        callees: &mut Vec<String>,
    ) {
        for stmt in &block.stmts {
            match stmt {
                syn::Stmt::Expr(expr, _) => {
                    self.analyze_expr_for_async(expr, calls_awaitable, callees);
                }
                syn::Stmt::Local(local) => {
                    if let Some(init) = &local.init {
                        self.analyze_expr_for_async(&init.expr, calls_awaitable, callees);
                    }
                }
                _ => {}
            }
        }
    }

    /// Analyze expression for async operations
    fn analyze_expr_for_async(
        &self,
        expr: &syn::Expr,
        calls_awaitable: &mut bool,
        callees: &mut Vec<String>,
    ) {
        match expr {
            syn::Expr::Await(await_expr) => {
                *calls_awaitable = true;
                self.analyze_expr_for_async(&await_expr.base, calls_awaitable, callees);
            }
            syn::Expr::Call(call) => {
                if let syn::Expr::Path(path) = &*call.func {
                    if let Some(ident) = path.path.get_ident() {
                        callees.push(ident.to_string());
                        // Consider adding heuristics for known async functions
                        if matches!(
                            ident.to_string().as_str(),
                            "tokio" | "async" | "sleep" | "spawn" | "join" | "select"
                        ) {
                            *calls_awaitable = true;
                        }
                    }
                }
                for arg in &call.args {
                    self.analyze_expr_for_async(arg, calls_awaitable, callees);
                }
            }
            syn::Expr::MethodCall(method_call) => {
                callees.push(method_call.method.to_string());
                self.analyze_expr_for_async(&method_call.receiver, calls_awaitable, callees);
                for arg in &method_call.args {
                    self.analyze_expr_for_async(arg, calls_awaitable, callees);
                }
            }
            syn::Expr::Block(block) => {
                self.analyze_block_for_async(&block.block, calls_awaitable, callees);
            }
            syn::Expr::If(if_expr) => {
                self.analyze_expr_for_async(&if_expr.cond, calls_awaitable, callees);
                self.analyze_block_for_async(&if_expr.then_branch, calls_awaitable, callees);
                for else_branch in &if_expr.else_branch {
                    if let syn::Expr::Block(else_block) = else_branch.1.as_ref() {
                        self.analyze_block_for_async(&else_block.block, calls_awaitable, callees);
                    }
                }
            }
            syn::Expr::Loop(loop_expr) => {
                self.analyze_block_for_async(&loop_expr.body, calls_awaitable, callees);
            }
            syn::Expr::Match(match_expr) => {
                self.analyze_expr_for_async(&match_expr.expr, calls_awaitable, callees);
                for arm in &match_expr.arms {
                    self.analyze_expr_for_async(&arm.body, calls_awaitable, callees);
                }
            }
            _ => {}
        }
    }

    /// Validate conversion compatibility
    fn validate_conversion(&self, info: &AsyncConversionInfo) -> Result<(), String> {
        if info.is_already_async {
            return Err(format!(
                "Function '{}' is already async",
                info.function_name
            ));
        }

        if info.calls_awaitable {
            Ok(())
        } else {
            Err(format!(
                "Function '{}' doesn't appear to use awaitable operations, conversion may not be beneficial",
                info.function_name
            ))
        }
    }
}

#[async_trait]
impl RefactoringOperation for ConvertToAsyncOperation {
    async fn execute(
        &self,
        context: &RefactoringContext,
        options: &RefactoringOptions,
    ) -> Result<RefactoringResult, Box<dyn std::error::Error + Send + Sync>> {
        println!("Advanced convert to async operation executing");

        let file_path = &context.file_path;

        // Read and parse file
        let content = fs::read_to_string(file_path)?;
        let mut syntax: syn::File = syn::parse_str::<syn::File>(syn::parse_str::<syn::File>(&content)??content)?;

        // Get target function name
        let function_name = context
            .symbol_name
            .as_deref()
            .ok_or_else(|| format!("No function name provided for async conversion"))?;

        // Validate file type
        if !is_ast_supported(file_path) {
            return Err(format!(
                "Async conversion only supports Rust (.rs) files, got: {}",
                file_path
            )
            .into());
        }

        // Analyze function for async conversion
        let conversion_info = self.analyze_function_for_async(&syntax, function_name)?;

        // Validate conversion
        self.validate_conversion(&conversion_info)?;

        // Visitor to modify function async-ness
        struct AsyncFunctionVisitor {
            target_name: String,
            should_make_async: bool,
        }

        impl VisitMut for AsyncFunctionVisitor {
            fn visit_item_fn_mut(&mut self, i: &mut syn::ItemFn) {
                if i.sig.ident == self.target_name {
                    println!("Converting function '{}' to async", self.target_name);
                    i.sig.asyncness = Some(syn::token::Async::default());

                    // Update return type if needed
                    if self.should_make_async {
                        match &i.sig.output {
                            syn::ReturnType::Default => {
                                i.sig.output = syn::ReturnType::Type(
                                    syn::token::RArrow::default(),
                                    Box::new(syn::parse_quote!(
                                        impl std::future::Future<Output = ()>
                                    )),
                                );
                            }
                            syn::ReturnType::Type(_, ty) => {
                                if !matches!(**ty, syn::Type::ImplTrait(_)) {
                                    i.sig.output = syn::ReturnType::Type(
                                        syn::token::RArrow::default(),
                                        Box::new(
                                            syn::parse_quote!(impl std::future::Future<Output = #ty>),
                                        ),
                                    );
                                }
                            }
                        }
                    }
                }
                syn::visit_mut::visit_item_fn_mut(self, i);
            }
        }

        // Apply AST transformation
        let mut visitor = AsyncFunctionVisitor {
            target_name: function_name.to_string(),
            should_make_async: true,
        };
        visitor.visit_file_mut(&mut syntax);

        // Generate modified content
        let modified_content = prettyplease::unparse(&syntax);

        // Find callers that need .await
        let caller_updates = self.find_callers_needing_await(&syntax, function_name);

        Ok(RefactoringResult {
            id: Some(crate::utils::RefactoringUtils::generate_refactoring_id()),
            success: true,
            changes: vec![CodeChange {
                file_path: file_path.clone(),
                range: CodeRange {
                    start_line: 1,
                    start_character: 0,
                    end_line: content.lines().count(),
                    end_character: 0,
                },
                old_text: content,
                new_text: modified_content.clone(),
                change_type: ChangeType::Replacement,
            }],
            error_message: None,
            warnings: vec![
                format!(
                    "Function '{}' converted to async - update all callers to use .await",
                    function_name
                ),
                "May need to adjust error handling for async context".to_string(),
                "Consider using tokio::spawn for long-running async operations".to_string(),
            ]
            .into_iter()
            .chain(
                caller_updates
                    .into_iter()
                    .map(|caller| format!("Update caller in {} to use .await", caller)),
            )
            .collect(),
            new_content: Some(modified_content),
        })
    }

    async fn analyze(
        &self,
        context: &RefactoringContext,
    ) -> Result<RefactoringAnalysis, Box<dyn std::error::Error + Send + Sync>> {
        // Parse file to analyze function
        let content = fs::read_to_string(&context.file_path)?;
        let syntax: syn::File = syn::parse_str::<syn::File>(syn::parse_str::<syn::File>(&content)??content)?;

        let function_name = context.symbol_name.clone().unwrap_or_default();
        let analysis = match self.analyze_function_for_async(&syntax, &function_name) {
            Ok(info) => {
                let mut confidence = 0.8;

                if info.is_already_async {
                    confidence = 0.0; // Can't convert if already async
                } else if info.calls_awaitable {
                    confidence = 0.9; // High confidence if already calling awaitable operations
                }

                RefactoringAnalysis {
                    is_safe: !info.is_already_async,
                    confidence_score: confidence,
                    potential_impact: RefactoringImpact::High,
                    affected_files: vec![context.file_path.clone()],
                    affected_symbols: vec![function_name.clone()],
                    breaking_changes: if info.calls_awaitable {
                        vec![
                            format!("All callers of \"{}\" must use .await", function_name),
                            "Return type now wrapped in Future<Output = _>.to_string()".to_string(),
                        ]
                    } else {
                        vec![format!(
                            "Callers may need to use .await for \"{}\"",
                            function_name
                        )]
                    },
                    suggestions: vec![
                        "Ensure all async chains properly handle errors ".to_string(),
                        "Consider using tokio::spawn for CPU-intensive operations ".to_string(),
                        "Review await points for potential deadlocks ".to_string(),
                    ],
                    warnings: if info.calls_awaitable {
                        vec!["High impact - all callers must be updated ".to_string()]
                    } else {
                        vec!["Lower risk - can be gradually migrated ".to_string()]
                    },
                }
            }
            Err(_) => RefactoringAnalysis {
                is_safe: false,
                confidence_score: 0.0,
                potential_impact: RefactoringImpact::High,
                affected_files: vec![context.file_path.clone()],
                affected_symbols: vec![function_name],
                breaking_changes: vec![
                    "Unable to analyze function for async conversion ".to_string()
                ],
                suggestions: vec!["Verify function exists and is convertible ".to_string()],
                warnings: vec!["Analysis failed - conversion may be unsafe ".to_string()],
            },
        };

        Ok(analysis)
    }

    async fn is_applicable(
        &self,
        context: &RefactoringContext,
        options: Option<&RefactoringOptions>,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        Ok(context.symbol_kind == Some(SymbolKind::Function) && context.symbol_name.is_some())
    }

    fn refactoring_type(&self) -> RefactoringType {
        RefactoringType::ConvertToAsync
    }

    fn name(&self) -> &str {
        "Convert to Async "
    }

    fn description(&self) -> &str {
        "Converts a synchronous function to async with proper Future return types "
    }
}

impl ConvertToAsyncOperation {
    /// Find functions that call the target function and need .await
    fn find_callers_needing_await(&self, syntax: &syn::File, target_function: &str) -> Vec<String> {
        let mut callers = Vec::new();

        for item in &syntax.items {
            match item {
                syn::Item::Fn(fn_item) => {
                    if self.function_calls_target(&fn_item.block, target_function) {
                        let fn_name = format!("{}", fn_item.sig.ident);
                        callers.push(format!("function \"{}\"", fn_name));
                    }
                }
                syn::Item::Impl(impl_block) => {
                    for impl_item in &impl_block.items {
                        if let syn::ImplItem::Fn(method) = impl_item {
                            if self.function_calls_target(&method.block, target_function) {
                                let method_name = format!("{}", method.sig.ident);
                                callers.push(format!("method \"{}\"", method_name));
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        callers
    }

    /// Check if a block contains calls to the target function
    fn function_calls_target(&self, block: &syn::Block, target: &str) -> bool {
        for stmt in &block.stmts {
            match stmt {
                syn::Stmt::Expr(expr, _) => {
                    if self.expr_calls_target(expr, target) {
                        return true;
                    }
                }
                syn::Stmt::Local(local) => {
                    if let Some(init) = &local.init {
                        if self.expr_calls_target(&init.expr, target) {
                            return true;
                        }
                    }
                }
                _ => {}
            }
        }
        false
    }

    /// Check if an expression calls the target function
    fn expr_calls_target(&self, expr: &syn::Expr, target: &str) -> bool {
        match expr {
            syn::Expr::Call(call) => {
                if let syn::Expr::Path(path) = &*call.func {
                    if let Some(ident) = path.path.get_ident() {
                        return ident == target;
                    }
                }
            }
            syn::Expr::MethodCall(_) => {
                // Method calls would be more complex to track
                // For now, we'll skip detailed method call analysis
            }
            syn::Expr::Block(block) => {
                if self.function_calls_target(&block.block, target) {
                    return true;
                }
            }
            syn::Expr::If(if_expr) => {
                if self.expr_calls_target(&if_expr.cond, target) {
                    return true;
                }
                if self.function_calls_target(&if_expr.then_branch, target) {
                    return true;
                }
                for branch in &if_expr.else_branch {
                    if let syn::Expr::Block(else_block) = branch.1.as_ref() {
                        if self.function_calls_target(&else_block.block, target) {
                            return true;
                        }
                    }
                }
            }
            _ => {}
        }
        false
    }
}
