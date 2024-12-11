use std::process::exit;
use once_cell::sync::Lazy;
use logos::Logos;
use crate::{grammar, Lexer};
use crate::Lexer::lexer::Token;
use crate::Parser::ast::Program;
use crate::Semantic::semantic_analyzer::SemanticAnalyzer;
use crate::Semantic::ts::print_table;
use crate::SymbolTable;

static INPUT: Lazy<&str> = Lazy::new(|| {
    r#"
    VAR_GLOBAL {
        INTEGER V = 0, X = 1, W = 2;
        FLOAT Y;
        CHAR E = '!';
        INTEGER Arr0[7] = [1, 2, 3, 4];
        INTEGER B = 4;
        INTEGER C = B + 4,
                A = B + C + 2;
        CHAR F = ' ';
        CHAR Arr5[3] = "";
        CHAR Arr3[6] = "Hello";
        FLOAT Arr4[5];
        FLOAT Arr1[B] = [1.2, .5, 2.0];
        CHAR Arr2[10] = ['S', 't', 'r', 'i', 'n', 'g'];
        CHAR I = 'X';
    }
    DECLARATION {
        CONST INTEGER D = 5;
        CONST FLOAT R = .6;
    }
    INSTRUCTION {
        B = B + 4;
        Arr2[1] = '1';
        Arr2[3] = '1';
        Arr1[1] = (Arr1[1] + Arr1[1]) / Arr1[2];
        Arr3[2] = 'L';
    }
    "#
});

#[test]
fn validate_lexical_analysis() {
    let mut lexer = Lexer::lexer::Token::lexer(&INPUT);
    let mut i = 0;
    while let Some(token) = lexer.next() {
        match token {
            Err(e) => {
                eprintln!("Lexical Error: {}", e);
                exit(1);
            },
            Ok(token) => {
                println!("{}: {:?}", i, token);
            }
        }
        i += 1;
    }
    println!("Lexical Analysis Successful.");
}
#[test]
fn test_syntactic_analysis() {
    use std::process::exit;

    // Tokenize the input using the lexer.
    let lexer = Lexer::lexer::Token::lexer(&INPUT);

    // Initialize the parser.
    let parser = grammar::ProgramParser::new();

    // Parse the input to generate the AST.
    match parser.parse(
        &INPUT,
        lexer
            .enumerate()
            .map(|(i, t)| t.map(|token| (i, token, i + 1)).map_err(|e| e))
    ) {
        Ok(program) => {
            println!("Syntactic Analysis Successful.");
            print_ast(&program);
        }
        Err(e) => {
            eprintln!("Syntactic Error: {:?}", e);
            exit(1);
        }
    }
}
#[test]
fn test_full_analysis_pipeline() {
    use std::process::exit;

    // Step 1: Lexical Analysis
    let lexer = Lexer::lexer::Token::lexer(&INPUT);
    let tokens: Vec<_> = lexer
        .enumerate()
        .map(|(i, t)| t.map(|token| (i, token, i + 1)).map_err(|e| e))
        .collect();

    // Check for lexical errors.
    if let Some(Err(e)) = tokens.iter().find(|t| t.is_err()) {
        eprintln!("Lexical Error: {:?}", e);
        exit(1);
    }
    println!("Lexical Analysis Successful.");

    // Step 2: Syntactic Analysis
    let parser = grammar::ProgramParser::new();
    let result = parser.parse(
        &INPUT,
        tokens.into_iter().map(|t| t.unwrap()), // Unwrap safe as errors were handled
    );

    let program = match result {
        Ok(ast) => {
            println!("Syntactic Analysis Successful.");
            print_ast(&ast); // Helper function to print the AST
            ast
        }
        Err(e) => {
            eprintln!("Syntactic Error: {:?}", e);
            exit(1);
        }
    };

    // Step 3: Semantic Analysis
    let mut semantic_analyzer = SemanticAnalyzer::new();
    match semantic_analyzer.analyze(&program) {
        Ok(_) => {
            println!("Semantic Analysis Successful.");
        }
        Err(msg) => {
            eprintln!("Semantic Error: {}", msg);
            exit(1);
        }
    }
    print_table(&SymbolTable);
}

fn print_ast(program: &Program) {
    println!("\nGenerated Abstract Syntax Tree (AST):");
    println!("{:#?}", program); // Pretty-print the AST for better readability.
}

