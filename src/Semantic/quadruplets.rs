use crate::Semantic::ts::{Symbol, Types};
use crate::Parser::ast::*;
use crate::Lexer::lexer::Token;
use crate::Lexer::error::CustomError;
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
            Token::Plus(_) => Operator::Add,
            Token::Minus(_) => Operator::Subtract,
            Token::Multiply(_) => Operator::Multiply,
            Token::Divide(_) => Operator::Divide,
            Token::GreaterThan(_) => Operator::GreaterThan,
            Token::LessThan(_) => Operator::LessThan,
            Token::GreaterEqual(_) => Operator::GreaterThanOrEqual,
            Token::LessEqual(_) => Operator::LessThanOrEqual,
            Token::Equal(_) => Operator::Equal,
            Token::NotEqual(_) => Operator::NotEqual,
            Token::And(_) => Operator::LogicalAnd,
            Token::Or(_) => Operator::LogicalOr,
            Token::Not(_) => Operator::LogicalNot,
            Token::Assign(_) => Operator::Assign,
            Token::Read(_) => Operator::Read,
            Token::Write(_) => Operator::Write,
            Token::For(_) => Operator::For,
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
    error_handler: Vec<CustomError>,
}

impl QuadrupletGenerator {
    pub fn new() -> Self {
        QuadrupletGenerator {
            quadruplets: Vec::new(),
            temp_counter: 0,
            error_handler: Vec::new(),
        }
    }

    /// Generates a new temporary variable name
    /// Returns a string in the format "t{number}" where number is incremented for each new temp
    pub fn generate_temp(&mut self) -> String {
        self.temp_counter += 1;
        format!("t{}", self.temp_counter)
    }

    /// Adds a quadruplet to the list of generated quadruplets
    /// Returns a reference to the added quadruplet
    pub fn add_quadruplet(&mut self, quadruplet: Quadruplet) -> &Quadruplet {
        self.quadruplets.push(quadruplet);
        self.quadruplets.last().unwrap()
    }

    /// Returns all generated quadruplets
    pub fn get_quadruplets(&self) -> &Vec<Quadruplet> {
        &self.quadruplets
    }

    /// Prints all quadruplets with their index and formatted representation
    pub fn print_quadruplets(&self) {
        println!("\nGenerated Quadruplets:");
        println!("----------------------");
        for (index, quad) in self.quadruplets.iter().enumerate() {
            println!("{}: {}", index, quad.to_string());
        }
        println!("----------------------\n");
    }

    fn generate_expression(&mut self, expr: &Expr) -> Result<String, CustomError> {
        match expr {
            Expr::BinaryOp(left, op, right) => {
                let left_temp = self.generate_expression(left)?;
                let right_temp = self.generate_expression(right)?;
                let result_temp = self.generate_temp();

                let operator = match op {
                    BinOp::Add(_, _) => Operator::Add,
                    BinOp::Sub(_, _) => Operator::Subtract,
                    BinOp::Mul(_, _) => Operator::Multiply,
                    BinOp::Div(_, _) => Operator::Divide,
                };
                self.add_quadruplet(Quadruplet::new(
                    operator,
                    Some(left_temp),
                    Some(right_temp),
                    Some(result_temp.clone())
                ));

                Ok(result_temp)
            },
            Expr::Variable((name, (line, col))) => {
                // We can use the line and col information for error reporting if needed
                Ok(name.clone())
            },
            Expr::SUBS(name, index) => {
                let index_temp = self.generate_expression(index)?;
                let result_temp = self.generate_temp();

                self.add_quadruplet(Quadruplet::new(
                    Operator::Assign,
                    Some(format!("{}[{}]", name, index_temp)),
                    None,
                    Some(result_temp.clone())
                ));

                Ok(result_temp)
            },
            Expr::Literal(lit) => Ok(type_value_to_string(lit)),
        }
    }

    pub fn get_errors(&self) -> &Vec<CustomError> {
        &self.error_handler
    }

    pub fn has_errors(&self) -> bool {
        !self.error_handler.is_empty()
    }

    fn add_error(&mut self, error: CustomError) {
        self.error_handler.push(error);
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
    use TypeValue;

    #[test]
    fn test_quadruplet_generation() {
        let mut generator = QuadrupletGenerator::new();

        // Test temporary variable generation
        let temp1 = generator.generate_temp();
        assert_eq!(temp1, "t1");
        let temp2 = generator.generate_temp();
        assert_eq!(temp2, "t2");

        // Test quadruplet addition
        generator.add_quadruplet(Quadruplet::new(
            Operator::Add,
            Some("a".to_string()),
            Some("b".to_string()),
            Some(temp1.clone())
        ));

        assert!(!generator.has_errors());

        generator.add_quadruplet(Quadruplet::new(
            Operator::Multiply,
            Some(temp1),
            Some("2".to_string()),
            Some(temp2)
        ));

        // Test quadruplet retrieval
        let quadruplets = generator.get_quadruplets();
        assert_eq!(quadruplets.len(), 2);

        // Test printing (this just verifies it doesn't panic)
        generator.print_quadruplets();
    }

    #[test]
    fn test_error_handling() {
        let mut generator = QuadrupletGenerator::new();
        generator.add_error(CustomError::IdentifierTooLong("toolongident".to_string(), (1, 1)));
        assert!(generator.has_errors());
        assert_eq!(generator.get_errors().len(), 1);
    }
}