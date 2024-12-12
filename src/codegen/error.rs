use std::fmt;

#[derive(Debug)]
pub enum CodegenError {
    UndefinedVariable(String),
    UnsupportedOperation(String),
    InvalidType(String),
    CompilationError(String),
    ModuleError(String),
}

impl std::error::Error for CodegenError {}

impl fmt::Display for CodegenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CodegenError::UndefinedVariable(name) => write!(f, "Undefined variable: {}", name),
            CodegenError::UnsupportedOperation(op) => write!(f, "Unsupported operation: {}", op),
            CodegenError::InvalidType(msg) => write!(f, "Invalid type: {}", msg),
            CodegenError::CompilationError(msg) => write!(f, "Compilation error: {}", msg),
            CodegenError::ModuleError(msg) => write!(f, "Module error: {}", msg),
        }
    }
}

impl From<cranelift_module::ModuleError> for CodegenError {
    fn from(err: cranelift_module::ModuleError) -> Self {
        CodegenError::ModuleError(err.to_string())
    }
}