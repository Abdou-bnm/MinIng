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
use crate::Parser::ast::BinOp;
use crate::Semantic::quadruplets::QuadrupletGenerator;
use crate::Semantic::semantic_analyzer::SemanticAnalyzer;
use crate::Semantic::ts::*;

lalrpop_mod!(pub grammar, "/Parser/grammar.rs");
pub static SymbolTable: Lazy<Mutex<HashMap<String, Symbol>>> = Lazy::new(|| Mutex::new(HashMap::new()));

fn main() {
    let input = r#"
    VAR_GLOBAL {
        INTEGER V;
        INTEGER X = 1;
        FLOAT Z = 2.0;
        CHAR Y[1] = "";
        CHAR E = '!';
        INTEGER Arr0[7] = [1, 2, 3, 4];
        INTEGER B = 4;
        INTEGER C = B + 4,
                A = B + C + 2;
        CHAR F = ' ';
        CHAR Arr5[3] = "";
        CHAR Arr3[6] = "Hello";
        FLOAT Arr4[5];
        FLOAT Arr1[B] = [1.2, .5124, 2.0];
        CHAR Arr2[10] = ['S', 't', 'r', 'i', 'n', 'g'];
        CHAR I = 'X';
    }
    DECLARATION {
        CONST INTEGER D = 5;
        CONST FLOAT R = .6;
    }
    INSTRUCTION {
        READ(V);
        X = V + 4;
        Z = ( - ( 6.5 * 4.5 + 5.6) );
        X = ( - ( 5 * 9 + 6 ));
        Arr1[2] = ( - 5.6 );
        X = (+1);
        X = (-B);
        WRITE(B);
        READ(Arr4[1]);
        B = B + 4;
        Arr2[1] = '1';
        Arr2[3] = '1';
        Arr1[1] = (Arr1[1] + Arr1[1]) / Arr1[2];
        Arr3[2] = 'L';
        %% This is a comment

        %% READ(B);
        WRITE("Enter a posivite number");
        WRITE("B read value : ", B , "." );
        IF( B >= 0) { B = B + 1; } ELSE {B = 0;}
        %% Z = Arr4[0];
        Arr1[3] = Arr4[1];
        WRITE(Arr4[1]);
        WRITE(Arr4[2]);
        FOR( B = 2 : 6 : 10) { B = B + 1; }
    }
    "#;
    
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

    println!("Program Structure:");

    // Print Global Variables
    if let Some(globals) = &program.global {
        println!("\nGlobal Variables:");
        for decl in globals {
            println!("{:?}", decl);
        }
    }

    // Print Declarations
    if let Some(decls) = &program.decls {
        println!("\nDeclarations:");
        for decl in decls {
            println!("{:?}", decl);
        }
    }

    // Print Instructions
    if let Some(instructions) = &program.inst {
        println!("\nInstructions:");
        for inst in instructions {
            println!("{:?}", inst);
        }
    }

// **************************************************** Semantic Analysis ****************************************************
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
    print_table(&SymbolTable);
    // **************************************************** Quadruplets ****************************************************

    // let mut quadruplet_generator = QuadrupletGenerator::new();
    // quadruplet_generator.generate_from_program(&program);
    //
    // println!("\nQuadruplets:");
    // quadruplet_generator.print_quadruplets();
}