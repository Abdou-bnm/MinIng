use crate::Parser::ast::{
    Expr, TypeValue, BinOp, 
    Condition, BasicCond, RelOp, 
    Assignment, Variable, Type
};
use crate::codegen::generator::CodeGenerator;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_codegen_setup() {
        println!("Running test: test_codegen_setup");
        let mut codegen = CodeGenerator::new();
        assert!(codegen.test_setup().is_ok(), "Code generator setup failed");
    }

    // Literal Expression Tests
    #[test]
    fn test_integer_literal() {
        println!("Running test: test_integer_literal");
        let mut codegen = CodeGenerator::new();

        let expr = Expr::Literal(TypeValue::Integer(42));
        assert!(codegen.compile_test_expr(&expr).is_ok(), "Integer literal compilation failed");
    }

    #[test]
    fn test_float_literal() {
        println!("Running test: test_float_literal");
        let mut codegen = CodeGenerator::new();

        let expr = Expr::Literal(TypeValue::Float(3.14));
        assert!(codegen.compile_test_expr(&expr).is_ok(), "Float literal compilation failed");
    }

    #[test]
    fn test_char_literal() {
        println!("Running test: test_char_literal");
        let mut codegen = CodeGenerator::new();

        let expr = Expr::Literal(TypeValue::Char('A'));
        assert!(codegen.compile_test_expr(&expr).is_ok(), "Char literal compilation failed");
    }

    // Binary Operation Tests
    #[test]
    fn test_simple_binary_operations() {
        println!("Running test: test_simple_binary_operations");
        let mut codegen = CodeGenerator::new();

        // Addition
        let add_expr = Expr::BinaryOp(
            Box::new(Expr::Literal(TypeValue::Integer(5))),
            BinOp::Add,
            Box::new(Expr::Literal(TypeValue::Integer(3))),
        );
        assert!(codegen.compile_test_expr(&add_expr).is_ok(), "Addition compilation failed");

        // Subtraction
        let sub_expr = Expr::BinaryOp(
            Box::new(Expr::Literal(TypeValue::Integer(10))),
            BinOp::Sub,
            Box::new(Expr::Literal(TypeValue::Integer(4))),
        );
        assert!(codegen.compile_test_expr(&sub_expr).is_ok(), "Subtraction compilation failed");

        // Multiplication
        let mul_expr = Expr::BinaryOp(
            Box::new(Expr::Literal(TypeValue::Integer(6))),
            BinOp::Mul,
            Box::new(Expr::Literal(TypeValue::Integer(7))),
        );
        assert!(codegen.compile_test_expr(&mul_expr).is_ok(), "Multiplication compilation failed");

        // Division
        let div_expr = Expr::BinaryOp(
            Box::new(Expr::Literal(TypeValue::Integer(15))),
            BinOp::Div,
            Box::new(Expr::Literal(TypeValue::Integer(3))),
        );
        assert!(codegen.compile_test_expr(&div_expr).is_ok(), "Division compilation failed");
    }

    // Nested Expression Test
    #[test]
    fn test_nested_binary_operation() {
        println!("Running test: test_nested_binary_operation");
        let mut codegen = CodeGenerator::new();

        // Expression: (5 + 3) * (10 - 2)
        let nested_expr = Expr::BinaryOp(
            Box::new(Expr::BinaryOp(
                Box::new(Expr::Literal(TypeValue::Integer(5))),
                BinOp::Add,
                Box::new(Expr::Literal(TypeValue::Integer(3))),
            )),
            BinOp::Mul,
            Box::new(Expr::BinaryOp(
                Box::new(Expr::Literal(TypeValue::Integer(10))),
                BinOp::Sub,
                Box::new(Expr::Literal(TypeValue::Integer(2))),
            ))
        );

        assert!(codegen.compile_test_expr(&nested_expr).is_ok(), "Nested expression compilation failed");
    }

    // Condition Tests
    #[test]
    fn test_basic_conditions() {
        println!("Running test: test_basic_conditions");
        let codegen = CodeGenerator::new();

        // Test greater than
        let gt_condition = Condition::Basic(BasicCond::new(
            Expr::Literal(TypeValue::Integer(10)),
            RelOp::Gt,
            Expr::Literal(TypeValue::Integer(5))
        ));

        // assert!(codegen.compile_condition(&gt_condition).is_ok(), "Greater than condition compilation failed");
    }

    // Variable and Assignment Tests
    #[test]
    fn test_variable_assignment() {
        println!("Running test: test_variable_assignment");
        let codegen = CodeGenerator::new();

        // Create an assignment
        let assignment = Assignment::new(
            "x".to_string(), 
            None, 
            Expr::Literal(TypeValue::Integer(42))
        );

        // assert!(codegen.compile_assignment(&assignment).is_ok(), "Variable assignment compilation failed");
    }
}
