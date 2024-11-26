#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]
#[cfg(test)]
mod tests {

    use crate::Lexer::lexer::{Keyword, Token,SymbolTable};
    use crate::error::CustomError;
    use logos::Logos;
    use crate::Parser::ast::*;
    use crate::Semantic::ts::*;
    use crate::Parser::*;

    #[test]
    fn test_keywords_and_types() {
        let mut lexer = Token::lexer("VAR_GLOBAL DECLARATION INSTRUCTION INTEGER FLOAT CHAR");
        assert_eq!(lexer.next(), Some(Ok(Token::VarGlobal)));
        assert_eq!(lexer.next(), Some(Ok(Token::Declaration)));
        assert_eq!(lexer.next(), Some(Ok(Token::Instruction)));
        assert_eq!(lexer.next(), Some(Ok(Token::IntegerType)));
        assert_eq!(lexer.next(), Some(Ok(Token::FloatType)));
        assert_eq!(lexer.next(), Some(Ok(Token::CharType)));
    }

    #[test]
    fn test_full_program() {
        let mut lexer = Token::lexer("
            %% THIS IS A COMMENT
            VAR_GLOBAL {
                INTEGER V,X, W;
            %% THIS IS A COMMENT
                FLOAT Y;
                CHAR Name[10];
            }
            DECLARATION {
            %% THIS IS A COMMENT
                CONST INTEGER D = 5;
                CONST FLOAT R = .6;
            }
            INSTRUCTION {

                N = 10;
                IF (X > 0) {
                    WRITE(\"X is positive\");
                } ELSE {
                    WRITE(\"x is non-positive\");
                }
                FOR (I = 0:  2 : N) {
                    WRITE(I);
                }
            }
        ");
        println!("Will start printing the tokens...");
        for token in lexer.by_ref() {
            match token {
                Ok(token) => println!("{:#?}", token),
                Err(e) => println!("some error occurred: {:?}", e),
            }
        }

        println!("\n\nWill start printing the Symbol Table...");
        let table = SymbolTable.lock().unwrap();
        for (key, value) in table.iter() {
            println!("{}:\n{}", key, value);
        }
    }
    // #[test]
    // fn test_invalid_identifiers_and_overflows() {
    //     let mut lexer = Token::lexer("VERYLONGID");
    //     assert_eq!(lexer.next(), Some(Err(CustomError::IdentifierTooLong("VERYLONGID".to_string()))));
    //
    //     let mut lexer = Token::lexer("2147483648"); // Overflows i16
    //     assert_eq!(lexer.next(), Some(Err(CustomError::IntegerOverflow("2147483648".to_string()))));
    //     // let mut lexer = Token::lexer("340282350000000000000000000000000000001.0"); // Overflows f32
    //     // assert_eq!(lexer.next(), Some(Err(CustomError::FloatOverflow("340282350000000000000000000000000000001.0".to_string()))));
    // }
    // fn generate_symbol_table(program: &Program) -> std::collections::HashMap<String, String> {
    //     let mut table = std::collections::HashMap::new();
    //
    //     if let Some(global_decls) = &program.global {
    //         for decl in global_decls {
    //             match decl {
    //                 Declaration::Variables(typ, vars) => {
    //                     for var in vars {
    //                         match var {
    //                             Variable::Simple(name) => {
    //                                 table.insert(name.clone(), format!("{:?}", typ));
    //                             }
    //                             Variable::Initialized(name, _) => {
    //                                 table.insert(name.clone(), format!("{:?}", typ));
    //                             }
    //                         }
    //                     }
    //                 }
    //                 Declaration::Array(typ, array_decl) => {
    //                     table.insert(array_decl.name.clone(), format!("Array<{:?}>", typ));
    //                 }
    //                 Declaration::Constant(typ, assignments) => {
    //                     for assign in assignments {
    //                         table.insert(assign.var.clone(), format!("Const<{:?}>", typ));
    //                     }
    //                 }
    //             }
    //         }
    //     }
    //
    //     if let Some(decls) = &program.decls {
    //         for decl in decls {
    //             match decl {
    //                 Declaration::Variables(typ, vars) => {
    //                     for var in vars {
    //                         match var {
    //                             Variable::Simple(name) => {
    //                                 table.insert(name.clone(), format!("{:?}", typ));
    //                             }
    //                             Variable::Initialized(name, _) => {
    //                                 table.insert(name.clone(), format!("{:?}", typ));
    //                             }
    //                         }
    //                     }
    //                 }
    //                 _ => {} // Handle arrays and constants similarly if needed
    //             }
    //         }
    //     }
    //
    //     table
    // }

    // #[test]
    // fn test_program_with_globals() {
    //     let globals = Some(vec![
    //         Declaration::Variables(
    //             Type::Integer,
    //             vec![Variable::Simple("x".to_string()), Variable::Simple("y".to_string())],
    //         ),
    //         Declaration::Constant(
    //             Type::Float,
    //             vec![Assignment::new("pi".to_string(), Expr::Literal(Literal::Float(3.14)))],
    //         ),
    //     ]);
    //
    //     let program = Program::new(globals, None, None);
    //     let table = generate_symbol_table(&program);
    //
    //     assert_eq!(table.get("x").unwrap(), "Integer");
    //     assert_eq!(table.get("y").unwrap(), "Integer");
    //     assert_eq!(table.get("pi").unwrap(), "Const<Float>");
    // }
    //
    // #[test]
    // fn test_program_with_instructions() {
    //     let inst = Some(vec![
    //         Instruction::Assign(Assignment::new(
    //             "z".to_string(),
    //             Expr::BinaryOp(
    //                 Box::new(Expr::Literal(Literal::Integer(5))),
    //                 BinOp::Add,
    //                 Box::new(Expr::Variable("x".to_string())),
    //             ),
    //         )),
    //         Instruction::Write(WriteStmt::new(vec![
    //             WriteElement::Variable("z".to_string()),
    //             WriteElement::String(" done".to_string()),
    //         ])),
    //     ]);
    //
    //     let program = Program::new(None, None, inst);
    //     // No symbol table for instructions; focus on testing AST correctness.
    //     if let Some(instructions) = &program.inst {
    //         match &instructions[0] {
    //             Instruction::Assign(assign) => {
    //                 assert_eq!(assign.var, "z");
    //             }
    //             _ => panic!("Expected an assignment instruction"),
    //         }
    //     }
    // }
    //
    // #[test]
    // fn test_full() {
    //     let globals = Some(vec![
    //         Declaration::Variables(
    //             Type::Integer,
    //             vec![Variable::Simple("x".to_string())],
    //         ),
    //     ]);
    //     let decls = Some(vec![
    //         Declaration::Variables(
    //             Type::Float,
    //             vec![Variable::Simple("y".to_string())],
    //         ),
    //     ]);
    //     let inst = Some(vec![
    //         Instruction::Assign(Assignment::new(
    //             "z".to_string(),
    //             Expr::BinaryOp(
    //                 Box::new(Expr::Variable("x".to_string())),
    //                 BinOp::Mul,
    //                 Box::new(Expr::Variable("y".to_string())),
    //             ),
    //         )),
    //     ]);
    //
    //     let program = Program::new(globals, decls, inst);
    //     let table = generate_symbol_table(&program);
    //
    //     assert_eq!(table.get("x").unwrap(), "Integer");
    //     assert_eq!(table.get("y").unwrap(), "Float");
    //     assert_eq!(table.get("z"), None); // 'z' is not in the table as it's defined in instructions
    // }
    //
}