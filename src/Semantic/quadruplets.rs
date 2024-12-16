use crate::Semantic::ts::{Symbol, Types};
use crate::Parser::ast::*;
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

    /// Generate quadruplets for a single instruction
    fn generate_instruction(&mut self, instruction: &Instruction) {
        match instruction {
            Instruction::Assign(assign) => self.generate_assignment(assign),
            Instruction::If(if_stmt) => self.generate_if_statement(if_stmt),
            Instruction::For(for_stmt) => self.generate_for_statement(for_stmt),
            Instruction::Read(read_stmt) => self.generate_read_statement(read_stmt),
            Instruction::Write(write_stmt) => self.generate_write_statement(write_stmt),
        }
    }
    /// Generate quadruplets for an expression, returning the temporary variable name
    pub fn generate_from_program(&mut self, program: &Program) {
        // Handle global variable declarations if any
        if let Some(globals) = &program.global {
            for decl in globals {
                self.generate_declaration(decl);
            }
        }

        // Handle declarations
        if let Some(decls) = &program.decls {
            for decl in decls {
                self.generate_declaration(decl);
            }
        }
        
        if let Some(instructions) = &program.inst {
            for instruction in instructions {
                self.generate_instruction(instruction);
            }
        }
    }

    fn generate_declaration(&mut self, declaration: &Declaration) {
        match declaration {
            // Handling simple variable declarations
            Declaration::Variables(type_, variables) => self.generate_declarations_variable(type_, variables),

            // Handling array declarations
            Declaration::Array(type_, array_decls) => self.generate_declarations_array(type_, array_decls),

            // Handling constant declarations (though this is typically done in the declarations section)
            Declaration::Constant(type_, assignments) => self.generate_declarations_constant(type_, assignments),
        }
    }
    
    fn generate_declarations_variable(&mut self, type_: &Type, variables: &Vec<Variable>) {
        for var in variables {
            match var {
                Variable::Simple(name) => {
                    // For uninitialized variables, we might just add a placeholder
                    self.add_quadruplet(Quadruplet::new(
                        Operator::Assign,
                        Some("0".to_string()),  // Default initialization
                        None,
                        Some(name.clone())
                    ));
                },
                Variable::Initialized(name, expr) => {
                    // For initialized variables, generate expression and assign
                    let result_temp = self.generate_expression(expr);

                    self.add_quadruplet(Quadruplet::new(
                        Operator::Assign,
                        Some(result_temp),
                        None,
                        Some(name.clone())
                    ));
                }
            }
        }
    }
    
    fn generate_declarations_array(&mut self, type_: &Type, array_decls: &Vec<ArrayDecl>) {
        for arr in array_decls {
            match arr {
                ArrayDecl::Simple(name, size) => {
                    // For uninitialized arrays, we might just generate a size quadruplet
                    let size_temp = self.generate_expression(size);

                    self.add_quadruplet(Quadruplet::new(
                        Operator::Assign,
                        Some(size_temp),
                        None,
                        Some(format!("size_{}", name))
                    ));
                },
                ArrayDecl::Initialized(name, size, initializers) => {
                    // Generate size quadruplet
                    let size_temp = self.generate_expression(size);

                    self.add_quadruplet(Quadruplet::new(
                        Operator::Assign,
                        Some(size_temp),
                        None,
                        Some(format!("size_{}", name))
                    ));

                    // Initialize array elements
                    for (index, init_expr) in initializers.iter().enumerate() {
                        let init_temp = self.generate_expression(init_expr);

                        self.add_quadruplet(Quadruplet::new(
                            Operator::Assign,
                            Some(init_temp),
                            None,
                            Some(format!("{}[{}]", name, index))
                        ));
                    }
                },
                ArrayDecl::InitializedString(name, size, string_val) => {
                    // Generate size quadruplet
                    let size_temp = self.generate_expression(size);

                    self.add_quadruplet(Quadruplet::new(
                        Operator::Assign,
                        Some(size_temp),
                        None,
                        Some(format!("size_{}", name))
                    ));

                    // Initialize array with string characters
                    for (index, ch) in string_val.chars().enumerate() {
                        self.add_quadruplet(Quadruplet::new(
                            Operator::Assign,
                            Some(ch.to_string()),
                            None,
                            Some(format!("{}[{}]", name, index))
                        ));
                    }
                }
            }
        }
    }
    
    fn generate_declarations_constant(&mut self, type_: &Type, assignments: &Vec<Assignment>) {
        for assign in assignments {
            let result_temp = self.generate_expression(&assign.expr);

            self.add_quadruplet(Quadruplet::new(
                Operator::Assign,
                Some(result_temp),
                None,
                Some(assign.var.clone())
            ));
        }
    }
    
    fn generate_assignment(&mut self, assignment: &Assignment) {
        let result_temp = self.generate_expression(&assignment.expr);
        
        match &assignment.index {
            None => self.add_quadruplet(Quadruplet::new(
                        Operator::Assign,
                        Some(result_temp),
                        None,
                        Some(format!("{}", assignment.var))
                    )),
            Some(index) => {
                let index_temp = self.generate_expression(&assignment.index.clone().unwrap());
                self.add_quadruplet(Quadruplet::new(
                    Operator::Assign,
                    Some(result_temp),
                    None,
                    Some(format!("{}[{}]", assignment.var, index_temp))
                ))
            }
        }
    }
    
    fn generate_expression(&mut self, expr: &Expr) -> String {
        match expr {
            Expr::BinaryOp(left, op, right) => {
                let left_temp = self.generate_expression(left);
                let right_temp = self.generate_expression(right);

                let result_temp = self.generate_temp();

                // Convert BinOp to Operator
                let operator = match op {
                    BinOp::Add => Operator::Add,
                    BinOp::Sub => Operator::Subtract,
                    BinOp::Mul => Operator::Multiply,
                    BinOp::Div => Operator::Divide,
                };

                // Add quadruplet for binary operation
                self.add_quadruplet(Quadruplet::new(
                    operator,
                    Some(left_temp),
                    Some(right_temp),
                    Some(result_temp.clone())
                ));

                result_temp
            },
            Expr::Variable(var) => var.clone(),
            Expr::Array(name, index) => {
                let index_temp = self.generate_expression(index);
                let result_temp = self.generate_temp();

                // Create quadruplet for array access
                self.add_quadruplet(Quadruplet::new(
                    Operator::Assign,
                    Some(format!("{}[{}]", name, index_temp)),
                    None,
                    Some(result_temp.clone())
                ));

                result_temp
            },
            Expr::Literal(lit) => {
                // For literals, we can use their string representation
                match lit {
                    TypeValue::Integer(i) => i.to_string(),
                    TypeValue::Float(f) => f.to_string(),
                    TypeValue::Char(c) => c.to_string(),
                    TypeValue::Array(_) => {
                        // This case might need more sophisticated handling
                        "array_literal".to_string()
                    }
                }
            },
        }
    }

    /// Generate quadruplets for an if statement
    fn generate_if_statement(&mut self, if_stmt: &IfStmt) {
        // TODO: Implement more complex condition handling
        // For now, this is a simplistic implementation
        let condition_temp = match &if_stmt.condition {
            Condition::Basic(basic_cond) => {
                let left_temp = self.generate_expression(&basic_cond.left);
                let right_temp = self.generate_expression(&basic_cond.right);

                let condition_temp = self.generate_temp();

                // Convert RelOp to comparison operator
                let operator = match basic_cond.operator {
                    RelOp::Gt => Operator::GreaterThan,
                    RelOp::Lt => Operator::LessThan,
                    RelOp::Ge => Operator::GreaterThanOrEqual,
                    RelOp::Le => Operator::LessThanOrEqual,
                    RelOp::Eq => Operator::Equal,
                    RelOp::Ne => Operator::NotEqual,
                };

                // Add comparison quadruplet
                self.add_quadruplet(Quadruplet::new(
                    operator,
                    Some(left_temp),
                    Some(right_temp),
                    Some(condition_temp.clone())
                ));

                condition_temp
            },
            // More complex condition handling would go here
            _ => "condition".to_string(),
        };

        // Quadruplet for conditional jump
        self.add_quadruplet(Quadruplet::new(
            Operator::IfFalse,
            Some(condition_temp),
            None,
            Some("L_else".to_string())
        ));

        // Generate quadruplets for then block
        for inst in &if_stmt.then_block {
            self.generate_instruction(inst);
        }

        // Generate quadruplets for else block if it exists
        if let Some(else_block) = &if_stmt.else_block {
            self.add_quadruplet(Quadruplet::new(
                Operator::Goto,
                None,
                None,
                Some("L_end".to_string())
            ));

            // Label for else block
            self.add_quadruplet(Quadruplet::new(
                Operator::Assign,
                None,
                None,
                Some("L_else".to_string())
            ));

            for inst in else_block {
                self.generate_instruction(inst);
            }

            // End label
            self.add_quadruplet(Quadruplet::new(
                Operator::Assign,
                None,
                None,
                Some("L_end".to_string())
            ));
        }
    }

    /// Generate quadruplets for a for statement
    fn generate_for_statement(&mut self, for_stmt: &ForStmt) {
        // Generate initialization quadruplet
        self.generate_instruction(&Instruction::Assign(for_stmt.init.clone()));

        // Start label for loop
        self.add_quadruplet(Quadruplet::new(
            Operator::Assign,
            None,
            None,
            Some("L_loop_start".to_string())
        ));

        // Condition check
        let condition_temp = self.generate_expression(&for_stmt.condition);

        // Conditional jump
        self.add_quadruplet(Quadruplet::new(
            Operator::IfFalse,
            Some(condition_temp),
            None,
            Some("L_loop_end".to_string())
        ));

        // Loop body
        for inst in &for_stmt.body {
            self.generate_instruction(inst);
        }

        // Step generation
        let step_temp = self.generate_expression(&for_stmt.step);
        let loop_var = for_stmt.init.var.clone();

        // Increment loop variable
        self.add_quadruplet(Quadruplet::new(
            Operator::Add,
            Some(loop_var.clone()),
            Some(step_temp),
            Some(loop_var.clone())
        ));

        // Unconditional jump back to loop start
        self.add_quadruplet(Quadruplet::new(
            Operator::Goto,
            None,
            None,
            Some("L_loop_start".to_string())
        ));

        // Loop end label
        self.add_quadruplet(Quadruplet::new(
            Operator::Assign,
            None,
            None,
            Some("L_loop_end".to_string())
        ));
    }

    /// Generate quadruplets for a read statement
    fn generate_read_statement(&mut self, read_stmt: &ReadStmt) {
        if let Some(index) = &read_stmt.index {
            let index_temp = self.generate_expression(index);

            // Array read
            self.add_quadruplet(Quadruplet::new(
                Operator::Read,
                None,
                Some(index_temp.clone()),
                Some(format!("{}[{}]", read_stmt.variable, index_temp))
            ));
        }
        else {
            // Simple variable read
            self.add_quadruplet(Quadruplet::new(
                Operator::Read,
                None,
                None,
                Some(read_stmt.variable.clone())
            ));
        }
    }

    /// Generate quadruplets for a write statement
    fn generate_write_statement(&mut self, write_stmt: &WriteStmt) {
        for element in &write_stmt.elements {
            self.generate_write_element(element);     
        }
    }
    
    fn generate_write_element(&mut self, write_element: &WriteElement) {
        match write_element {
            WriteElement::String(s) => {
                self.add_quadruplet(Quadruplet::new(
                    Operator::Write,
                    Some(s.clone()),
                    None,
                    None
                ));
            },
            WriteElement::Variable(var, index) => {
                if let Some(idx) = index {
                    let index_temp = self.generate_expression(idx);
        
                    // Write array element
                    self.add_quadruplet(Quadruplet::new(
                        Operator::Write,
                        Some(format!("{}[{}]", var, index_temp)),
                        None,
                        None
                    ));
                } else {
                    // Write simple variable
                    self.add_quadruplet(Quadruplet::new(
                        Operator::Write,
                        Some(var.clone()),
                        None,
                        None
                    ));
                }
            }
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
    use TypeValue;

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