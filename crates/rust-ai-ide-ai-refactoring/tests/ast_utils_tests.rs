#[cfg(test)]
mod tests {
    use rust_ai_ide_ai_refactoring::ast_utils::{is_ast_supported, IdentifierRenamer};
    use syn::parse_str;
    use syn::visit_mut::VisitMut;

    #[test]
    fn test_is_ast_supported_rust_files() {
        assert!(is_ast_supported("main.rs"));
        assert!(is_ast_supported("lib.rs"));
        assert!(is_ast_supported("mod.rs"));
        assert!(is_ast_supported("/path/to/file.rs"));
    }

    #[test]
    fn test_is_ast_supported_non_rust_files() {
        assert!(!is_ast_supported("main.py"));
        assert!(!is_ast_supported("lib.js"));
        assert!(!is_ast_supported("file.txt"));
        assert!(!is_ast_supported(""));
        assert!(!is_ast_supported("no_extension"));
    }

    #[test]
    fn test_is_ast_supported_edge_cases() {
        assert!(!is_ast_supported(".rs"));
        assert!(!is_ast_supported("file.rs.bak"));
        assert!(!is_ast_supported("file.RS")); // case sensitive
    }

    #[test]
    fn test_identifier_renamer_new() {
        let renamer = IdentifierRenamer::new("old_name".to_string(), "new_name".to_string());
        assert_eq!(renamer.old_name, "old_name");
        assert_eq!(renamer.new_name, "new_name");
        assert_eq!(renamer.rename_count, 0);
    }

    #[test]
    fn test_identifier_renamer_visit_ident_successful_rename() {
        let mut renamer = IdentifierRenamer::new("old_var".to_string(), "new_var".to_string());

        // Parse a simple let statement
        let mut expr: syn::Expr = parse_str("old_var").unwrap();
        renamer.visit_expr_mut(&mut expr);

        if let syn::Expr::Path(path_expr) = expr {
            if let Some(ident) = path_expr.path.get_ident() {
                assert_eq!(ident.to_string(), "new_var");
                assert_eq!(renamer.rename_count, 1);
            } else {
                panic!("Expected identifier");
            }
        } else {
            panic!("Expected path expression");
        }
    }

    #[test]
    fn test_identifier_renamer_visit_ident_keyword_not_renamed() {
        let mut renamer = IdentifierRenamer::new("fn".to_string(), "function".to_string());

        // Parse a keyword
        let mut expr: syn::Expr = parse_str("fn").unwrap();
        renamer.visit_expr_mut(&mut expr);

        if let syn::Expr::Path(path_expr) = expr {
            if let Some(ident) = path_expr.path.get_ident() {
                assert_eq!(ident.to_string(), "fn"); // Should not be renamed
                assert_eq!(renamer.rename_count, 0);
            }
        }
    }

    #[test]
    fn test_identifier_renamer_visit_ident_multiple_occurrences() {
        let mut renamer = IdentifierRenamer::new("x".to_string(), "y".to_string());

        // Parse code with multiple occurrences
        let code = "fn test() { let x = 1; x + x }";
        let mut file: syn::File = parse_str(code).unwrap();
        renamer.visit_file_mut(&mut file);

        assert_eq!(renamer.rename_count, 3); // x in let, x in x+x, x in second x+x
    }

    #[test]
    fn test_identifier_renamer_visit_ident_no_matches() {
        let mut renamer = IdentifierRenamer::new("nonexistent".to_string(), "new".to_string());

        let code = "fn test() { let x = 1; }";
        let mut file: syn::File = parse_str(code).unwrap();
        renamer.visit_file_mut(&mut file);

        assert_eq!(renamer.rename_count, 0);
    }

    #[test]
    fn test_identifier_renamer_visit_ident_mixed_keywords_and_identifiers() {
        let mut renamer = IdentifierRenamer::new("let".to_string(), "declare".to_string());

        let code = "let x = 1;"; // let is a keyword, should not be renamed
        let mut stmt: syn::Stmt = parse_str(code).unwrap();
        renamer.visit_stmt_mut(&mut stmt);

        assert_eq!(renamer.rename_count, 0);
    }

    #[test]
    fn test_identifier_renamer_visit_ident_complex_code() {
        let mut renamer = IdentifierRenamer::new("count".to_string(), "total".to_string());

        let code = r#"
            fn increment(count: i32) -> i32 {
                let local_count = count + 1;
                local_count
            }
        "#;
        let mut file: syn::File = parse_str(code).unwrap();
        renamer.visit_file_mut(&mut file);

        assert_eq!(renamer.rename_count, 3); // count parameter, count in body, local_count
    }
}
