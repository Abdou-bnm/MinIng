#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(nonstandard_style)]
mod error;
mod codegen;
mod Lexer;
mod Parser;
mod Semantic;
mod Test;

// Import LALRPOP utilities
use lalrpop_util;
use lalrpop_util::lalrpop_mod;
use logos::Logos;

lalrpop_mod!(pub grammar, "/Parser/grammar.rs");

fn main() {
    let input = r#"
    VAR_GLOBAL {
        INTEGER V, X, W;
        FLOAT Y;
        CHAR Names[10];
        INTEGER I;
    }
    DECLARATION {
        CONST INTEGER D = 5;
        CONST FLOAT R = .6;
    }
    INSTRUCTION {
        Names[0] = 10;
        %% N = 10;
        IF (X > 0) {
            WRITE("X is positive");
        } ELSE {
            WRITE("x is non-positive");
        }
        FOR (I = 0 : 2 : X) {
            WRITE(I);
        }
    }
    "#;

    let mut lexer = Lexer::lexer::Token::lexer(input);
    while let Some(token) = lexer.next() {
        match token {
            Err(e) => panic!("{:?}", e),
            Ok(token) => {}
        }
    }

    let lexer = Lexer::lexer::Token::lexer(input);
    let parser = grammar::ProgramParser::new();
    let result = parser.parse(input, lexer.enumerate().map(|(i, t)| t.map(|token| (i, token, i+1)).map_err(|e| e)));

    match result {
        Ok(program) => {
            println!("Program Structure:");

            // Print Global Variables
            if let Some(globals) = program.global {
                println!("\nGlobal Variables:");
                for decl in globals {
                    println!("{:?}", decl);
                }
            }

            // Print Declarations
            if let Some(decls) = program.decls {
                println!("\nDeclarations:");
                for decl in decls {
                    println!("{:?}", decl);
                }
            }

            // Print Instructions
            if let Some(instructions) = program.inst {
                println!("\nInstructions:");
                for inst in instructions {
                    println!("{:?}", inst);
                }
            }
        },
        Err(e) => println!("Parsing error: {:?}", e),
    }
}