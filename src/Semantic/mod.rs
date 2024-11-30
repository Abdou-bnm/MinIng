pub mod ts;
pub mod type_checker;
pub mod semantic_analyzer;
pub mod semantic_rules;
pub mod error;

// Re-export key types and functions for easy access
pub use ts::SymbolTable;
pub use type_checker::TypeChecker;
// pub use semantic_analyzer::SemanticAnalyzer;
pub use error::SemanticError;