#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(nonstandard_style)]
mod codegen;
mod Lexer;
mod Parser;
mod Semantic;
mod Test;
use std::{env, fs};
use std::collections::HashMap;
use std::process::exit;
use std::sync::Mutex;
use lalrpop_util;
use lalrpop_util::lalrpop_mod;
use logos::Logos;
use once_cell::sync::Lazy;
use crate::Parser::ast::BinOp;
// use crate::Semantic::quadruplets::QuadrupletGenerator;
use crate::Semantic::semantic_analyzer::SemanticAnalyzer;
use crate::Semantic::ts::*;
use colored::*;

lalrpop_mod!(pub grammar, "/Parser/grammar.rs");
pub static SymbolTable: Lazy<Mutex<HashMap<String, Symbol>>> = Lazy::new(|| Mutex::new(HashMap::new()));

const DEFAULT_PROGRAM: &str = r#"
    VAR_GLOBAL {
    %% Testing global variables
    INTEGER Global1;
    FLOAT Global2;
    CHAR Global3;
}

DECLARATION {
    %% Testing all types of declarations
    %% Simple variables
    INTEGER Var1, Var2, Var3;
    FLOAT F1, F2;
    CHAR C1, C2;

    %% Arrays
    INTEGER Array1[10];
    FLOAT Array2[5];
    CHAR String1[8];

    %% Constants
    CONST INTEGER MaxVal = 32767;
    CONST FLOAT Pi = 3.14159;
    CONST FLOAT NegPi = (-3.14159);
    CONST CHAR Grade = 'A';
}

INSTRUCTION {
    %% Testing assignments with complex expressions
    Var1 = (-15);
    Var2 = (-25);
    F1 = 3.14;
    F2 = (-2.718);
    C1 = 'X';

    %% Testing arithmetic operations with mixed types
    Var3 = (Var1 + Var2) * 3 / 2;
    F2 = (F1 * 2.0) + (-1.5);

    %% Testing array operations
    Array1[0] = 100;
    Array2[4] = 99.9;
    String1[0] = 'H';

    %% Testing input/output
    WRITE("Please enter a number: ");
    READ(Var1);
    WRITE("You entered ", Var1, " which is ", Var1 , " for positive");

    %% Testing nested if conditions with logical operators
    IF (Var1 > 0 && Var1 < 100) {
        WRITE("Number is between 0 and 100");
        IF (Var1 >= 50) {
            WRITE("Number is >= 50");
        } ELSE {
            WRITE("Number is < 50");
        }
    } ELSE {
        IF (Var1 <= 0 || Var1 >= 100) {
            WRITE("Number is outside range");
        }
    }

    %% Testing comparison operators
    IF (Var1 == Var2) {
        WRITE("Equal");
    }
    IF (Var1 != Var2) {
        WRITE("Not equal");
    }
    IF (Var1 <= Var2) {
        WRITE("Less or equal");
    }
    IF (Var1 >= Var2) {
        WRITE("Greater or equal");
    }

    %% Testing logical operators
    IF (!((Var1 > 0) && (Var2 < 0))) {
        WRITE("Complex logical condition");
    }

    %% Testing FOR loop with nested conditions
    FOR(Var1 = 1 : 1 : 10) {
        IF ((Var1 / 2) == 0) {
            WRITE("Even number: ", Var1);
        } ELSE {
            WRITE("Odd number: ", Var1);
        }

        %% Testing nested loops
        FOR(Var2 = 0 : 2 : 6) {
            Array1[Var2] = Var1 * Var2;
        }
    }

    %% Testing complex arithmetic expressions
    F1 = 5.5 / (F2 - (-2.5));
    Var3 = MaxVal / 2 + MaxVal * (-1);

    %% Testing string operations
    String1[0] = 'T';
    String1[1] = 'E';
    String1[2] = 'S';
    String1[3] = 'T';

    %% Testing boundary conditions
    Array1[9] = MaxVal;
    Array2[0] = (-99.99);

    %% Testing multiple operations in one expression
    Var1 = (MaxVal + MinVal) / 2 * (1 + 2 * 3) / 4;
}
    "#;

fn process_program(input: &str, is_default: bool) {
    if is_default {
        println!("{}", "No input file provided or file reading failed. Running default example:".yellow());
        println!("{}", input);
        println!("-------------------------------------------------------------------------------------------------");
    }

    println!("{}", "Printing found tokens: ".blue());
    let mut lexer = Lexer::lexer::Token::lexer(input);
    let mut i = 0;
    while let Some(token) = lexer.next() {
        match token {
            Err(e) => {
                eprintln!("{} {} token number {}", "Lexical Error:".red(), e,i);
                exit(1);
            },
            Ok(token) => {
               //println!("{}: {:?}", i, token);
                i += 1;
            }
        }
    }
    println!("{}", "Lexical Analysis Successful.".green());
    println!("-------------------------------------------------------------------------------------------------");
    println!();

    let lexer = Lexer::lexer::Token::lexer(input);
    let parser = grammar::ProgramParser::new();
    let result = parser.parse(input, lexer.enumerate().map(|(i, t)| t.map(|token| (i, token, i+1)).map_err(|e| e)));
    let program = match result {
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

    let mut semanticAnalyzer = SemanticAnalyzer::new();
    let semantic_result = semanticAnalyzer.analyze(&program);
    match semantic_result {
        Ok(_) => {
            println!("{}", "Semantic Analysis Successful.".green());
            println!("-------------------------------------------------------------------------------------------------");
            println!();
            println!("{}", "Printing the contents of the abstract syntax tree: ".yellow());

            if let Some(globals) = &program.global {
                println!("{}", "\nGlobal Variables:".blue());
                for decl in globals {
                    println!("{:?}", decl);
                }
            }

            if let Some(decls) = &program.decls {
                println!("{}", "\nDeclarations:".blue());
                for decl in decls {
                    println!("{:?}", decl);
                }
            }

            if let Some(instructions) = &program.inst {
                println!("{}", "\nInstructions:".blue());
                for inst in instructions {
                    println!("{:?}", inst);
                }
            }
        },
        Err(msg) => {
            eprintln!("{} {} at token {:?}", "Semantic Error:".red(), msg, program);
            exit(1);
        },
    }

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
