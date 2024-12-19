#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(nonstandard_style)]

use std::collections::HashMap;
use std::sync::Mutex;
use logos::{Logos, Skip};
use once_cell::sync::Lazy;
use crate::Lexer::error::CustomError;
use crate::Semantic::ts::Symbol;

pub static lineNumber: Lazy<Mutex<u16>> = Lazy::new(|| Mutex::new(0u16));
pub static SymbolTable: Lazy<Mutex<HashMap<String, Symbol>>> = Lazy::new(|| Mutex::new(HashMap::new()));

fn validate_identifier(lex: &logos::Lexer<Token>) -> Result<(String, (usize, usize)), CustomError> {
    let Identifier = lex.slice().to_string();
    let line = lex.extras.0;
    let column = lex.span().start - lex.extras.1;
    if Identifier.len() > 8 {
        Err(CustomError::IdentifierTooLong(Identifier, (line, column)))
    }
    else {
        Ok((Identifier, (line, column)))
    }
}

pub fn validate_integer(lex: &logos::Lexer<Token>) -> Result<(i16, (usize, usize)), CustomError> {
    let slice = lex.slice();
    let line = lex.extras.0;
    let column = lex.span().start - lex.extras.1;
    // Parse as an i16 integer, supporting both positive and negative values
    match slice.parse::<i16>() {
        Ok(num) => Ok((num, (line, column))),
        Err(_) => Err(CustomError::IntegerOverflow(slice.to_string(), (line, column))),
    }
}

fn validate_float(lex: &logos::Lexer<Token>) -> Result<(f32, (usize, usize)), CustomError> {
    let slice = lex.slice();
    let line = lex.extras.0;
    let column = lex.span().start - lex.extras.1;
    match slice.parse::<f32>() {
        Ok(num) => Ok((num, (line, column))),
        Err(_) => Err(CustomError::FloatOverflow(slice.to_string(), (line, column))),
    }
}

fn validate_char(lex: &logos::Lexer<Token>) -> Result<(char, (usize, usize)), CustomError> {
    match lex.slice().chars().nth(1) {
        None => Err(CustomError::UnknownError),
        Some(c) => {
            let line = lex.extras.0;
            let column = lex.span().start - lex.extras.1;
            Ok((c, (line, column)))
        },
    }
}
fn validate_string_literal(lex: &logos::Lexer<Token>) -> Result<(String, (usize, usize)), CustomError> {
    let line = lex.extras.0;
    let column = lex.span().start - lex.extras.1;
    Ok((lex.slice().to_string(), (line, column)))
}
fn newline_callback(lex: &mut logos::Lexer<Token>) -> Skip {
    lex.extras.0 += 1;
    lex.extras.1 = lex.span().end;
    Skip
}
fn word_callback(lex: &mut logos::Lexer<Token>) -> (usize, usize) {
    let line = lex.extras.0;
    let column = lex.span().start - lex.extras.1;
    (line, column)
}

#[derive(Logos, Debug, PartialEq, Clone)]
pub enum Keyword{
    VarGlobal,
    Declaration,
    Instruction,
    Const,
    IF,
    ELSE,
    FOR,
}

pub enum Type{
    INTEGER,
    FLOAT,
    CHAR
}

#[derive(Logos, Debug, PartialEq,Clone)]
#[logos(error = CustomError)]
#[logos(extras = (usize, usize))]
#[logos(skip r"([ \t\f]+|%%[^\n]*)")]
pub enum Token {
    #[regex(r"\n", newline_callback)]
    NewLine,
    #[token("VAR_GLOBAL", word_callback, priority = 5)]
    VarGlobal((usize, usize)),
    #[token("DECLARATION", word_callback, priority = 5)]
    Declaration((usize, usize)),
    #[token("INSTRUCTION", word_callback, priority = 5)]
    Instruction((usize, usize)),
    #[token("CONST", word_callback, priority = 5)]
    Const((usize, usize)),
    #[token("READ", word_callback, priority = 5)]
    Read((usize, usize)),
    #[token("WRITE", word_callback, priority = 5)]
    Write((usize, usize)),
    #[token("IF", word_callback, priority = 5)]
    If((usize, usize)),
    #[token("ELSE", word_callback, priority = 5)]
    Else((usize, usize)),
    #[token("FOR", word_callback, priority = 5)]
    For((usize, usize)),

    #[token("INTEGER", word_callback, priority = 5)]
    IntegerType((usize, usize)),
    #[token("FLOAT", word_callback, priority = 5)]
    FloatType((usize, usize)),
    #[token("CHAR", word_callback, priority = 5)]
    CharType((usize, usize)),

    #[token("+", word_callback)]
    Plus((usize, usize)),
    #[token("-", word_callback)]
    Minus((usize, usize)),
    #[token("*", word_callback)]
    Multiply((usize, usize)),
    #[token("/", word_callback)]
    Divide((usize, usize)),
    #[token("&&", word_callback)]
    And((usize, usize)),
    #[token("||", word_callback)]
    Or((usize, usize)),
    #[token("!", word_callback)]
    Not((usize, usize)),

    #[token(">", word_callback)]
    GreaterThan((usize, usize)),
    #[token("<", word_callback)]
    LessThan((usize, usize)),
    #[token(">=", word_callback)]
    GreaterEqual((usize, usize)),
    #[token("<=", word_callback)]
    LessEqual((usize, usize)),
    #[token("==", word_callback)]
    Equal((usize, usize)),
    #[token("!=", word_callback)]
    NotEqual((usize, usize)),

    #[token("=", word_callback)]
    Assign((usize, usize)),
    #[token(";", word_callback)]
    Semicolon((usize, usize)),
    #[token("{", word_callback)]
    OpenBrace((usize, usize)),
    #[token("}", word_callback)]
    CloseBrace((usize, usize)),
    #[token("(", word_callback)]
    OpenParen((usize, usize)),
    #[token(")", word_callback)]
    CloseParen((usize, usize)),
    #[token(",", word_callback)]
    Comma((usize, usize)),
    #[token(":", word_callback)]
    Colon((usize, usize)),
    #[token("[", word_callback)]
    OpenBracket((usize, usize)),
    #[token("]", word_callback)]
    CloseBracket((usize, usize)),

    // Constants and Identifiers with strict ordering
    #[regex(r"[0-9]+", validate_integer, priority = 2)]
    Integer((i16, (usize, usize))),

    #[regex(r"[0-9]*\.[0-9]+", validate_float, priority = 2)]
    Float((f32, (usize, usize))),

    #[regex(r"[A-Z][a-zA-Z0-9]*", validate_identifier, priority = 1)]
    Identifier((String, (usize, usize))),

    #[regex(r"'[^']'", validate_char)]
    Char((char, (usize, usize))),

    // String literal should have lowest priority
    #[regex(r#""(?:[^"\\]|\\.)*""#, validate_string_literal)]
    StringLiteral((String, (usize, usize))),
}