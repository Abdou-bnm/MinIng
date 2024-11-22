#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]
#[cfg(test)]
mod tests {
    use crate::lexer::{Keyword, Token};
    use crate::error::CustomError;
    use logos::Logos;
    use crate::lexer::SymbolTable;

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
            %% THIS IS A COMMENT
            VAR_GLOBAL {
                INTEGER V,X, W;
            %% THIS IS A COMMENT
                FLOAT Y;
                CHAR Name[10];
            }
            DECLARATION {
            %% THIS IS A COMMENT
                CONST INTEGER D = 5;
                CONST FLOAT R = .6;
            }
            INSTRUCTION {
                INTEGER Var0;
                N = 10;
                IF (X > 0) {
                    WRITE(\"X is positive\");
                } ELSE {
                    WRITE(\"x is non-positive\");
                }
                FOR (I = 0:  2 : N) {
                    WRITE(I);
                }
            }
        ");
        println!("Will start printing the tokens...");
        for token in lexer.by_ref() {
            match token {
                Ok(token) => println!("{:#?}", token),
                Err(e) => println!("some error occurred: {:?}", e),
            }
        }

        println!("\n\nWill start printing the Symbol Table...");
        let table = SymbolTable.lock().unwrap();
        for (key, value) in table.iter() {
            println!("{}", value);
        }
    }
    #[test]
    fn test_invalid_identifiers_and_overflows() {
        let mut lexer = Token::lexer("VERYLONGID");
        assert_eq!(lexer.next(), Some(Err(CustomError::IdentifierTooLong("VERYLONGID".to_string()))));

        let mut lexer = Token::lexer("2147483648"); // Overflows i16
        assert_eq!(lexer.next(), Some(Err(CustomError::IntegerOverflow("2147483648".to_string()))));
        // let mut lexer = Token::lexer("340282350000000000000000000000000000001.0"); // Overflows f32
        // assert_eq!(lexer.next(), Some(Err(CustomError::FloatOverflow("340282350000000000000000000000000000001.0".to_string()))));
    }
}