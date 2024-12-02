#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]
#[cfg(test)]
mod tests {

    use crate::Lexer::lexer::{Keyword, Token,SymbolTable};
    use crate::Lexer::error::CustomError;
    use logos::Logos;
    use crate::Parser::ast::*;
    use crate::Semantic::ts::*;
    use crate::Semantic::type_checker::*;
    use crate::Semantic::semantic_rules::*;
    use crate::Semantic::semantic_analyzer::*;
    use crate::Semantic::error::SemanticError;
    use crate::Parser::*;

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
            println!("{}:\n{}", key, value);
        }
    }

    // Semantic Part

    // Table symbole (ts.rs)
    #[test]
    fn test_symbol_table_insert_and_lookup() {
        let mut table = SymbolTable::new();

        // Create and insert a symbol
        let symbol = Symbol::new(
            "VarA".to_string(),
            Some(Types::Integer),
            Some(false),
            Some(table.get_next_address()),
            None,
        );
        assert!(table.insert(symbol.clone()).is_ok());

        // Lookup the symbol
        let found_symbol = table.lookup("VarA");
        assert!(found_symbol.is_some());
        assert_eq!(found_symbol.unwrap().Identifier, "VarA");
    }

    #[test]
    fn test_duplicate_identifier() {
        let mut table = SymbolTable::new();

        let symbol1 = Symbol::new(
            "VarB".to_string(),
            Some(Types::Integer),
            Some(false),
            Some(table.get_next_address()),
            None,
        );
        assert!(table.insert(symbol1.clone()).is_ok());

        // Try inserting a duplicate identifier
        let symbol2 = Symbol::new(
            "VarB".to_string(),
            Some(Types::Float),
            Some(false),
            Some(table.get_next_address()),
            None,
        );
        assert!(table.insert(symbol2.clone()).is_err());
    }

    #[test]
    fn test_symbol_removal() {
        let mut table = SymbolTable::new();

        let symbol = Symbol::new(
            "VarC".to_string(),
            Some(Types::Char),
            Some(false),
            Some(table.get_next_address()),
            None,
        );
        assert!(table.insert(symbol.clone()).is_ok());

        // Remove the symbol
        let removed_symbol = table.remove("VarC");
        assert!(removed_symbol.is_some());
        assert_eq!(removed_symbol.unwrap().Identifier, "VarC");

        // Ensure it's no longer in the table
        let found_symbol = table.lookup("VarC");
        assert!(found_symbol.is_none());
    }

    // Type checker (type_checker.rs)

    #[test]
    fn test_arithmetic_compatibility() {
        assert!(TypeChecker::check_arithmetic_compatibility(&Types::Integer, &Types::Integer).is_ok());
        assert!(TypeChecker::check_arithmetic_compatibility(&Types::Integer, &Types::Float).is_ok());
        assert!(TypeChecker::check_arithmetic_compatibility(&Types::Float, &Types::Float).is_ok());
        assert!(TypeChecker::check_arithmetic_compatibility(&Types::Char, &Types::Integer).is_err());
    }

    #[test]
    fn test_assignment_compatibility() {
        // Valid assignment
        assert!(TypeChecker::check_assignment_compatibility(&Types::Integer, &Types::Integer).is_ok());

        // Implicit conversion from Float to Integer
        assert!(TypeChecker::check_assignment_compatibility(&Types::Integer, &Types::Float).is_ok());

        // Array compatibility
        assert!(TypeChecker::check_assignment_compatibility(
            &Types::Array(Box::new(Types::Integer), 3),
            &Types::Array(Box::new(Types::Integer), 3)
        )
        .is_ok());

        // Invalid assignment
        assert!(TypeChecker::check_assignment_compatibility(
            &Types::Array(Box::new(Types::Float), 3),
            &Types::Array(Box::new(Types::Integer), 3)
        )
        .is_err());
    }

    #[test]
    fn test_infer_expression_type() {
        assert_eq!(
            TypeChecker::infer_expression_type(&TypeValue::Integer(42)),
            Types::Integer
        );
        assert_eq!(
            TypeChecker::infer_expression_type(&TypeValue::Float(3.14)),
            Types::Float
        );
        assert_eq!(
            TypeChecker::infer_expression_type(&TypeValue::Char('c')),
            Types::Char
        );

        // Array inference
        assert_eq!(
            TypeChecker::infer_expression_type(&TypeValue::Array(vec![
                TypeValue::Integer(1),
                TypeValue::Integer(2),
                TypeValue::Integer(3)
            ])),
            Types::Array(Box::new(Types::Integer), 3)
        );
    }

    // Type checker + table symbole (both)

    #[test]
    fn test_symbol_table_with_type_checker() {
        let mut table = SymbolTable::new();
    
        // Insert a symbol with no scope (global is implicit)
        let symbol = Symbol::new(
            "VarX".to_string(),
            Some(Types::Integer),
            Some(false),
            Some(table.get_next_address()),
            None, // Value is not required for this test
        );
        assert!(table.insert(symbol.clone()).is_ok());
    
        // Lookup the symbol
        let found_symbol = table.lookup("VarX").unwrap();
    
        // Verify type compatibility for assignment
        assert!(TypeChecker::check_assignment_compatibility(
            found_symbol.Type.as_ref().unwrap(),
            &Types::Integer
        )
        .is_ok());
    }
    

    // sementic Rules 

    #[test]
    fn test_validate_variable_declaration() {
        // Valid variable declaration
        assert!(SemanticRules::validate_variable_declaration(
            "x",
            &Types::Integer,
            false,
            Some(&TypeValue::Integer(42))
        )
        .is_ok());

        // Identifier exceeds maximum length
        let result = SemanticRules::validate_variable_declaration(
            "toolongid",
            &Types::Integer,
            false,
            None,
        );
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Identifier 'toolongid' cannot exceed 8 characters"
        );

        // Constant without initialization
        let result = SemanticRules::validate_variable_declaration(
            "CONST",
            &Types::Float,
            true,
            None,
        );
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Constant 'CONST' must be initialized at declaration"
        );

        // Type mismatch in initialization
        let result = SemanticRules::validate_variable_declaration(
            "y",
            &Types::Char,
            false,
            Some(&TypeValue::Integer(5)),
        );
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Type mismatch: cannot assign Integer to Char"
        );

        // Valid constant declaration with initialization
        let result = SemanticRules::validate_variable_declaration(
            "CONST",
            &Types::Float,
            true,
            Some(&TypeValue::Float(3.14)),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_array_declaration() {
        // Valid array declaration
        assert!(SemanticRules::validate_array_declaration("arr", &Types::Integer, 5).is_ok());

        // Identifier exceeds maximum length
        let result = SemanticRules::validate_array_declaration("toolongid", &Types::Float, 10);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Identifier 'toolongid' cannot exceed 8 characters"
        );

        // Array size is zero
        let result = SemanticRules::validate_array_declaration("arr", &Types::Float, 0);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Array 'arr' must have a positive size");

        // Invalid array type
        let result = SemanticRules::validate_array_declaration(
            "arr",
            &Types::Array(Box::new(Types::Integer), 5),
            10,
        );
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid array type for 'arr'");
    }

    #[test]
    fn test_validate_condition() {
        // Valid condition type
        assert!(SemanticRules::validate_condition(&Types::Integer).is_ok());

        // Invalid condition type: Float
        let result = SemanticRules::validate_condition(&Types::Float);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Condition must be an integer expression"
        );

        // Invalid condition type: Char
        let result = SemanticRules::validate_condition(&Types::Char);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Condition must be an integer expression"
        );
    }

    #[test]
    fn test_complex_variable_initialization() {
        // Valid array variable initialization
        let value = TypeValue::Array(vec![
            TypeValue::Integer(1),
            TypeValue::Integer(2),
            TypeValue::Integer(3),
        ]);
        let result = SemanticRules::validate_variable_declaration(
            "arr",
            &Types::Array(Box::new(Types::Integer), 3),
            false,
            Some(&value),
        );
        assert!(result.is_ok());

        // Invalid array initialization (type mismatch)
        let value = TypeValue::Array(vec![
            TypeValue::Integer(1),
            TypeValue::Float(2.0),
            TypeValue::Integer(3),
        ]);
        let result = SemanticRules::validate_variable_declaration(
            "arr",
            &Types::Array(Box::new(Types::Integer), 3),
            false,
            Some(&value),
        );
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Type mismatch: cannot assign Array to Array of Integer"
        );
    }

    // Semantic Analyzer

    #[test]
    fn test_valid_variable_declaration() {
        let mut analyzer = SemanticAnalyzer::new();

        // Valid variable declaration
        let result = analyzer.analyze_variable_declaration(
            "x".to_string(),
            Types::Integer,
            false,
            Some(TypeValue::Integer(42)),
        );
        assert!(result.is_ok(), "Valid variable declaration failed");
    }

    #[test]
    fn test_valid_array_declaration() {
        let mut analyzer = SemanticAnalyzer::new();

        // Valid array declaration
        let result = analyzer.analyze_array_declaration(
            "arr".to_string(),
            Types::Integer,
            10,
        );
        assert!(result.is_ok(), "Valid array declaration failed");
    }

}