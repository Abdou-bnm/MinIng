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
// use crate::Lexer::lexer::Token;
use logos::Logos;

lalrpop_mod!(pub grammar, "/Parser/grammar.rs");

fn main() {
    let input = r#"
    VAR_GLOBAL {
        INTEGER V, X, W;
        FLOAT Y;
        CHAR NamesNames[10];
        INTEGER I;
    }
    DECLARATION {
        CONST INTEGER D = 5;
        CONST FLOAT R = .6;
    }
    INSTRUCTION {
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
    assert!(result.is_ok());
}