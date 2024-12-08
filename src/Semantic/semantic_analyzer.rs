use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{MutexGuard, TryLockResult};
use crate::Semantic::ts::*;
use crate::Semantic::type_checker::TypeChecker;
use crate::Semantic::semantic_rules::SemanticRules;
use crate::Parser::ast::{ArrayDecl, Assignment, BasicCond, BinOp, Condition, Declaration, Expr, IfStmt, Instruction, Program, ReadStmt, RelOp, Type, TypeValue, WriteElement, WriteStmt};
use crate::SymbolTable;

pub struct SemanticAnalyzer;
impl SemanticAnalyzer {
    pub fn new() -> Self {SemanticAnalyzer}

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
                SemanticRules::validate_variable_declaration(
                    name,
                    type_decl,
                    false,
                    None
                )
            },
            crate::Parser::ast::Variable::Initialized(name, expr) => {
                let value = self.parse_expr(expr)?;
                match SymbolTable.lock().unwrap().get_mut(name) {
                    Some(e) => e.Value = Some(value.clone()),
                    None => return Err(format!("Syntactic Error: Undeclared variable '{}'.", name)),
                };

                SemanticRules::validate_variable_declaration(
                    name,
                    type_decl,
                    false,
                    Some(&value)
                )
            }
        }
    }

    // Calculates the result of a binary arithmetic operation, crated it to reduce size of parse_expr function
    fn calculate_expr(&mut self, a0: TypeValue, op: &BinOp, a1: TypeValue) -> Result<TypeValue, String> {
        match (&a0, op, &a1) {
            (TypeValue::Integer(i0), BinOp::Add, TypeValue::Integer(i1)) => Ok(TypeValue::Integer(*i0 + *i1)),
            (TypeValue::Float(f0), BinOp::Add, TypeValue::Float(f1)) => Ok(TypeValue::Float(*f0 + *f1)),
            (TypeValue::Char(c0), BinOp::Add, TypeValue::Char(c1)) => Ok(TypeValue::Char((((*c0 as u8) + (*c1 as u8)) % 0x7F) as char)),

            (TypeValue::Integer(i0), BinOp::Sub, TypeValue::Integer(i1)) => Ok(TypeValue::Integer(*i0 - *i1)),
            (TypeValue::Float(f0), BinOp::Sub, TypeValue::Float(f1)) => Ok(TypeValue::Float(*f0 - *f1)),
            (TypeValue::Char(c0), BinOp::Sub, TypeValue::Char(c1)) => Ok(TypeValue::Char((((*c0 as u8) - (*c1 as u8)) % 0x7F) as char)),

            (TypeValue::Integer(i0), BinOp::Mul, TypeValue::Integer(i1)) => Ok(TypeValue::Integer(*i0 * *i1)),
            (TypeValue::Float(f0), BinOp::Mul, TypeValue::Float(f1)) => Ok(TypeValue::Float(*f0 * f1)),

            (TypeValue::Integer(i0), BinOp::Div, TypeValue::Integer(i1)) => {
                if *i1 == 0 {
                    return Err("Division by zero".to_string())
                }
                Ok(TypeValue::Integer(i0 / *i1))
            },
            (TypeValue::Float(f0), BinOp::Div, TypeValue::Float(f1)) => {
                if *f1 == 0f32 {
                    return Err("Division by zero".to_string())
                }
                Ok(TypeValue::Float(f0 / f1))
            },
            _ => Err(format!("Invalid Expression:\n\tLeft-Hand Operator: {:?}\n\tBinary Operator: {}\n\tRight-Hand Operator: {:?}", a0, op, a1))
        }
    }
    
    fn get_array_cell(&mut self, symbol: &Symbol, index: &Expr) -> Result<TypeValue, String> {
        match symbol.size {
            None => Err(format!("Index Assignment used with Non-Array variable '{}'.", symbol.Identifier)),
            Some(size) => match self.parse_expr(index)? {
                TypeValue::Integer(i) => {
                    if i < 0 {
                        return Err("Index of array can't be negative".to_string());
                    }
                    if i >= size {
                        return Err(format!("Index out of bounds, Array of size {}, Got {}.", size, i));
                    }
                    if i == 0 {
                        match symbol.Value.clone() {
                            Some(value) => Ok(value),
                            None => Err(format!("Variable '{}' used before being Assigned", symbol.Identifier))
                        }
                    }
                    else {
                        // Will be used when we actually implement addresses
                        match symbol.Type.clone().unwrap() { 
                            Types::Integer => Ok(TypeValue::Integer(0)),
                            Types::Float => Ok(TypeValue::Float(0f32)),
                            Types::Char => Ok(TypeValue::Char('!')),
                            _ => Err(format!("Invalid type for expression, symbol in error: {}", symbol))
                        }
                    }
                }
                _ => Err("Invalid Array size type.".to_string())?
            }
        }
    }

    fn parse_expr(&mut self, p0: &Expr) -> Result<TypeValue, String> {
        println!("{:?}", p0);
        // let symbol_table = SymbolTable.lock().unwrap();
        match  SymbolTable.try_lock() {
            Ok(_) => {println!(
                "Success"
            )}
            Err(e) => {println!("{:?}", e); return Err(format!("Syntactic Error: {}", e));}
        }
        match p0 {
            Expr::Literal(i) => match i {
                TypeValue::Integer(j) => Ok(TypeValue::Integer(*j)),
                TypeValue::Float(j) => Ok(TypeValue::Float(*j)),
                TypeValue::Char(j) => Ok(TypeValue::Char(*j)),
                TypeValue::Array(_) => Err("Cannot use array values in expression.".to_string()),
            },
            Expr::Variable(s) => match SymbolTable.lock().unwrap().get(s) {
                Some(t) => {
                    match &t.Value {
                        Some(e) => Ok(e.clone()),
                        None => Err(format!("Variable '{}' used before being Assigned", t.Identifier))
                    }
                },
                None => Err(format!("Undeclared Variable: {:?}", s)),
            },

            Expr::Array(s, i) => match SymbolTable.lock().unwrap().get(s) {
                Some(t) => self.get_array_cell(t, i),
                None => Err(format!("Undeclared Variable: {:?}", s)),
            },
            Expr::BinaryOp(expr0, binOp, expr1) => {
                let result1 = self.parse_expr(expr1)?;
                let result0 = self.parse_expr(expr0)?;
                self.calculate_expr(result0, binOp, result1)
            }
        }
    }
    fn validate_array_initialization(&mut self, type_decl: &Types, declared_size: &Expr, elements: &Vec<Expr>) -> Result<(), String> {
        let parsed_declared_size;
        match self.parse_expr(declared_size)? {
            TypeValue::Integer(i) => parsed_declared_size = i,
            _ => return Err ("Can't use a Non-Integer value as an array's size".to_string()),
        }
        if parsed_declared_size < elements.len() as i16 {
            return Err(format!("Array overflow detected\nExpected a maximum of '{}' elements, got assigned {} elements.", parsed_declared_size, elements.len()));
        }
        let first_element = &elements[0];

        for element in elements {
            let value_type = self.infer_expression_type(element)?;
            if value_type != *type_decl {
                return Err(format!("Invalid Type for array assignment\nExpected '{:?}', got '{:?}'", type_decl, value_type));
            }
        }
        Ok(())
    }
    
    fn validate_array_string_initialization(&mut self, type_decl: &Types, declared_size: &Expr, elements: &str) -> Result<(), String> {
        let parsed_declared_size;
        match self.parse_expr(declared_size)? {
            TypeValue::Integer(i) => parsed_declared_size = i,
            _ => return Err ("Can't use a Non-Integer value as an array's size".to_string()),
        }
        if parsed_declared_size < elements.len() as i16 {
            return Err(format!("Array overflow detected\nExpected a maximum of '{}' elements, got assigned {} elements.", parsed_declared_size, elements.len()));
        }
        Ok(())   
    }

    fn validate_array(&mut self, type_decl: &Types, arr: &ArrayDecl) -> Result<(), String> {
        match arr {
            ArrayDecl::Simple(name, size_expr) => {
                let size = self.evaluate_array_size(size_expr)?;
                match SymbolTable.lock().unwrap().get_mut(name) {
                    Some(symbol) => {
                        symbol.size = Some(size);
                    },
                    None => return Err(format!("Undeclared variable '{}'.", name)),
                };
                SemanticRules::validate_array_declaration(name, type_decl, size)
            },
            ArrayDecl::Initialized(name, size_expr, values) => {
                let size = self.evaluate_array_size(size_expr)?;
                // Additional type checking for initialized arrays
                self.validate_array_initialization(type_decl, size_expr, values)?;
                let value = self.parse_expr(&values[0])?;
                match SymbolTable.lock().unwrap().get_mut(name) {
                    Some(e) => {
                        e.size = Some(size);
                        e.Value = Some(value)
                    },
                    None => return Err(format!("Undeclared variable '{}'.", name)),
                };

                SemanticRules::validate_array_declaration(name, type_decl, size)
            },
            ArrayDecl::InitializedString(name, size_expr, value) => {
                let size = self.evaluate_array_size(size_expr)?;
                let value = &value[1..value.len() - 1];
                self.validate_array_string_initialization(type_decl, size_expr, value)?;
                let symbolTableValue;
                if value.chars().count() == 0{
                    symbolTableValue = Some(TypeValue::Char('\0'));
                }
                else { 
                    symbolTableValue = Some(TypeValue::Char(value.chars().nth(0).unwrap()));
                }
                match SymbolTable.lock().unwrap().get_mut(name) {
                    Some(e) => {
                        e.size = Some(size);
                        e.Value = symbolTableValue},
                    None => return Err(format!("Undeclared variable '{}'.", name)),
                };

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
        
        let value = self.parse_expr(&constant.expr)?;
        let Identifier = constant.var.clone();
        match SymbolTable.lock().unwrap().get_mut(&Identifier) {
            Some(e) => e.Value = Some(value.clone()),
            None => return Err(format!("Undeclared variable '{}'.", &Identifier)),
        };

        SemanticRules::validate_variable_declaration(
            &constant.var,
            type_decl,
            true,
            Some(&value)
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
        // match SymbolTable.lock().unwrap().get_mut(&assignment.var) {
        //     Some(symbol) => {
        //         // Check if variable is constant
        //         if symbol.Is_Constant.unwrap_or(false) {
        //             return Err(format!("Cannot reassign constant variable '{}'.", assignment.var));
        //         }
        //
        //         // Infer type of expression
        //         // let expr_type = self.infer_expression_type(&assignment.expr)?;
        //         let expr_value = self.parse_expr(&assignment.expr)?;
        //
        //         // Check type compatibility
        //         // TypeChecker::check_assignment_compatibility(
        //         //     symbol.Type.as_ref().ok_or_else(|| format!("No type for variable '{}'.", assignment.var))?,
        //         //     &expr_type
        //         // )?;
        //         match &assignment.index {
        //             Some(indexExpr) => {
        //                 let index = self.parse_expr(&indexExpr)?;
        //                 match index {
        //                     TypeValue::Integer(i) => {
        //                         if i < 0 {
        //                             return Err("Non-Positive Array size detected.".to_string());
        //                         }
        //                         match symbol.size {
        //                             Some(size) => {
        //                                 if i >= size {
        //                                     return Err(format!("Index out of bounds, Array of size {}, Got {}.", symbol.size.unwrap(), i));
        //                                 }
        //                             }
        //                             None => return Err(format!("Index Assignment used with Non-Array variable '{}'.", assignment.var)),
        //                         }
        //                     }
        //                     _ => Err("Invalid Array size type.".to_string())?
        //                 };
        //                 if index == TypeValue::Integer(0) {
        //                     symbol.Value = Some(self.parse_expr(&assignment.expr)?);
        //                 }
        //                 else {
        //                     // Will be used when we actually implement addresses
        //                 }
        //             }
        //             None => {
        //                 println!("Entered None index");
        //                 symbol.Value = Some(self.parse_expr(&assignment.expr)?);
        //             }
        //         }
        //
        //     }
        //     None => return Err(format!("Undeclared variable '{}'.", assignment.var)),
        // }
        let symbol_table = SymbolTable.lock().unwrap().get_mut(&assignment.var);
        match symbol_table {
            None => return Err(format!("Undeclared variable '{}'.", assignment.var)),
            Some(symbol) => {}
        }
        drop(symbol_table);
        let expr_value = self.parse_expr(&assignment.expr)?;
        let index;
        match &assignment.index {
            None => {index = -1}
            Some(e) => {index = self.parse_expr(&e)}
        }
        Ok(())
    }

    fn validate_if_statement(&mut self, if_stmt: &IfStmt) -> Result<(), String> {
        // Create a type-checking closure that can be passed to validate_condition
        let mut type_check_closure = |condition: &Condition| -> Result<Types, String> {
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
        SemanticRules::validate_condition(&if_stmt.condition, &mut type_check_closure)?;

        // Validate then block instructions
        self.analyze_instructions(&if_stmt.then_block)?;

        // Validate else block instructions if present
        if let Some(else_instructions) = &if_stmt.else_block {
            self.analyze_instructions(else_instructions)?;
        }

        Ok(())
    }

    // Add a helper method to infer condition type
    fn infer_condition_type(&mut self, condition: &Condition) -> Result<Types, String> {
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
        let init_type = self.infer_expression_type(&for_loop.init.expr)?;
        // Validate initialization expression type

        // Validate step type (should be same as initialization type)
        let step_type = self.infer_expression_type(&for_loop.step)?;
        if step_type != init_type {
            return Err("Step type must match initialization type".to_string());
        }

        // Create a type-checking closure for the condition
        let mut type_check_closure = |condition: &Condition| -> Result<Types, String> {
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
            left: Expr::Variable(for_loop.init.clone().var),
            operator: RelOp::Lt, // Default to less than, but this might need to be adjusted based on your language semantics
            right: for_loop.condition.clone() // Placeholder right side
        });

        SemanticRules::validate_condition(&condition, &mut type_check_closure)?;

        // Validate loop body instructions
        self.analyze_instructions(&for_loop.body)?;

        Ok(())
    }

    fn validate_read(&mut self, read_stmt: &ReadStmt) -> Result<(), String> {
        // For READ, the expression should be a variable
        let Identifier = &read_stmt.variable;
        match SymbolTable.lock().unwrap().get(Identifier){
            None => Err(format!("Undeclared variable '{}' inside READ instruction.", Identifier)),
            Some(x) => {
                if x.Is_Constant.unwrap() == false {
                    Ok(())
                }
                else {
                    Err(format!("Cannot READ into constant '{}'.", Identifier))
                }
            }
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
                    SymbolTable.lock().unwrap().get(var).ok_or_else(|| format!("Undefined variable '{}' in WRITE.", var))?;
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
                    SymbolTable.lock().unwrap().get(var).ok_or_else(|| format!("Undefined variable '{}' in WRITE.", var))?;
                },
                Expr::Array(var, expr) => {
                    return match SymbolTable.lock().unwrap().get(var) {
                        Some(symbol) => match self.get_array_cell(symbol, expr) {
                            Ok(t) => Ok(()),
                            Err(msg) => Err(msg),
                        },
                        None => return Err(format!("Undefined variable '{}' in WRITE.", var)),
                    }
                }
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

    fn infer_expression_type(&mut self, expr: &Expr) -> Result<Types, String> {
        // Implement type inference for expressions
        match expr {
            Expr::Literal(lit) => Ok(match lit {
                TypeValue::Integer(_) => Types::Integer,
                TypeValue::Float(_) => Types::Float,
                TypeValue::Char(_) => Types::Char,
                TypeValue::Array(_) => return Err("Cannot use array values in expression.".to_string()),
            }),
            Expr::Variable(var) => {
                match SymbolTable.lock().unwrap().get(var) {
                    Some(symbol) => { 
                        match symbol.Type.clone() {
                            Some(t) => Ok(t),
                            None => Err(format!("No type for variable '{}' in WRITE.", var))
                        }
                    },
                    None => Err(format!("Undefined variable '{}'.", var)),
                }
            },
            Expr::Array(var, expr) => {
                match SymbolTable.lock().unwrap().get(var) {
                    Some(symbol) => match symbol.Type.clone() {
                        Some(t) => Ok(t),
                        None => Err(format!("No type for variable '{}' in WRITE.", var))
                    },
                    None => return Err(format!("Undefined variable '{}' in WRITE.", var)),
                }
            },
            Expr::BinaryOp(left, _, right) => {
                let left_type = self.infer_expression_type(left)?;
                let right_type = self.infer_expression_type(right)?;
                TypeChecker::check_arithmetic_compatibility(&left_type, &right_type)
            },
        }
    }
    
    fn evaluate_array_size(&mut self, size_expr: &Expr) -> Result<i16, String> {
        let result = self.parse_expr(size_expr)?;
        match result {
            TypeValue::Integer(i) => {
                if i <= 0 { 
                    return Err("Non-Positive Array size detected.".to_string());
                }
                Ok(i)
            }
            _ => Err("Non-Integer Array size detected.".to_string()),
        }
    }
}