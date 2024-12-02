#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(nonstandard_style)]
mod codegen;
mod Lexer;
mod Parser;
mod Semantic;
mod Test;

use std::process::exit;
// Import LALRPOP utilities
use lalrpop_util;
use lalrpop_util::lalrpop_mod;
use logos::Logos;
use crate::Lexer::lexer::SymbolTable;
use crate::Semantic::semantic_analyzer::SemanticAnalyzer;

lalrpop_mod!(pub grammar, "/Parser/grammar.rs");

fn main() {
    let input = r#"
    VAR_GLOBAL {
        INTEGER V, X, W;
        FLOAT Y;
        CHAR A;
        %%CHAR Arr0[10] = "String";
        %%FLOAT Arr1[2] = [1.2, .5];
        %%CHAR Arr2[10] = ['S', 't', 'r', 'i', 'n', 'g'];
        INTEGER I;
    }
    DECLARATION {
        CONST INTEGER D = 5;
        CONST FLOAT R = .6;
    }
    INSTRUCTION {
        %%Arr0[4] = 45 + 2;
        Y = .2 + 1.5;
        A = 'X';
        WRITE(Dea);
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

// Display of all tokens, enumerated
//     let mut i = 0;
//     let mut lexer = Lexer::lexer::Token::lexer(input);
//     while let Some(token) = lexer.next() {
//         println!("{}: {:?}", i, token);
//         i += 1;
//     }

// Prints errors found in the lexical analysis phase
//     let mut lexer = Lexer::lexer::Token::lexer(input);
//     while let Some(token) = lexer.next() {
//         match token {
//             Err(e) => panic!("{:?}", e),
//             Ok(token) => {}
//         }
//     }

// Syntactic analysis result
    let lexer = Lexer::lexer::Token::lexer(input);
    let parser = grammar::ProgramParser::new();
    let result = parser.parse(input, lexer.enumerate().map(|(i, t)| t.map(|token| (i, token, i+1)).map_err(|e| e)));
    // TODO: NEED TO KNOW HOW TO CLONE THE PROGRAM STRUCT TO RE-USE IT MULTIPLE TIMES
    // match result {
    //     Ok(program) => {
    //         println!("Program Structure:");
    //
    //         // Print Global Variables
    //         if let Some(globals) = program.global {
    //             println!("\nGlobal Variables:");
    //             for decl in globals {
    //                 println!("{:?}", decl);
    //             }
    //         }
    //
    //         // Print Declarations
    //         if let Some(decls) = program.decls {
    //             println!("\nDeclarations:");
    //             for decl in decls {
    //                 println!("{:?}", decl);
    //             }
    //         }
    //
    //         // Print Instructions
    //         if let Some(instructions) = program.inst {
    //             println!("\nInstructions:");
    //             for inst in instructions {
    //                 println!("{:?}", inst);
    //             }
    //         }
    //     },
    //     Err(e) => {
    //         println!("Parsing error: {:?}", e);
    //         exit(1);
    //     },
    // }
    let result = result.as_ref().unwrap();

// Semantic Analysis
    let mut semanticAnalyzer = SemanticAnalyzer::new();
    let semantic_result = semanticAnalyzer.analyze(&result);
    match semantic_result {
        Ok(semantic) => println!("Semantic Analysis Successful!"),
        Err(msg) => eprintln!("Semantic Error: {}", msg),
    }

// Full print of the symbol table
//     println!("\nSymbol Table:");
//     let ST = SymbolTable.lock().unwrap();
//     for (key, value) in ST.iter() {
//         println!("{}:\n{}", key, value);
//     }
}