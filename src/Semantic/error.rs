use crate::Semantic::ts::Types as SymbolType;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SemanticError {
    #[error("Undeclared variable: {0}")]
    UndeclaredVariable(String),
    
    #[error("Type mismatch: cannot assign {0:?} to {1:?}")]
    TypeMismatch(SymbolType, SymbolType),
    
    #[error("Constant variable {0} cannot be modified")]
    ConstantModification(String),
    
    #[error("Array size mismatch: expected {0}, got {1}")]
    ArraySizeMismatch(usize, usize),
    
    #[error("Variable '{0}' is already declared")]
    DuplicateVariableDeclaration(String), // This handles redeclaration
    
    #[error("Invalid declaration: {0}")]
    InvalidDeclaration(String), // This handles invalid declarations
    
    #[error("Symbol table error: {0}")]
    SymbolTableError(String), // This handles symbol table errors
    
    #[error("Semantic error: {0}")]
    Generic(String),
}
