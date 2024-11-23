mod ast;
use ast::*;

mod lexer;
use lexer::*;

mod grammar;
use grammar::*;

mod error;
use error::*;

/// Mock function to simulate a symbol table generator.
fn generate_symbol_table(program: &Program) -> std::collections::HashMap<String, String> {
    let mut table = std::collections::HashMap::new();

    if let Some(global_decls) = &program.global {
        for decl in global_decls {
            match decl {
                Declaration::Variables(typ, vars) => {
                    for var in vars {
                        match var {
                            Variable::Simple(name) => {
                                table.insert(name.clone(), format!("{:?}", typ));
                            }
                            Variable::Initialized(name, _) => {
                                table.insert(name.clone(), format!("{:?}", typ));
                            }
                        }
                    }
                }
                Declaration::Array(typ, array_decl) => {
                    table.insert(array_decl.name.clone(), format!("Array<{:?}>", typ));
                }
                Declaration::Constant(typ, assignments) => {
                    for assign in assignments {
                        table.insert(assign.var.clone(), format!("Const<{:?}>", typ));
                    }
                }
            }
        }
    }

    if let Some(decls) = &program.decls {
        for decl in decls {
            match decl {
                Declaration::Variables(typ, vars) => {
                    for var in vars {
                        match var {
                            Variable::Simple(name) => {
                                table.insert(name.clone(), format!("{:?}", typ));
                            }
                            Variable::Initialized(name, _) => {
                                table.insert(name.clone(), format!("{:?}", typ));
                            }
                        }
                    }
                }
                _ => {} // Handle arrays and constants similarly if needed
            }
        }
    }

    table
}

#[test]
fn test_program_with_globals() {
    let globals = Some(vec![
        Declaration::Variables(
            Type::Integer,
            vec![Variable::Simple("x".to_string()), Variable::Simple("y".to_string())],
        ),
        Declaration::Constant(
            Type::Float,
            vec![Assignment::new("pi".to_string(), Expr::Literal(Literal::Float(3.14)))],
        ),
    ]);

    let program = Program::new(globals, None, None);
    let table = generate_symbol_table(&program);

    assert_eq!(table.get("x").unwrap(), "Integer");
    assert_eq!(table.get("y").unwrap(), "Integer");
    assert_eq!(table.get("pi").unwrap(), "Const<Float>");
}

#[test]
fn test_program_with_instructions() {
    let inst = Some(vec![
        Instruction::Assign(Assignment::new(
            "z".to_string(),
            Expr::BinaryOp(
                Box::new(Expr::Literal(Literal::Integer(5))),
                BinOp::Add,
                Box::new(Expr::Variable("x".to_string())),
            ),
        )),
        Instruction::Write(WriteStmt::new(vec![
            WriteElement::Variable("z".to_string()),
            WriteElement::String(" done".to_string()),
        ])),
    ]);

    let program = Program::new(None, None, inst);
    // No symbol table for instructions; focus on testing AST correctness.
    if let Some(instructions) = &program.inst {
        match &instructions[0] {
            Instruction::Assign(assign) => {
                assert_eq!(assign.var, "z");
            }
            _ => panic!("Expected an assignment instruction"),
        }
    }
}

#[test]
fn test_full_program() {
    let globals = Some(vec![
        Declaration::Variables(
            Type::Integer,
            vec![Variable::Simple("x".to_string())],
        ),
    ]);
    let decls = Some(vec![
        Declaration::Variables(
            Type::Float,
            vec![Variable::Simple("y".to_string())],
        ),
    ]);
    let inst = Some(vec![
        Instruction::Assign(Assignment::new(
            "z".to_string(),
            Expr::BinaryOp(
                Box::new(Expr::Variable("x".to_string())),
                BinOp::Mul,
                Box::new(Expr::Variable("y".to_string())),
            ),
        )),
    ]);

    let program = Program::new(globals, decls, inst);
    let table = generate_symbol_table(&program);

    assert_eq!(table.get("x").unwrap(), "Integer");
    assert_eq!(table.get("y").unwrap(), "Float");
    assert_eq!(table.get("z"), None); // 'z' is not in the table as it's defined in instructions
}
