#[cfg(test)]
mod tests {
    use crate::lexer::Token;
    use crate::error::CustomError;
    use logos::Logos;

    #[test]
    fn test_keywords_and_types() {
        let mut lexer = Token::lexer("VAR_GLOBAL DECLARATION INSTRUCTION INTEGER FLOAT CHAR");
        assert_eq!(lexer.next(), Some(Ok(Token::VarGlobal)));
        assert_eq!(lexer.next(), Some(Ok(Token::Declaration)));
        assert_eq!(lexer.next(), Some(Ok(Token::Instruction)));
        assert_eq!(lexer.next(), Some(Ok(Token::IntegerType)));
        assert_eq!(lexer.next(), Some(Ok(Token::FloatType)));
        assert_eq!(lexer.next(), Some(Ok(Token::CharType)));
    }

    #[test]
    fn test_full_program() {
        let mut lexer = Token::lexer("
            VAR_GLOBAL {
                INTEGER X;
                FLOAT Y;
                CHAR Name[10];
            }
            DECLARATION {
                CONST INTEGER D = 5;
            }
            INSTRUCTION {
                IF (X > 0) {
                    WRITE(\"X is positive\");
                } ELSE {
                    WRITE(\"x is non-positive\");
                }
                FOR (I = 0; I < 10; I = I + 1) {
                    WRITE(I);
                }
            }
        ");
        for token in lexer.by_ref() {
            match token {
                Ok(token) => println!("{:#?}", token),
                Err(e) => println!("some error occurred: {:?}", e),
            }
        }
        // assert_eq!(lexer.next(), Some(Ok(Token::VarGlobal)));
        // assert_eq!(lexer.next(), Some(Ok(Token::OpenBrace)));
        // assert_eq!(lexer.next(), Some(Ok(Token::IntegerType)));
        // assert_eq!(lexer.next(), Some(Ok(Token::Identifier("x".to_string()))));
        // assert_eq!(lexer.next(), Some(Ok(Token::Semicolon)));
        // assert_eq!(lexer.next(), Some(Ok(Token::FloatType)));
        // assert_eq!(lexer.next(), Some(Ok(Token::Identifier("y".to_string()))));
        // assert_eq!(lexer.next(), Some(Ok(Token::Semicolon)));
        // assert_eq!(lexer.next(), Some(Ok(Token::CharType)));
        // assert_eq!(lexer.next(), Some(Token::CharArray("name".to_string(), 10)));
    }

    #[test]
    fn test_invalid_identifiers_and_overflows() {
        let mut lexer = Token::lexer("VERYLONGID");
        assert_eq!(lexer.next(), Some(Ok(Token::Error(CustomError::IdentifierTooLong("VERYLONGID".to_string())))));

        let mut lexer = Token::lexer("2147483648"); // Overflows i32
        assert_eq!(lexer.next(), Some(Ok(Token::Error(CustomError::IntegerOverflow("2147483648".to_string())))));
    }
}
