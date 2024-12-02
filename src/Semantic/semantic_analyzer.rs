use std::fmt::Debug;
use crate::Semantic::ts::{Types, TypeValue, Symbol, SymbolTable};
use crate::Semantic::type_checker::TypeChecker;
use crate::Semantic::semantic_rules::SemanticRules;
use crate::Parser::ast::{Program, Instruction, Expr, Declaration, Assignment, Condition, Type, IfStmt, BasicCond, RelOp, Literal, ReadStmt, WriteStmt, WriteElement, ArrayDecl};

pub struct SemanticAnalyzer {
    symbol_table: SymbolTable,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        SemanticAnalyzer {
            symbol_table: SymbolTable::new(),
        }
    }

    pub fn analyze(&mut self, program: &Program) -> Result<(), String> {
        // Analyze global variables
        if let Some(global_vars) = &program.global {
            self.analyze_declarations(global_vars)?;
        }

        // Analyze declarations
        if let Some(declarations) = &program.decls {
            self.analyze_declarations(declarations)?;
        }

        // Analyze instructions
        if let Some(instructions) = &program.inst {
            self.analyze_instructions(instructions)?;
        }

        Ok(())
    }

    fn analyze_declarations(&mut self, declarations: &Vec<Declaration>) -> Result<(), String> {
        for decl in declarations {
            match decl {
                Declaration::Variables(type_decl, vars) => {
                    for var in vars {
                        match type_decl {
                            Type::Integer => self.validate_variable(&Types::Integer, var)?,
                            Type::Float => self.validate_variable(&Types::Float, var)?,
                            Type::Char => self.validate_variable(&Types::Char, var)?,
                        }
                    }
                },
                Declaration::Array(type_decl, arrays) => {
                    for arr in arrays {
                        match type_decl {
                            Type::Integer => self.validate_array(&Types::Integer, arr)?,
                            Type::Float => self.validate_array(&Types::Float, arr)?,
                            Type::Char => self.validate_array(&Types::Char, arr)?,
                        }
                    }
                },
                Declaration::Constant(type_decl, constants) => {
                    for constant in constants {
                        match type_decl {
                            Type::Integer => self.validate_constant(&Types::Integer, constant)?,
                            Type::Float => self.validate_constant(&Types::Float, constant)?,
                            Type::Char => self.validate_constant(&Types::Char, constant)?,
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn validate_variable(&mut self, type_decl: &Types, var: &crate::Parser::ast::Variable) -> Result<(), String> {
        match var {
            crate::Parser::ast::Variable::Simple(name) => {
                let symbol = Symbol::new(
                    name.clone(),
                    Some(type_decl.clone()),
                    Some(false),
                    None,
                    None
                );
                self.symbol_table.insert(symbol)?;
                SemanticRules::validate_variable_declaration(
                    name,
                    type_decl,
                    false,
                    None
                )
            },
            crate::Parser::ast::Variable::Initialized(name, expr) => {
                let value_type = self.infer_expression_type(expr)?;
                let symbol = Symbol::new(
                    name.clone(),
                    Some(type_decl.clone()),
                    Some(false),
                    None,
                    Some(self.convert_to_type_value(expr)?)
                );
                self.symbol_table.insert(symbol)?;
                SemanticRules::validate_variable_declaration(
                    name,
                    type_decl,
                    false,
                    Some(&self.convert_to_type_value(expr)?)
                )
            }
        }
    }

    fn validate_array(&mut self, type_decl: &Types, arr: &ArrayDecl) -> Result<(), String> {
        match arr {
            ArrayDecl::Simple(name, size_expr) => {
                let size = self.evaluate_array_size(size_expr)?;
                let symbol = Symbol::new(
                    name.clone(),
                    Some(Types::Array(Box::new(type_decl.clone()), size)),
                    Some(false),
                    None,
                    None
                );
                self.symbol_table.insert(symbol)?;
                SemanticRules::validate_array_declaration(name, type_decl, size)
            },
            ArrayDecl::Initialized(name, size_expr, values) => {
                let size = self.evaluate_array_size(size_expr)?;
                // Additional type checking for initialized arrays
                self.validate_array_initialization(type_decl, values)?;
                let symbol = Symbol::new(
                    name.clone(),
                    Some(Types::Array(Box::new(type_decl.clone()), size)),
                    Some(false),
                    None,
                    Some(self.convert_array_to_type_value(values)?)
                );
                self.symbol_table.insert(symbol)?;
                SemanticRules::validate_array_declaration(name, type_decl, size)
            },
            ArrayDecl::InitializedString(name, size_expr, value) => {
                let size = self.evaluate_array_size(size_expr)?;
                let symbol = Symbol::new(
                    name.clone(),
                    Some(Types::Array(Box::new(type_decl.clone()), size)),
                    Some(false),
                    None,
                    Some(TypeValue::Array(
                        value.chars()
                            .map(|c| TypeValue::Char(c))
                            .collect()
                    ))
                );
                self.symbol_table.insert(symbol)?;
                SemanticRules::validate_array_declaration(name, type_decl, size)
            }
        }
    }

    fn validate_constant(
        &mut self,
        type_decl: &Types,
        constant: &Assignment
    ) -> Result<(), String> {
        let value_type = self.infer_expression_type(&constant.expr)?;
        TypeChecker::check_assignment_compatibility(type_decl, &value_type)?;

        let symbol = Symbol::new(
            constant.var.clone(),
            Some(type_decl.clone()),
            Some(true),
            None,
            Some(self.convert_to_type_value(&constant.expr)?)
        );
        self.symbol_table.insert(symbol)?;

        SemanticRules::validate_variable_declaration(
            &constant.var,
            type_decl,
            true,
            Some(&self.convert_to_type_value(&constant.expr)?)
        )
    }

    fn analyze_instructions(&mut self, instructions: &Vec<Instruction>) -> Result<(), String> {
        for instruction in instructions {
            match instruction {
                Instruction::Assign(assignment) => self.validate_assignment(assignment)?,
                Instruction::If(if_stmt) => self.validate_if_statement(if_stmt)?,
                Instruction::For(for_loop) => self.validate_for_loop(for_loop)?,
                Instruction::Read(read_stmt) => self.validate_read(read_stmt)?,
                Instruction::Write(write_stmt) => self.validate_write(write_stmt)?,
            }
        }
        Ok(())
    }

    // Implement other validation methods here:
    // validate_assignment, validate_if_statement, validate_for_loop,
    // validate_read, validate_write...
    fn validate_assignment(&mut self, assignment: &Assignment) -> Result<(), String> {
        // Check if variable exists in symbol table
        let symbol = self.symbol_table.lookup(&assignment.var)
            .ok_or_else(|| format!("Undefined variable: {}", assignment.var))?;

        // Check if variable is constant
        if symbol.Is_Constant{
            return Err(format!("Cannot reassign constant variable: {}", assignment.var));
        }

        // Infer type of expression
        let expr_type = self.infer_expression_type(&assignment.expr)?;

        // Check type compatibility
        TypeChecker::check_assignment_compatibility(
            symbol.Type.as_ref().ok_or_else(|| format!("No type for variable: {}", assignment.var))?,
            &expr_type
        )?;

        Ok(())
    }

    fn validate_if_statement(&mut self, if_stmt: &IfStmt) -> Result<(), String> {
        // Create a type-checking closure that can be passed to validate_condition
        let type_check_closure = |condition: &Condition| -> Result<Types, String> {
            match condition {
                Condition::Not(inner_condition) => {
                    // Recursively infer type for inner condition
                    self.infer_condition_type(inner_condition)
                },
                Condition::Logic(left_cond, op, right_cond) => {
                    // Validate both sides of logical conditions
                    let left_type = self.infer_condition_type(left_cond)?;
                    let right_type = self.infer_condition_type(right_cond)?;

                    // Ensure both sides resolve to integer
                    if left_type == Types::Integer && right_type == Types::Integer {
                        Ok(Types::Integer)
                    } else {
                        Err("Logical conditions must resolve to integer expressions".to_string())
                    }
                },
                Condition::Basic(basic_cond) => {
                    // Validate basic condition's operands
                    let left_type = self.infer_expression_type(&basic_cond.left)?;
                    let right_type = self.infer_expression_type(&basic_cond.right)?;

                    // Ensure types are compatible for comparison
                    if TypeChecker::are_types_compatible(&left_type, &right_type) {
                        Ok(Types::Integer)
                    } else {
                        Err(format!("Incompatible types in condition: {:?} and {:?}", left_type, right_type))
                    }
                }
            }
        };

        // Validate the condition
        SemanticRules::validate_condition(&if_stmt.condition, &type_check_closure)?;

        // Validate then block instructions
        if let Some(then_instructions) = &if_stmt.then_block {
            self.analyze_instructions(then_instructions)?;
        }

        // Validate else block instructions if present
        if let Some(else_instructions) = &if_stmt.else_block {
            self.analyze_instructions(else_instructions)?;
        }

        Ok(())
    }

    // Add a helper method to infer condition type
    fn infer_condition_type(&self, condition: &Condition) -> Result<Types, String> {
        match condition {
            Condition::Not(inner_condition) => {
                // Recursively infer type for inner condition
                self.infer_condition_type(inner_condition)
            },
            Condition::Logic(_, _, _) => Ok(Types::Integer),
            Condition::Basic(basic_cond) => {
                // Validate basic condition's operands
                let left_type = self.infer_expression_type(&basic_cond.left)?;
                let right_type = self.infer_expression_type(&basic_cond.right)?;

                // Ensure types are compatible for comparison
                if TypeChecker::are_types_compatible(&left_type, &right_type) {
                    Ok(Types::Integer)
                } else {
                    Err(format!("Incompatible types in condition: {:?} and {:?}", left_type, right_type))
                }
            }
        }
    }

    fn validate_for_loop(&mut self, for_loop: &crate::Parser::ast::ForStmt) -> Result<(), String> {
        // Validate initialization variable
        let init_symbol = self.symbol_table.lookup(&for_loop.init.var)
            .ok_or_else(|| format!("Undefined loop variable: {}", for_loop.init.var))?;

        // Validate initialization expression type
        let init_type = self.infer_expression_type(&for_loop.init.expr)?;
        TypeChecker::check_assignment_compatibility(
            init_symbol.Type.as_ref().ok_or_else(|| format!("No type for loop variable: {}", for_loop.init.var))?,
            &init_type
        )?;

        // Validate step type (should be same as initialization type)
        let step_type = self.infer_expression_type(&for_loop.step)?;
        if step_type != init_type {
            return Err("Step type must match initialization type".to_string());
        }

        // Create a type-checking closure for the condition
        let type_check_closure = |condition: &Condition| -> Result<Types, String> {
            match condition {
                Condition::Not(inner_condition) => {
                    // Recursively infer type for inner condition
                    self.infer_condition_type(inner_condition)
                },
                Condition::Logic(left_cond, _, right_cond) => {
                    // Validate both sides of logical conditions
                    let left_type = self.infer_condition_type(left_cond)?;
                    let right_type = self.infer_condition_type(right_cond)?;

                    // Ensure both sides resolve to integer
                    if left_type == Types::Integer && right_type == Types::Integer {
                        Ok(Types::Integer)
                    } else {
                        Err("Logical conditions must resolve to integer expressions".to_string())
                    }
                },
                Condition::Basic(basic_cond) => {
                    // Validate basic condition's operands
                    let left_type = self.infer_expression_type(&basic_cond.left)?;
                    let right_type = self.infer_expression_type(&basic_cond.right)?;

                    // Ensure types are compatible for comparison
                    if TypeChecker::are_types_compatible(&left_type, &right_type) {
                        Ok(Types::Integer)
                    } else {
                        Err(format!("Incompatible types in condition: {:?} and {:?}", left_type, right_type))
                    }
                }
            }
        };

        // Validate condition
        // Note: for loops expect a condition expression, so we'll convert it to a Condition first
        let condition = Condition::Basic(BasicCond {
            left: for_loop.condition.clone(),
            operator: RelOp::Lt, // Default to less than, but this might need to be adjusted based on your language semantics
            right: Expr::Literal(Literal::Integer(0)) // Placeholder right side
        });

        SemanticRules::validate_condition(&condition, &type_check_closure)?;

        // Validate loop body instructions
        self.analyze_instructions(&for_loop.body)?;

        Ok(())
    }

    fn validate_read(&mut self, read_stmt: &ReadStmt) -> Result<(), String> {
        // For READ, the expression should be a variable
        match read_stmt {
            Expr::Variable(var) => {
                // Check if variable exists in symbol table
                let _ = self.symbol_table.lookup(var)
                    .ok_or_else(|| format!("Undefined variable in READ: {}", var))?;

                // Validate that the variable is not a constant
                let symbol = self.symbol_table.lookup(var).unwrap();
                if symbol.Is_Constant {
                    return Err(format!("Cannot read into constant variable: {}", var));
                }
                Ok(())
            },
            _ => Err("READ statement must have a variable as its argument".to_string())
        }
    }

    fn validate_write(&mut self, write_stmt: &WriteStmt) -> Result<(), String> {
        // Validate each element in the write statement
        for element in &write_stmt.elements {
            match element {
                WriteElement::String(_) => {
                    // String literals are always valid
                    continue;
                },
                WriteElement::Variable(var) => {
                    // Check if variable exists in symbol table
                    let _ = self.symbol_table.lookup(var)
                        .ok_or_else(|| format!("Undefined variable in WRITE: {}", var))?;
                }
            }
        }
        Ok(())
    }
    fn validate_write_expressions(&mut self, exprs: &Vec<Expr>) -> Result<(), String> {
        for expr in exprs {
            match expr {
                Expr::Literal(_) => {
                    // Literals are always valid
                    continue;
                },
                Expr::Variable(var) => {
                    // Check if variable exists in symbol table
                    let _ = self.symbol_table.lookup(var)
                        .ok_or_else(|| format!("Undefined variable in WRITE: {}", var))?;
                },
                Expr::BinaryOp(left, _, right) => {
                    // Validate that binary operations resolve to a valid type
                    let left_type = self.infer_expression_type(left)?;
                    let right_type = self.infer_expression_type(right)?;

                    // Check arithmetic compatibility of operands
                    TypeChecker::check_arithmetic_compatibility(&left_type, &right_type)?;
                }
            }
        }
        Ok(())
    }

    fn infer_expression_type(&self, expr: &Expr) -> Result<Types, String> {
        // Implement type inference for expressions
        match expr {
            Expr::Literal(lit) => Ok(match lit {
                Literal::Integer(_) => Types::Integer,
                Literal::Float(_) => Types::Float,
                Literal::Char(_) => Types::Char,
            }),
            Expr::Variable(var) => {
                let symbol = self.symbol_table.lookup(var)
                    .ok_or_else(|| format!("Undefined variable: {}", var))?;
                symbol.Type.clone().ok_or_else(|| format!("No type for variable: {}", var))
            },
            Expr::BinaryOp(left, _, right) => {
                let left_type = self.infer_expression_type(left)?;
                let right_type = self.infer_expression_type(right)?;
                TypeChecker::check_arithmetic_compatibility(&left_type, &right_type)
            },
        }
    }

    // Helper methods for type conversion and validation
    fn convert_to_type_value(&self, expr: &Expr) -> Result<TypeValue, String> {
        match expr {
            Expr::Literal(lit) => Ok(match lit {
                Literal::Integer(i) => TypeValue::Integer(*i),
                Literal::Float(f) => TypeValue::Float(*f),
                Literal::Char(c) => TypeValue::Char(*c),
            }),
            Expr::Variable(var) => {
                let symbol = self.symbol_table.lookup(var)
                    .ok_or_else(|| format!("Undefined variable: {}", var))?;
                symbol.Value.clone().ok_or_else(|| format!("No value for variable: {}", var))
            },
            Expr::BinaryOp(_, _, _) => Err("Cannot directly convert binary operation to TypeValue".to_string()),
        }
    }

    fn convert_array_to_type_value(&self, exprs: &Vec<Expr>) -> Result<TypeValue, String> {
        let values: Result<Vec<TypeValue>, String> = exprs
            .iter()
            .map(|expr| self.convert_to_type_value(expr))
            .collect();
        Ok(TypeValue::Array(values?))
    }

    fn evaluate_array_size(&self, size_expr: &Expr) -> Result<usize, String> {
        match size_expr {
            Expr::Literal(Literal::Integer(i)) => {
                if *i <= 0 {
                    Err("Array size must be positive".to_string())
                } else {
                    Ok(*i as usize)
                }
            },
            _ => Err("Array size must be a positive integer literal".to_string())
        }
    }
}