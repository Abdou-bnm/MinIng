#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]
use logos::Logos;
use crate::error::CustomError;

// Validation functions
pub fn validate_identifier(lex: &logos::Lexer<Token>) -> Result<String, CustomError> {
    let ident = lex.slice().to_string();
    if ident.len() > 8 {
        Err(CustomError::IdentifierTooLong(ident))
    } else {
        Ok(ident)
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

// fn validate_char_array(lex: &logos::Lexer<Token>) -> Result<String, CustomError> {
//     let slice = lex.slice().to_string();
//     let parts: Vec<&str> = slice.split(|c| c == '[' || c == ']').collect();
//     let identifier = parts[0].to_string();
//     if identifier.len() > 8 {
//         return Err(CustomError::IdentifierTooLong(identifier));
//     }
//     let length: usize = parts[1].parse().map_err(|_| CustomError::InvalidNumberFormat(parts[1].to_string()))?;
//     Ok(slice)
// }
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
#[derive(Logos, Debug, PartialEq)]
#[logos(error = CustomError)]
#[logos(skip r"([ \t\n\f]+|%%[^\n]*)")]
pub enum Token {
    // Keywords
    #[token("VAR_GLOBAL",priority=5)]
    VarGlobal,
    #[token("DECLARATION", priority=5 )]
    Declaration,
    #[token("INSTRUCTION", priority=5 )]
    Instruction,
    #[token("CONST", priority=5 )]
    Const,
    #[token("READ", priority=5 )]
    Read,
    #[token("WRITE", priority=5 )]
    Write,
    #[token("IF", priority=5 )]
    If,
    #[token("ELSE", priority=5 )]
    Else,
    #[token("FOR", priority=5 )]
    For,
    // Types
    #[token("INTEGER", priority=5 )]
    IntegerType,
    #[token("FLOAT", priority=5 )]
    FloatType,
    #[token("CHAR", priority=5 )]
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
    #[regex(r"[A-Z][a-zA-Z0-9]*",validate_identifier)]
    Identifier(String),

    #[regex(r"[0-9]+",validate_integer)]
    Integer(i16),

    #[regex(r"[0-9]*\.[0-9]+", validate_float)]
    Float(f32),

    #[regex(r#"'[a-zA-Z]'"#, |lex| lex.slice().chars().nth(1))] // Single CHAR type
    Char(char),

    // #[regex(r"[A-Z][a-zA-Z0-9]{0,7}\[[0-9]+\]", validate_char_array)]
    // CharArray(String),

    #[regex(r#""(?:[^"\\]|\\.)*""#, |lex| lex.slice().to_string())]
    StringLiteral(String),
}