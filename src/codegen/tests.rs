// tests/codegen_tests.rs

use crate::{
    Parser::ast::*,
    codegen::generator::CodeGenerator,
};

#[test]
fn test_basic_setup() {
    let mut codegen = CodeGenerator::new();
    assert!(codegen.test_setup().is_ok());
}

#[test]
fn test_literal_expressions() {
    let mut codegen = CodeGenerator::new();
    
    // Test integer literal
    let int_expr = Expr::Literal(TypeValue::Integer(42));
    assert!(codegen.compile_test_expr(&int_expr).is_ok());
    
    // Test float literal
    let float_expr = Expr::Literal(TypeValue::Float(3.14));
    assert!(codegen.compile_test_expr(&float_expr).is_ok());
    
    // Test char literal
    let char_expr = Expr::Literal(TypeValue::Char('A'));
    assert!(codegen.compile_test_expr(&char_expr).is_ok());
}