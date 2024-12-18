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
use std::{env, fs};
use std::process::exit;
use std::sync::Mutex;
// Import LALRPOP utilities
use lalrpop_util;
use lalrpop_util::lalrpop_mod;
use logos::Logos;
use once_cell::sync::Lazy;
use crate::Parser::ast::BinOp;
// use crate::Semantic::quadruplets::QuadrupletGenerator;
use crate::Semantic::semantic_analyzer::SemanticAnalyzer;
use crate::Semantic::ts::*;

lalrpop_mod!(pub grammar, "/Parser/grammar.rs");
pub static SymbolTable: Lazy<Mutex<HashMap<String, Symbol>>> = Lazy::new(|| Mutex::new(HashMap::new()));

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        println!("Usage: cargo run -- <file>");
        exit(1);
    }
    let input: String;
    
    match fs::read_to_string(args[1].clone()) {
        Ok(t) => input = t,
        Err(e) => {
            eprintln!("File Read Error: file '{}' {}", args[1].clone(), e);
            exit(1);
        }
    };
    
// **************************************************** Lexical Analysis ****************************************************
// Display of all tokens, enumerated
//     let mut i = 0;
//     let mut lexer = Lexer::lexer::Token::lexer(input);
//     while let Some(token) = lexer.next() {
//         println!("{}: {:?}", i, token);
//         i += 1;
//     }

// Prints errors found in the lexical analysis phase
    let mut lexer = Lexer::lexer::Token::lexer(input.as_str());
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
    let lexer = Lexer::lexer::Token::lexer(input.as_str());
    let parser = grammar::ProgramParser::new();
    let result = parser.parse(input.as_str(), lexer.enumerate().map(|(i, t)| t.map(|token| (i, token, i+1)).map_err(|e| e)));
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