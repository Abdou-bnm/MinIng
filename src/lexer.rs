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

pub fn validate_integer(lex: &logos::Lexer<Token>) -> Result<i32, CustomError> {
    let slice = lex.slice();
    match slice.parse::<i32>() {
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

fn validate_char_array(lex: &logos::Lexer<Token>) -> Result<String, CustomError> {
    let slice = lex.slice().to_string();
    let parts: Vec<&str> = slice.split(|c| c == '[' || c == ']').collect();
    let identifier = parts[0].to_string();
    if identifier.len() > 8 {
        return Err(CustomError::IdentifierTooLong(identifier));
    }
    let length: usize = parts[1].parse().map_err(|_| CustomError::InvalidNumberFormat(parts[1].to_string()))?;
    Ok(slice)
}
// Main token enum
#[derive(Logos, Debug, PartialEq)]
pub enum Token {
    // Keywords
    #[token("VAR_GLOBAL")]
    VarGlobal,
    #[token("DECLARATION")]
    Declaration,
    #[token("INSTRUCTION")]
    Instruction,
    #[token("CONST")]
    Const,
    #[token("READ")]
    Read,
    #[token("WRITE")]
    Write,
    #[token("IF")]
    If,
    #[token("ELSE")]
    Else,
    #[token("FOR")]
    For,

    // Types
    #[token("INTEGER")]
    IntegerType,
    #[token("FLOAT")]
    FloatType,
    #[token("CHAR")]
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

    // Assignment and punctuation
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

    // Constants and Identifiers
    #[regex(r"[A-Z][a-zA-Z0-9]{0,7}", crate::validate_identifier)]
    Identifier(String),

    #[regex(r"[0-9]+", validate_integer)]
    Integer(i32),

    #[regex(r"[0-9]*\.[0-9]+", validate_float)]
    Float(f32),

    #[regex(r#"'[a-zA-Z]'"#, |lex| lex.slice().chars().nth(1))] // Single CHAR type
    Char(char),

    #[regex(r"[A-Z][a-zA-Z0-9]{0,7}\[[0-9]+\]", validate_char_array)]
    CharArray(String),

    #[regex(r#""(?:[^"\\]|\\.)*""#, |lex| lex.slice().to_string())]
    StringLiteral(String),
    // Comment and whitespace
    #[regex(r"%%[^\n]*", logos::skip, priority = 5)]
    Comment,
    #[regex(r"[ \t\n\f]+", logos::skip,priority = 2)]
    Whitespace,

    // Custom error handling for unrecognized tokens
    #[regex(r"[^A-Za-z0-9+\-*/(){};=<>!&|.%\[\]\s]", |lex| CustomError::UnrecognizedToken(lex.slice().to_string()))]
    Error(CustomError),
}


