#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(nonstandard_style)]

use std::collections::HashMap;
use std::sync::atomic::Ordering;
use std::sync::Mutex;
use once_cell::sync::Lazy;
use logos::Logos;
use crate::error::CustomError;
use crate::Semantic::ts;

pub static SymbolTable: Lazy<Mutex<HashMap<String, ts::Symbol>>> = Lazy::new(|| Mutex::new(HashMap::new()));

// Validation functions Copy,
fn validate_identifier(lex: &logos::Lexer<Token>) -> Result<String, CustomError> {
    let Identifier = lex.slice().to_string();
    if Identifier.len() > 8 {
        Err(CustomError::IdentifierTooLong(Identifier))
    } else {
        if !ts::IB_FLAG.load(Ordering::SeqCst) {
            let symbol = ts::Symbol::new(Identifier.to_string(), None, None, None, None);
            SymbolTable.lock().unwrap()
                .insert(Identifier.as_str().to_string(), symbol);
        }
        Ok(Identifier)
    }
}

pub fn validate_integer(lex: &logos::Lexer<Token>) -> Result<i16, CustomError> {
    let slice = lex.slice();
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

fn Clear_BI_Flag(lex: &logos::Lexer<Token>) {
    ts::IB_FLAG.store(true, Ordering::SeqCst);
}
// Main token enum
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
#[logos(skip r"([ \t\n\f]+|%%[^\n]*)")]
pub enum Token {
    // Keywords
    #[token("VAR_GLOBAL", priority = 5)]
    VarGlobal,
    #[token("DECLARATION", priority = 5)]
    Declaration,
    #[token("INSTRUCTION", Clear_BI_Flag, priority = 5)]
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
    // Types
    #[token("INTEGER", priority = 5)]
    IntegerType,
    #[token("FLOAT", priority = 5)]
    FloatType,
    #[token("CHAR", priority = 5)]
    CharType,

    // Operators
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

    // comparison operators
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

    // Assignment
    #[token("=")]
    Assign,
    // delimiters
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

    // Constants and Identifiers
    #[regex(r"[A-Z][a-zA-Z0-9]*", validate_identifier)]
    Identifier(String),

    #[regex(r"[0-9]+", validate_integer)]
    Integer(i16),

    #[regex(r"[0-9]*\.[0-9]+", validate_float)]
    Float(f32),

    #[regex(r#"'[a-zA-Z]'"#, |lex| lex.slice().chars().nth(1))] // Single CHAR type
    Char(char),

    #[regex(r#""(?:[^"\\]|\\.)*""#, |lex| lex.slice().to_string())]
    StringLiteral(String),
}