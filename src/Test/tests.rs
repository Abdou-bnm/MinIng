#![cfg(test)]

use logos::Logos;
use crate::{grammar, Lexer};
use crate::Semantic::semantic_analyzer::SemanticAnalyzer;
use super::*;

#[test]
fn test_lexical_error() {
    let program = r#"
        VAR_GLOBAL {
            INTEGER 123InvalidName;
        }
    "#;
    let mut lexer = Lexer::lexer::Token::lexer(program);
    while let Some(token) = lexer.next() {
        if token.is_err() {
            assert!(true, "Lexical error detected as expected");
            return;
        }
    }
    panic!("Expected a lexical error but none occurred");
}

#[test]
fn test_syntactic_error() {
    let program = r#"
        VAR_GLOBAL {
            INTEGER A
        }
        DECLARATION {
            CONST INTEGER B = 5;
        }
    "#; // Missing semicolon after `INTEGER A`
    let lexer = Lexer::lexer::Token::lexer(program);
    let parser = grammar::ProgramParser::new();
    let result = parser.parse(program, lexer.enumerate().map(|(i, t)| t.map(|token| (i, token, i+1)).map_err(|e| e)));
    assert!(result.is_err(), "Syntactic error detected as expected");
}

#[test]
fn test_semantic_error_undeclared_variable() {
    let program = r#"
        VAR_GLOBAL {
            INTEGER A;
        }
        INSTRUCTION {
            B = 5;
        }
    "#; // `B` is not declared
    let lexer = Lexer::lexer::Token::lexer(program);
    let parser = grammar::ProgramParser::new();
    let parse_result = parser.parse(program, lexer.enumerate().map(|(i, t)| t.map(|token| (i, token, i+1)).map_err(|e| e)));
    assert!(parse_result.is_ok(), "Parsing should succeed");

    let mut semanticAnalyzer = SemanticAnalyzer::new();
    let semantic_result = semanticAnalyzer.analyze(&parse_result.unwrap());
    assert!(semantic_result.is_err(), "Semantic error detected as expected");
}

#[test]
fn test_semantic_error_type_mismatch() {
    let program = r#"
        VAR_GLOBAL {
            INTEGER A;
            FLOAT B;
        }
        INSTRUCTION {
            A = B + 3.14;
        }
    "#; // Type mismatch: assigning FLOAT to INTEGER
    let lexer = Lexer::lexer::Token::lexer(program);
    let parser = grammar::ProgramParser::new();
    let parse_result = parser.parse(program, lexer.enumerate().map(|(i, t)| t.map(|token| (i, token, i+1)).map_err(|e| e)));
    assert!(parse_result.is_ok(), "Parsing should succeed");

    let mut semanticAnalyzer = SemanticAnalyzer::new();
    let semantic_result = semanticAnalyzer.analyze(&parse_result.unwrap());
    assert!(semantic_result.is_err(), "Semantic error detected as expected");
}

#[test]
fn test_semantic_error_array_bounds() {
    let program = r#"
        VAR_GLOBAL {
            INTEGER Arr[3] = [1, 2, 3];
        }
        INSTRUCTION {
            Arr[5] = 10;
        }
    "#; // Out-of-bounds array access
    let lexer = Lexer::lexer::Token::lexer(program);
    let parser = grammar::ProgramParser::new();
    let parse_result = parser.parse(program, lexer.enumerate().map(|(i, t)| t.map(|token| (i, token, i+1)).map_err(|e| e)));
    assert!(parse_result.is_ok(), "Parsing should succeed");

    let mut semanticAnalyzer = SemanticAnalyzer::new();
    let semantic_result = semanticAnalyzer.analyze(&parse_result.unwrap());
    assert!(semantic_result.is_err(), "Semantic error detected as expected");
}

#[test]
fn test_semantic_error_const_assignment() {
    let program = r#"
        DECLARATION {
            CONST INTEGER A = 10;
        }
        INSTRUCTION {
            A = 5;
        }
    "#; // Attempt to modify a constant
    let lexer = Lexer::lexer::Token::lexer(program);
    let parser = grammar::ProgramParser::new();
    let parse_result = parser.parse(program, lexer.enumerate().map(|(i, t)| t.map(|token| (i, token, i+1)).map_err(|e| e)));
    assert!(parse_result.is_ok(), "Parsing should succeed");

    let mut semanticAnalyzer = SemanticAnalyzer::new();
    let semantic_result = semanticAnalyzer.analyze(&parse_result.unwrap());
    assert!(semantic_result.is_err(), "Semantic error detected as expected");
}
