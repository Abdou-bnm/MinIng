#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(nonstandard_style)]

use std::collections::HashMap;
use std::sync::Mutex;
use logos::Logos;
use once_cell::sync::Lazy;
use crate::Lexer::error::CustomError;
use crate::Semantic::ts::Symbol;

pub static lineNumber: Lazy<Mutex<u16>> = Lazy::new(|| Mutex::new(0u16));
pub static SymbolTable: Lazy<Mutex<HashMap<String, Symbol>>> = Lazy::new(|| Mutex::new(HashMap::new()));

fn validate_identifier(lex: &logos::Lexer<Token>) -> Result<String, CustomError> {
    let Identifier = lex.slice().to_string();
    if Identifier.len() > 8 {
        Err(CustomError::IdentifierTooLong(Identifier))
    }
    else {
        Ok(Identifier)
    }
}

pub fn validate_integer(lex: &logos::Lexer<Token>) -> Result<i16, CustomError> {
    let slice = lex.slice();

    // Parse as an i16 integer, supporting both positive and negative values
    match slice.parse::<i16>() {
        Ok(num) => Ok(num),
        Err(_) => Err(CustomError::IntegerOverflow(slice.to_string())),
    }
}

fn validate_float(lex: &logos::Lexer<Token>) -> Result<f32, CustomError> {
    let slice = lex.slice();
    match slice.parse::<f32>() {
        Ok(num) => Ok(num),
        Err(_) => Err(CustomError::FloatOverflow(slice.to_string())),
    }
}

fn newline_callback(lex: &logos::Lexer<Token>) {
    let lineNumberClone = lineNumber.lock().unwrap().clone();
    *lineNumber.lock().unwrap() = lineNumberClone + 1;
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
#[logos(skip r"([ \n\t\f]+|%%[^\n]*)")]
pub enum Token {
    #[token("VAR_GLOBAL", priority = 5)]
    VarGlobal,
    #[token("DECLARATION", priority = 5)]
    Declaration,
    #[token("INSTRUCTION", priority = 5)]
    Instruction,
    #[token("CONST", priority = 5)]
    Const,
    #[token("READ", priority = 5)]
    Read,
    #[token("WRITE", priority = 5)]
    Write,
    #[token("IF", priority = 5)]
    If,
    #[token("ELSE", priority = 5)]
    Else,
    #[token("FOR", priority = 5)]
    For,

    #[token("INTEGER", priority = 5)]
    IntegerType,
    #[token("FLOAT", priority = 5)]
    FloatType,
    #[token("CHAR", priority = 5)]
    CharType,

    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Multiply,
    #[token("/")]
    Divide,
    #[token("&&")]
    And,
    #[token("||")]
    Or,
    #[token("!")]
    Not,

    #[token(">")]
    GreaterThan,
    #[token("<")]
    LessThan,
    #[token(">=")]
    GreaterEqual,
    #[token("<=")]
    LessEqual,
    #[token("==")]
    Equal,
    #[token("!=")]
    NotEqual,

    #[token("=")]
    Assign,
    #[token(";")]
    Semicolon,
    #[token("{")]
    OpenBrace,
    #[token("}")]
    CloseBrace,
    #[token("(")]
    OpenParen,
    #[token(")")]
    CloseParen,
    #[token(",")]
    Comma,
    #[token(":")]
    Colon,
    #[token("[")]
    OpenBracket,
    #[token("]")]
    CloseBracket,

    // Constants and Identifiers with strict ordering
    #[regex(r"-?[0-9]+", validate_integer, priority = 2)]
    Integer(i16),

    #[regex(r"-?[0-9]*\.[0-9]+", validate_float, priority = 2)]
    Float(f32),

    #[regex(r"[A-Z][a-zA-Z0-9]*", validate_identifier, priority = 1)]
    Identifier(String),

    #[regex(r"'[^']'", |lex| lex.slice().chars().nth(1), priority = 2)]
    Char(char),

    // String literal should have lowest priority
    #[regex(r#""(?:[^"\\]|\\.)*""#, |lex| lex.slice().to_string(), priority = 0)]
    StringLiteral(String),
}