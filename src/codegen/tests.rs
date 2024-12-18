use super::*;
use crate::{
    codegen::generator::CodeGenerator,
    Parser::ast::*,
    Semantic::ts::{Symbol, Types},
    SymbolTable,
};
use cranelift::prelude::*;
use cranelift_codegen::ir::Function;

struct FunctionBuilderWrapper {
    func: Function,
    builder_context: FunctionBuilderContext,
}

impl FunctionBuilderWrapper {
    fn new() -> Self {
        Self {
            func: Function::new(),
            builder_context: FunctionBuilderContext::new(),
        }
    }

    fn create_builder(&mut self) -> FunctionBuilder {
        let mut builder = FunctionBuilder::new(&mut self.func, &mut self.builder_context);
        let entry_block = builder.create_block();
        builder.append_block_params_for_function_params(entry_block);
        builder.switch_to_block(entry_block);
        builder.seal_block(entry_block);
        builder
    }
}

fn setup_symbol_table() {
    let mut table = SymbolTable.lock().unwrap();
    table.clear();
}

fn setup_function_builder() -> (CodeGenerator, FunctionBuilderWrapper) {
    setup_symbol_table();
    let codegen = CodeGenerator::new();
    let builder_wrapper = FunctionBuilderWrapper::new();
    (codegen, builder_wrapper)
}

#[cfg(test)]
mod codegen_tests {
    use super::*;

    #[test]
    fn test_binary_operations() {
        let (mut codegen, mut builder_wrapper) = setup_function_builder();
        let mut builder = builder_wrapper.create_builder();

        {
            let mut table = SymbolTable.lock().unwrap();
            table.insert("x".to_string(), Symbol {
                Identifier: "x".to_string(),
                Type: Some(Types::Integer),
                Is_Constant: None,
                Address: None,
                Value: None,
                size: None,
            });
        }

        let operations = vec![
            (BinOp::Add, 5, 3, 8),
            (BinOp::Sub, 10, 4, 6),
            (BinOp::Mul, 6, 7, 42),
            (BinOp::Div, 15, 3, 5),
        ];

        for (op, left, right, _expected) in operations {
            let expr = Expr::BinaryOp(
                Box::new(Expr::Literal(TypeValue::Integer(left))),
                op,
                Box::new(Expr::Literal(TypeValue::Integer(right)))
            );

            let result = codegen.compile_expr(&mut builder, &expr);
            assert!(builder.func.dfg.value_type(result) == types::I32);
        }
    }

    #[test]
    fn test_variable_assignments() {
        let (mut codegen, mut builder_wrapper) = setup_function_builder();
        let mut builder = builder_wrapper.create_builder();

        {
            let mut table = SymbolTable.lock().unwrap();
            table.insert("x".to_string(), Symbol {
                Identifier: "x".to_string(),
                Type: Some(Types::Integer),
                Is_Constant: None,
                Address: None,
                Value: None,
                size: None,
            });
            table.insert("y".to_string(), Symbol {
                Identifier: "y".to_string(),
                Type: Some(Types::Integer),
                Is_Constant: None,
                Address: None,
                Value: None,
                size: None,
            });
        }

        let simple_assign = Assignment {
            var: "x".to_string(),
            index: None,
            expr: Expr::Literal(TypeValue::Integer(42))
        };

        let complex_assign = Assignment {
            var: "y".to_string(),
            index: None,
            expr: Expr::BinaryOp(
                Box::new(Expr::Literal(TypeValue::Integer(10))),
                BinOp::Add,
                Box::new(Expr::Literal(TypeValue::Integer(5)))
            )
        };

        assert!(codegen.compile_assignment(&mut builder, &simple_assign).is_ok());
        assert!(codegen.compile_assignment(&mut builder, &complex_assign).is_ok());
    }

    #[test]
    fn test_conditions_and_if_statements() {
        let (mut codegen, mut builder_wrapper) = setup_function_builder();
        let mut builder = builder_wrapper.create_builder();

        {
            let mut table = SymbolTable.lock().unwrap();
            table.insert("x".to_string(), Symbol {
                Identifier: "x".to_string(),
                Type: Some(Types::Integer),
                Is_Constant: None,
                Address: None,
                Value: None,
                size: None,
            });
        }

        let condition = Condition::Basic(BasicCond {
            left: Expr::Literal(TypeValue::Integer(10)),
            operator: RelOp::Gt,
            right: Expr::Literal(TypeValue::Integer(5))
        });

        let if_stmt = IfStmt {
            condition: condition.clone(),
            then_block: vec![
                Instruction::Assign(Assignment {
                    var: "x".to_string(),
                    index: None,
                    expr: Expr::Literal(TypeValue::Integer(1))
                })
            ],
            else_block: Some(vec![
                Instruction::Assign(Assignment {
                    var: "x".to_string(),
                    index: None,
                    expr: Expr::Literal(TypeValue::Integer(0))
                })
            ])
        };

        codegen.compile_if(&mut builder, &if_stmt);
    }

    #[test]
    fn test_loops() {
        let (mut codegen, mut builder_wrapper) = setup_function_builder();
        let mut builder = builder_wrapper.create_builder();

        {
            let mut table = SymbolTable.lock().unwrap();
            table.insert("i".to_string(), Symbol {
                Identifier: "i".to_string(),
                Type: Some(Types::Integer),
                Is_Constant: None,
                Address: None,
                Value: None,
                size: None,
            });
            table.insert("sum".to_string(), Symbol {
                Identifier: "sum".to_string(),
                Type: Some(Types::Integer),
                Is_Constant: None,
                Address: None,
                Value: None,
                size: None,
            });
        }

        let while_condition = Condition::Basic(BasicCond {
            left: Expr::Variable("i".to_string()),
            operator: RelOp::Lt,
            right: Expr::Literal(TypeValue::Integer(10))
        });

        let while_body = Statement::Assignment(Assignment {
            var: "i".to_string(),
            index: None,
            expr: Expr::BinaryOp(
                Box::new(Expr::Variable("i".to_string())),
                BinOp::Add,
                Box::new(Expr::Literal(TypeValue::Integer(1)))
            )
        });

        let while_loop = Loop::While(Box::new(while_condition), Box::new(while_body));

        let for_stmt = ForStmt {
            init: Assignment {
                var: "i".to_string(),
                index: None,
                expr: Expr::Literal(TypeValue::Integer(0))
            },
            condition: Expr::BinaryOp(
                Box::new(Expr::Variable("i".to_string())),
                BinOp::Sub,
                Box::new(Expr::Literal(TypeValue::Integer(10)))
            ),
            step: Expr::BinaryOp(
                Box::new(Expr::Variable("i".to_string())),
                BinOp::Add,
                Box::new(Expr::Literal(TypeValue::Integer(1)))
            ),
            body: vec![Instruction::Assign(Assignment {
                var: "sum".to_string(),
                index: None,
                expr: Expr::BinaryOp(
                    Box::new(Expr::Variable("sum".to_string())),
                    BinOp::Add,
                    Box::new(Expr::Variable("i".to_string()))
                )
            })]
        };

        codegen.compile_loop(&mut builder, &while_loop);
        codegen.compile_for(&mut builder, &for_stmt);
    }

    #[test]
    fn test_read_write_operations() {
        let (mut codegen, mut builder_wrapper) = setup_function_builder();
        let mut builder = builder_wrapper.create_builder();

        {
            let mut table = SymbolTable.lock().unwrap();
            table.insert("x".to_string(), Symbol {
                Identifier: "x".to_string(),
                Type: Some(Types::Integer),
                Is_Constant: None,
                Address: None,
                Value: None,
                size: None,
            });
        }

        let read_stmt = ReadStmt {
            variable: "x".to_string()
        };

        let write_stmt = WriteStmt {
            elements: vec![
                WriteElement::String("Value of x: ".to_string()),
                WriteElement::Variable("x".to_string())
            ]
        };

        assert!(codegen.compile_read(&mut builder, &read_stmt).is_ok());
        assert!(codegen.compile_write(&mut builder, &write_stmt).is_ok());
    }

    #[test]
    fn test_array_operations() {
        let (mut codegen, mut builder_wrapper) = setup_function_builder();
        let mut builder = builder_wrapper.create_builder();

        {
            let mut table = SymbolTable.lock().unwrap();
            table.insert("arr".to_string(), Symbol {
                Identifier: "arr".to_string(),
                Type: Some(Types::Array(Box::new(Types::Integer), 10)),
                Is_Constant: None,
                Address: None,
                Value: Some(vec![TypeValue::Integer(0); 10]),
                size: Some(10),
            });
        }

        let array_assign = Assignment {
            var: "arr".to_string(),
            index: Some(Expr::Literal(TypeValue::Integer(0))),
            expr: Expr::Literal(TypeValue::Integer(42))
        };

        let array_access = Expr::Array(
            "arr".to_string(),
            Box::new(Expr::Literal(TypeValue::Integer(0)))
        );

        assert!(codegen.compile_assignment(&mut builder, &array_assign).is_ok());
        let result = codegen.compile_expr(&mut builder, &array_access);
        assert!(builder.func.dfg.value_type(result) == types::I32);
    }

    #[test]
    fn test_complex_expressions() {
        let (mut codegen, mut builder_wrapper) = setup_function_builder();
        let mut builder = builder_wrapper.create_builder();

        {
            let mut table = SymbolTable.lock().unwrap();
            table.insert("a".to_string(), Symbol {
                Identifier: "a".to_string(),
                Type: Some(Types::Integer),
                Is_Constant: None,
                Address: None,
                Value: None,
                size: None,
            });
            table.insert("b".to_string(), Symbol {
                Identifier: "b".to_string(),
                Type: Some(Types::Integer),
                Is_Constant: None,
                Address: None,
                Value: None,
                size: None,
            });
        }

        let complex_expr = Expr::BinaryOp(
            Box::new(Expr::BinaryOp(
                Box::new(Expr::Variable("a".to_string())),
                BinOp::Mul,
                Box::new(Expr::Literal(TypeValue::Integer(2)))
            )),
            BinOp::Add,
            Box::new(Expr::BinaryOp(
                Box::new(Expr::Variable("b".to_string())),
                BinOp::Div,
                Box::new(Expr::Literal(TypeValue::Integer(3)))
            ))
        );

        let result = codegen.compile_expr(&mut builder, &complex_expr);
        assert!(builder.func.dfg.value_type(result) == types::I32);
    }
}