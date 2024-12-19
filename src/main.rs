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
use std::fs;
use std::env;
// Import LALRPOP utilities
use lalrpop_util;
use lalrpop_util::lalrpop_mod;
use logos::Logos;
use once_cell::sync::Lazy;
use crate::Parser::ast::BinOp;
use crate::Semantic::quadruplets::QuadrupletGenerator;
use crate::Semantic::semantic_analyzer::SemanticAnalyzer;
use crate::Semantic::ts::*;
use colored::*;

lalrpop_mod!(pub grammar, "/Parser/grammar.rs");
pub static SymbolTable: Lazy<Mutex<HashMap<String, Symbol>>> = Lazy::new(|| Mutex::new(HashMap::new()));

// Example program to use as fallback
const DEFAULT_PROGRAM: &str = r#"
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
        FOR( B = 2 : 6 : 10) {
            IF(B < 5) { B = B+1; } ELSE {B = 0;}
        }
        Arr3[2] = 'D';
    }
    "#;

fn process_program(input: &str, is_default: bool) {
    if is_default {
        println!("{}", "No input file provided or file reading failed. Running default example:".yellow());
        println!("{}", input);
        println!("-------------------------------------------------------------------------------------------------");
    }

    // **************************************************** Lexical Analysis ****************************************************
    println!("{}", "Printing found tokens: ".blue());
    let mut lexer = Lexer::lexer::Token::lexer(input);
    let mut i = 0;
    while let Some(token) = lexer.next() {
        match token {
            Err(e) => {
                eprintln!("{} {}", "Lexical Error:".red(), e);
                exit(1);
            },
            Ok(token) => {
                println!("{}: {:?}", i, token);
                i += 1;
            }
        }
    }
    println!("{}", "Lexical Analysis Successful.".green());
    println!("-------------------------------------------------------------------------------------------------");
    println!();

    // **************************************************** Syntactic Analysis ****************************************************
    let lexer = Lexer::lexer::Token::lexer(input);
    let parser = grammar::ProgramParser::new();
    let result = parser.parse(input, lexer.enumerate().map(|(i, t)| t.map(|token| (i, token, i+1)).map_err(|e| e)));
    let program  = match result {
        Ok(t) => {
            println!("{}", "Syntactic Analysis Successful.".green());
            println!("-------------------------------------------------------------------------------------------------");
            t
        },
        Err(e) => {
            eprintln!("{} {:?}", "Syntactic Error: {}".red(), e);
            exit(1);
        },
    };
    // **************************************************** Semantic Analysis ****************************************************
    let mut semanticAnalyzer = SemanticAnalyzer::new();
    let semantic_result = semanticAnalyzer.analyze(&program);

    match semantic_result {
        Ok(_) => {
            println!("{}", "Semantic Analysis Successful.".green());
            println!("-------------------------------------------------------------------------------------------------");
            println!();
            println!("{}", "Printing the contents of the abstract syntax tree: ".yellow());

            // Print Global Variables
            if let Some(globals) = &program.global {
                println!("{}", "\nGlobal Variables:".blue());
                for decl in globals {
                    println!("{:?}", decl);
                }
            }

            // Print Declarations
            if let Some(decls) = &program.decls {
                println!("{}", "\nDeclarations:".blue());
                for decl in decls {
                    println!("{:?}", decl);
                }
            }

            // Print Instructions
            if let Some(instructions) = &program.inst {
                println!("{}", "\nInstructions:".blue());
                for inst in instructions {
                    println!("{:?}", inst);
                }
            }
        },
        Err(msg) => {
            eprintln!("{} {}", "Semantic Error:".red(), msg);
            exit(1);
        },
    }

    // **************************************************** Symbol Table ****************************************************
    println!("-------------------------------------------------------------------------------------------------");
    println!("{}", "The contents of the symbols table".green());
    print_table(&SymbolTable);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = if args.len() > 1 {
        match fs::read_to_string(&args[1]) {
            Ok(content) => {
                println!("{} {}", "Reading from file:".blue(), args[1]);
                (content, false)
            },
            Err(e) => {
                eprintln!("{} {}: {}", "Error reading file".red(), args[1], e);
                (DEFAULT_PROGRAM.to_string(), true)
            }
        }
    } else {
        (DEFAULT_PROGRAM.to_string(), true)
    };

    process_program(&program.0, program.1);
}