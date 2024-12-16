use crate::Semantic::ts::{Symbol, Types};
use crate::Parser::ast::{ArrayDecl, Assignment, ForStmt, IfStmt, ReadStmt, Type, TypeValue, Variable, WriteStmt};
use crate::Lexer::lexer::Token;
use std::collections::HashMap;
use std::sync::Mutex;
use once_cell::sync::Lazy;

/// Represents an operation in the intermediate representation
#[derive(Debug, Clone)]
pub enum Operator {
    // Arithmetic Operators
    Add,
    Subtract,
    Multiply,
    Divide,

    // Comparison Operators
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
    Equal,
    NotEqual,

    // Logical Operators
    LogicalAnd,
    LogicalOr,
    LogicalNot,

    // Assignment
    Assign,

    // Input/Output
    Read,
    Write,

    // Control Flow
    Goto,
    IfTrue,
    IfFalse,
    For,
}


impl From<Token> for Operator {
    fn from(token: Token) -> Self {
        match token {
            Token::Plus => Operator::Add,
            Token::Minus => Operator::Subtract,
            Token::Multiply => Operator::Multiply,
            Token::Divide => Operator::Divide,
            Token::GreaterThan => Operator::GreaterThan,
            Token::LessThan => Operator::LessThan,
            Token::GreaterEqual => Operator::GreaterThanOrEqual,
            Token::LessEqual => Operator::LessThanOrEqual,
            Token::Equal => Operator::Equal,
            Token::NotEqual => Operator::NotEqual,
            Token::And => Operator::LogicalAnd,
            Token::Or => Operator::LogicalOr,
            Token::Not => Operator::LogicalNot,
            Token::Assign => Operator::Assign,
            Token::Read => Operator::Read,
            Token::Write => Operator::Write,
            Token::For => Operator::For,
            _ => panic!("Cannot convert token to operator"),
        }
    }
}


#[derive(Debug, Clone)]
pub struct Quadruplet {
    pub operator: Operator,
    pub operand1: Option<String>,
    pub operand2: Option<String>,
    pub result: Option<String>,
}

impl Quadruplet {

    pub fn new(
        operator: Operator,
        operand1: Option<String>,
        operand2: Option<String>,
        result: Option<String>
    ) -> Self {
        Quadruplet {
            operator,
            operand1,
            operand2,
            result,
        }
    }


    pub fn to_string(&self) -> String {
        format!(
            "({:?}, {}, {}, {})",
            self.operator,
            self.operand1.as_ref().unwrap_or(&"_".to_string()),
            self.operand2.as_ref().unwrap_or(&"_".to_string()),
            self.result.as_ref().unwrap_or(&"_".to_string())
        )
    }
}


#[derive(Debug)]
pub struct QuadrupletGenerator {
    quadruplets: Vec<Quadruplet>,
    temp_counter: usize,
}

impl QuadrupletGenerator {

    pub fn new() -> Self {
        QuadrupletGenerator {
            quadruplets: Vec::new(),
            temp_counter: 0,
        }
    }


    pub fn generate_temp(&mut self) -> String {
        self.temp_counter += 1;
        format!("t{}", self.temp_counter)
    }


    pub fn add_quadruplet(&mut self, quadruplet: Quadruplet) {
        self.quadruplets.push(quadruplet);
    }


    pub fn get_quadruplets(&self) -> &Vec<Quadruplet> {
        &self.quadruplets
    }

    /// Print all quadruplets
    pub fn print_quadruplets(&self) {
        for (index, quad) in self.quadruplets.iter().enumerate() {
            println!("{}: {}", index, quad.to_string());
        }
    }
}


fn type_value_to_string(value: &TypeValue) -> String {
    match value {
        TypeValue::Integer(i) => i.to_string(),
        TypeValue::Float(f) => f.to_string(),
        TypeValue::Char(c) => c.to_string(),
        TypeValue::Array(_) => "Array".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Parser::ast::TypeValue;

    #[test]
    fn test_quadruplet_generation() {
        let mut generator = QuadrupletGenerator::new();

        let temp1 = generator.generate_temp();
        generator.add_quadruplet(Quadruplet::new(
            Operator::Add,
            Some("a".to_string()),
            Some("b".to_string()),
            Some(temp1.clone())
        ));

        let temp2 = generator.generate_temp();
        generator.add_quadruplet(Quadruplet::new(
            Operator::Multiply,
            Some(temp1),
            Some("2".to_string()),
            Some(temp2)
        ));

        generator.print_quadruplets();
    }

    #[test]
    fn test_type_value_conversion() {
        // Test conversion of TypeValue to string
        assert_eq!(
            type_value_to_string(&TypeValue::Integer(42)),
            "42"
        );
        assert_eq!(
            type_value_to_string(&TypeValue::Float(3.14)),
            "3.14"
        );
        assert_eq!(
            type_value_to_string(&TypeValue::Char('A')),
            "A"
        );
    }
}