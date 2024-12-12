use crate::Parser::ast::{Expr, TypeValue, BinOp};
use crate::codegen::generator::CodeGenerator;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_codegen_setup() {
        let mut codegen = CodeGenerator::new();
        assert!(codegen.test_setup().is_ok(), "Code generator setup failed");
    }

    #[test]
    fn test_simple_expression() {
        let mut codegen = CodeGenerator::new();
        
        // Test expression: 5 + 3
        let expr = Expr::BinaryOp(
            Box::new(Expr::Literal(TypeValue::Integer(5))),
            BinOp::Add,
            Box::new(Expr::Literal(TypeValue::Integer(3))),
        );
        
        assert!(codegen.compile_test_expr(&expr).is_ok(), "Expression compilation failed");
    }
}