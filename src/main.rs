#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(nonstandard_style)]
mod codegen;
mod Lexer;
mod Parser;
mod Semantic;
mod Test;

use std::collections::HashMap;
use std::process::exit;
use std::sync::Mutex;
// Import LALRPOP utilities
use lalrpop_util;
use lalrpop_util::lalrpop_mod;
use logos::Logos;
use once_cell::sync::Lazy;
// use crate::Lexer::lexer::SymbolTable;
use crate::Parser::ast::BinOp;
use crate::Semantic::semantic_analyzer::SemanticAnalyzer;
use crate::Semantic::ts;
use crate::Semantic::ts::TypeValue;

lalrpop_mod!(pub grammar, "/Parser/grammar.rs");
pub static SymbolTable: Lazy<Mutex<HashMap<String, ts::Symbol>>> = Lazy::new(|| Mutex::new(HashMap::new()));
// pub static symbol: Lazy<Mutex<ts::SymbolTable>> = Lazy::new(|| Mutex::new(ts::SymbolTable::new()));


fn main() {
    let input = r#"
    VAR_GLOBAL {
        INTEGER V, X, W;
        FLOAT Y;
        CHAR A;
        INTEGER B = 4;
        CHAR Arr0[B * 2] = "";
        CHAR Arr3[6] = "Hello";
        FLOAT Arr1[B] = [1.2, .5];
        CHAR Arr2[10] = ['S', 't', 'r', 'i', 'n', 'g'];
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
    
    let nn = SymbolTable.lock().unwrap();
    
// **************************************************** Lexical Analysis ****************************************************
// Display of all tokens, enumerated
//     let mut i = 0;
//     let mut lexer = Lexer::lexer::Token::lexer(input);
//     while let Some(token) = lexer.next() {
//         println!("{}: {:?}", i, token);
//         i += 1;
//     }

// Prints errors found in the lexical analysis phase
    let mut lexer = Lexer::lexer::Token::lexer(input);
    while let Some(token) = lexer.next() {
        match token {
            Err(e) => {
                eprintln!("Lexical Error: {}", e);
                exit(1);
            },
            Ok(token) => {}
        }
    }
    println!("Lexical Analysis Successful.");

// **************************************************** Syntactic Analysis ****************************************************
    let lexer = Lexer::lexer::Token::lexer(input);
    let parser = grammar::ProgramParser::new();
    let result = parser.parse(input, lexer.enumerate().map(|(i, t)| t.map(|token| (i, token, i+1)).map_err(|e| e)));

    let program;
    
    match result {
        Ok(t) => {
            println!("Syntactic Analysis Successful.");
            program = t;
        },
        Err(e) => {
            eprintln!("Syntactic Error: {:?}", e);
            exit(1);
        },
    }
    
    
// Printing Program's Structure
//     println!("Program Structure:");
// 
//     // Print Global Variables
//     if let Some(globals) = &program.global {
//         println!("\nGlobal Variables:");
//         for decl in globals {
//             println!("{:?}", decl);
//         }
//     }
// 
//     // Print Declarations
//     if let Some(decls) = &program.decls {
//         println!("\nDeclarations:");
//         for decl in decls {
//             println!("{:?}", decl);
//         }
//     }
// 
//     // Print Instructions
//     if let Some(instructions) = &program.inst {
//         println!("\nInstructions:");
//         for inst in instructions {
//             println!("{:?}", inst);
//         }
//     }

// **************************************************** Semantic Analysis ****************************************************
//     constant re-assignment: done,
//     Wrong Type re-assignment: done,
//     READ & WRITE variable verification: done
//     Expression Parsing and calculating results: Done (for Ints, tested it inside array size)
//     Array size check: Done
//     If conditions: Not yet, PC's battery will die
    let mut semanticAnalyzer = SemanticAnalyzer::new();
    
    let semantic_result = semanticAnalyzer.analyze(&program);
    match semantic_result {
        Ok(semantic) => println!("Semantic Analysis Successful."),
        Err(msg) => {
            eprintln!("Semantic Error: {}", msg);
            exit(1);
        },
    }

// **************************************************** Symbol Table ****************************************************
// Full print of the symbol table
    println!("\nSymbol Table:");
    let ST = SymbolTable.lock().unwrap();
    for (key, value) in ST.iter() {
        println!("{}:\n{}", key, value);
    }
}