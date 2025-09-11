use std::path::Path;
use syn::{visit_mut::VisitMut, Ident};

/// Check if a file is supported by AST-based operations (Rust files)
pub fn is_ast_supported(file_path: &str) -> bool {
    rust_ai_ide_shared_utils::get_extension(Path::new(file_path))
        .map(|ext| ext == "rs")
        .unwrap_or(false)
}

/// AST visitor to rename identifiers in code
pub struct IdentifierRenamer {
    old_name: String,
    new_name: String,
    rename_count: usize,
}

impl IdentifierRenamer {
    pub fn new(old_name: String, new_name: String) -> Self {
        IdentifierRenamer {
            old_name,
            new_name,
            rename_count: 0,
        }
    }
}

impl VisitMut for IdentifierRenamer {
    fn visit_ident_mut(&mut self, i: &mut Ident) {
        if i.to_string() == self.old_name {
            // Only rename if this isn't a keyword or built-in identifier
            if ![
                "fn", "let", "const", "mut", "if", "else", "while", "for", "loop", "match",
                "return", "break", "continue", "struct", "enum", "trait", "impl", "pub", "use",
                "mod", "type", "where", "as", "crate", "super", "self", "Self", "true", "false",
            ]
            .contains(&self.old_name.as_str())
            {
                *i = Ident::new(&self.new_name, i.span());
                self.rename_count += 1;
            }
        }
        syn::visit_mut::visit_ident_mut(self, i);
    }
}
