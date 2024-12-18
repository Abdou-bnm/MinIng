#[cfg(test)]
mod codegen_tests {
    use crate::{
        codegen::generator::CodeGenerator,
        Parser::ast::*,
        Symbol, Types, SymbolTable,
    };
    use cranelift::prelude::*;
    use cranelift_codegen::ir::Function;

    fn setup_function_builder() -> (CodeGenerator, FunctionBuilder<'static>) {
        let codegen = CodeGenerator::new();
        let func = Function::new();
        let builder_context = FunctionBuilderContext::new();
        
        // Move these into heap allocation to extend their lifetime
        let func_box = Box::new(func);
        let context_box = Box::new(builder_context);
        
        let mut builder = FunctionBuilder::new(
            Box::leak(func_box), 
            Box::leak(context_box)
        );
        
        let entry_block = builder.create_block();
        builder.append_block_params_for_function_params(entry_block);
        builder.switch_to_block(entry_block);
        builder.seal_block(entry_block);
        
        (codegen, builder)
    }

    #[test]
    fn test_binary_operations() {
        let (mut codegen, mut builder) = setup_function_builder();
        
        // Test all binary operations
        let operations = vec![
            (BinOp::Add, 5, 3, 8),
            (BinOp::Sub, 10, 4, 6),
            (BinOp::Mul, 6, 7, 42),
            (BinOp::Div, 15, 3, 5),
        ];

        for (op, left, right, expected) in operations {
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
        let (mut codegen, mut builder) = setup_function_builder();
        
        // Simple assignment
        let simple_assign = Assignment {
            var: "x".to_string(),
            index: None,
            expr: Expr::Literal(TypeValue::Integer(42))
        };

        // Assignment with binary operation
        let complex_assign = Assignment {
            var: "y".to_string(),
            index: None,
            expr: Expr::BinaryOp(
                Box::new(Expr::Literal(TypeValue::Integer(10))),
                BinOp::Add,
                Box::new(Expr::Literal(TypeValue::Integer(5)))
            )
        };

        // Setup symbol table
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

        assert!(codegen.compile_assignment(&mut builder, &simple_assign).is_ok());
        assert!(codegen.compile_assignment(&mut builder, &complex_assign).is_ok());
    }

    #[test]
    #[test]
fn test_conditions_and_if_statements() {
    let (mut codegen, mut builder) = setup_function_builder();

    // Test basic condition
    let condition = Condition::Basic(*Box::new(BasicCond {
        left: Expr::Literal(TypeValue::Integer(10)),
        operator: RelOp::Gt,
        right: Expr::Literal(TypeValue::Integer(5))
    }));

    // Test if statement
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

    // Setup symbol table
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

    codegen.compile_if(&mut builder, &if_stmt);
}


    #[test]
    fn test_loops() {
        let (mut codegen, mut builder) = setup_function_builder();

        // Test while loop
        let while_condition = Condition::Basic(*Box::new(BasicCond {
            left: Expr::Variable("i".to_string()),
            operator: RelOp::Lt,
            right: Expr::Literal(TypeValue::Integer(10))
        }));

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

        // Test for loop
        let for_stmt = ForStmt {
            init: Assignment {
                var: "i".to_string(),
                index: None,
                expr: Expr::Literal(TypeValue::Integer(0))
            },
            condition: Expr::BinaryOp(
                Box::new(Expr::Variable("i".to_string())),
                BinOp::Sub,  // Use BinOp instead of RelOp
                Box::new(Expr::Literal(TypeValue::Integer(10)))
            ),
            step: Expr::BinaryOp(
                Box::new(Expr::Variable("i".to_string())),
                BinOp::Add,
                Box::new(Expr::Literal(TypeValue::Integer(1)))
            ),
            body: vec![
                Instruction::Assign(Assignment {
                    var: "sum".to_string(),
                    index: None,
                    expr: Expr::BinaryOp(
                        Box::new(Expr::Variable("sum".to_string())),
                        BinOp::Add,
                        Box::new(Expr::Variable("i".to_string()))
                    )
                })
            ]
        };

        // Setup symbol table
        {
            let mut table = SymbolTable.lock().unwrap();
            table.insert("i".to_string(), Symbol {
                Identifier: "i".to_string(),
                Type: Some(Types::Integer),
                Is_Constant: None,
                Address: None,
                Value: Some(vec![TypeValue::Integer(0)]),
                size: None,
            });
            table.insert("sum".to_string(), Symbol {
                Identifier: "sum".to_string(),
                Type: Some(Types::Integer),
                Is_Constant: None,
                Address: None,
                Value: Some(vec![TypeValue::Integer(0)]),
                size: None,
            });
        }

        codegen.compile_loop(&mut builder, &while_loop);
        codegen.compile_for(&mut builder, &for_stmt);
    }

    #[test]
    fn test_read_write_operations() {
        let (mut codegen, mut builder) = setup_function_builder();

        // Test read operation
        let read_stmt = ReadStmt {
            variable: "x".to_string()
        };

        // Test write operation
        let write_stmt = WriteStmt {
            elements: vec![
                WriteElement::String("Value of x: ".to_string()),
                WriteElement::Variable("x".to_string())
            ]
        };

        // Setup symbol table
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

        assert!(codegen.compile_read(&mut builder, &read_stmt).is_ok());
        assert!(codegen.compile_write(&mut builder, &write_stmt).is_ok());
    }

    #[test]
    fn test_array_operations() {
        let (mut codegen, mut builder) = setup_function_builder();

        // Array assignment
        let array_assign = Assignment {
            var: "arr".to_string(),
            index: Some(*Box::new(Expr::Literal(TypeValue::Integer(0)))),
            expr: Expr::Literal(TypeValue::Integer(42))
        };

        // Array access
        let array_access = Expr::Array(
            "arr".to_string(),
            Box::new(Expr::Literal(TypeValue::Integer(0)))
        );

        // Setup symbol table with array
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

        assert!(codegen.compile_assignment(&mut builder, &array_assign).is_ok());
        let result = codegen.compile_expr(&mut builder, &array_access);
        assert!(builder.func.dfg.value_type(result) == types::I32);
    }

    #[test]
    fn test_complex_expressions() {
        let (mut codegen, mut builder) = setup_function_builder();

        // Complex arithmetic expression
        let complex_expr = Expr::BinaryOp(
            Box::new(Expr::BinaryOp(
                Box::new(Expr::Literal(TypeValue::Integer(5))),
                BinOp::Mul,
                Box::new(Expr::BinaryOp(
                    Box::new(Expr::Literal(TypeValue::Integer(3))),
                    BinOp::Add,
                    Box::new(Expr::Literal(TypeValue::Integer(2)))
                ))
            )),
            BinOp::Sub,
            Box::new(Expr::Literal(TypeValue::Integer(10)))
        );

        // Complex condition
        let complex_condition = Condition::Logic(
            Box::new(Condition::Basic(*Box::new(BasicCond {
                left: Expr::Variable("x".to_string()),
                operator: RelOp::Gt,
                right: Expr::Literal(TypeValue::Integer(0))
            }))),
            LogOp::And,
            Box::new(Condition::Basic(*Box::new(BasicCond {
                left: Expr::Variable("y".to_string()),
                operator: RelOp::Lt,
                right: Expr::Literal(TypeValue::Integer(10))
            })))
        );

        // Setup symbol table
        {
            let mut table = SymbolTable.lock().unwrap();
            table.insert("x".to_string(), Symbol {
                Identifier: "x".to_string(),
                Type: Some(Types::Integer),
                Is_Constant: None,
                Address: None,
                Value: Some(vec![TypeValue::Integer(5)]),
                size: None,
            });
            table.insert("y".to_string(), Symbol {
                Identifier: "y".to_string(),
                Type: Some(Types::Integer),
                Is_Constant: None,
                Address: None,
                Value: Some(vec![TypeValue::Integer(3)]),
                size: None,
            });
        }

        let expr_result = codegen.compile_expr(&mut builder, &complex_expr);
        assert!(builder.func.dfg.value_type(expr_result) == types::I32);

        let condition_result = codegen.compile_condition(&mut builder, &complex_condition);
        assert!(builder.func.dfg.value_type(condition_result) == types::I32);
    }
}