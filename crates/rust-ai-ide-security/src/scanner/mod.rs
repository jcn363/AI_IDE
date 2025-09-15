//! OWASP Security Scanner module

mod code_scanner;
mod dependency_scanner;
mod types;

pub use code_scanner::scan_code;
pub use dependency_scanner::scan_dependencies;
pub use types::*;
