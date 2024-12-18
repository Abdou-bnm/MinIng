use std::collections::HashMap;
use std::convert::identity;
use std::fmt::Debug;
use std::sync::{MutexGuard, TryLockResult};
use logos::Source;
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
            self.analyze_instructions(instructions, false)?;
        }

        Ok(())
    }

    fn analyze_declarations(&mut self, declarations: &Vec<Declaration>) -> Result<(), String> {
        for decl in declarations {
            match decl {
                Declaration::Variable(type_decl, vars) => {
                    for var in vars {
                        match type_decl {
                            Type::Integer => self.validate_variable(&Types::Integer, var)?,
                            Type::Float => self.validate_variable(&Types::Float, var)?,
                            Type::Char => self.validate_variable(&Types::Char, var)?,
                        }
                    }
                },
                Declaration::ADEC(type_decl, arrays) => {
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
                    name.clone(),
                    type_decl,
                    false,
                    None
                )
            },
            crate::Parser::ast::Variable::Initialized(name, expr) => {
                let value = self.parse_expr(expr)?;
                
                match SymbolTable.lock().unwrap().get_mut(name.0.as_str()) {
                    Some(e) => e.Value[0] = Some(value.clone()),
                    None => return Err(format!("Syntactic Error: Undeclared variable '{}'. ({}:{})", name.0.as_str(), name.1.0, name.1.1)),
                };

                SemanticRules::validate_variable_declaration(
                    name.clone(),
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
            (TypeValue::Integer(i0), BinOp::Add(_, _), TypeValue::Integer(i1)) => Ok(TypeValue::Integer((i0.0 + i1.0, (i0.1.0, i0.1.1)))),
            (TypeValue::Float(f0), BinOp::Add(_, _), TypeValue::Float(f1)) => Ok(TypeValue::Float((f0.0 + f1.0, (f0.1.0, f0.1.1)))),
            (TypeValue::Char(c0), BinOp::Add(_, _), TypeValue::Char(c1)) => Ok(TypeValue::Char(((((c0.0 as u8) + (c1.0 as u8)) % 0x7F) as char, (c0.1.0, c0.1.1)))),

            (TypeValue::Integer(i0), BinOp::Sub(_, _), TypeValue::Integer(i1)) => Ok(TypeValue::Integer((i0.0 - i1.0, (i0.1.0, i0.1.1)))),
            (TypeValue::Float(f0), BinOp::Sub(_, _), TypeValue::Float(f1)) => Ok(TypeValue::Float((f0.0 - f1.0, (f0.1.0, f0.1.1)))),
            (TypeValue::Char(c0), BinOp::Sub(_, _), TypeValue::Char(c1)) => Ok(TypeValue::Char(((((c0.0 as u8) - (c1.0 as u8)) % 0x7F) as char, (c0.1.0, c0.1.1)))),

            (TypeValue::Integer(i0), BinOp::Mul(_, _), TypeValue::Integer(i1)) => Ok(TypeValue::Integer((i0.0 * i1.0, (i0.1.0, i0.1.1)))),
            (TypeValue::Float(f0), BinOp::Mul(_, _), TypeValue::Float(f1)) => Ok(TypeValue::Float((f0.0 + f1.0, (f0.1.0, f0.1.1)))),

            (TypeValue::Integer(i0), BinOp::Div(_, _), TypeValue::Integer(i1)) => {
                if i1.0 == 0 {
                    return Err(format!("Division by zero at ({}:{})", i0.1.0, i0.1.1));
                }
                Ok(TypeValue::Integer((i0.0 / i1.0, (i0.1.0, i0.1.1))))
            },
            (TypeValue::Float(f0), BinOp::Div(_, _), TypeValue::Float(f1)) => {
                if f1.0 == 0f32 {
                    return Err(format!("Division by zero at ({}:{})", f0.1.0, f0.1.1));
                }
                Ok(TypeValue::Float((f0.0 / f1.0, (f0.1.0, f0.1.1))))
            },
            _ => Err(format!("Invalid Expression:\n\tLeft-Hand Operator: {:?}\n\tBinary Operator: {}\n\tRight-Hand Operator: {:?}", a0, op, a1))
        }
    }
    
    fn get_array_cell(&mut self, symbol: &Symbol, index: &Expr) -> Result<TypeValue, String> {
        match symbol.size {
            None => Err(format!("Index Assignment used with Non-Array variable '{}'.", symbol.Identifier)),
            Some(size) => match self.parse_expr(index)? {
                TypeValue::Integer(i) => {
                    if i.0 < 0 {
                        return Err(format!("Negative Index Array at ({}:{})", i.1.0, i.1.1));
                    }
                    if i.0>= size {
                        return Err(format!("Index out of bounds, Array of size {}, Got {} at ({}:{}).", size, i.0, i.1.0, i.1.1));
                    }
                    match symbol.Value[i.0 as usize].clone() {
                        None => Err(format!("Cell '{}[{}]' used before being Assigned at ({}:{})", symbol.Identifier, i.0, i.1.0, i.1.1)),
                        Some(val) => Ok(val)
                    }
                }
                _ => Err("Invalid Array size type.".to_string())?
            }
        }
    }

    fn parse_expr(&mut self, p0: &Expr) -> Result<TypeValue, String> {
        match SymbolTable.try_lock() {
            Ok(_) => {}
            Err(e) => return Err(format!("Syntactic Error: {}", e))
        }
        match p0 {
            Expr::Literal(i) => match i {
                TypeValue::Integer(j) => Ok(TypeValue::Integer(*j)),
                TypeValue::Float(j) => Ok(TypeValue::Float(*j)),
                TypeValue::Char(j) => Ok(TypeValue::Char(*j)),
                TypeValue::Array(_) => Err("Cannot use array values in expression.".to_string()),
            },
            Expr::Variable(s) => match SymbolTable.lock().unwrap().get(s.0.as_str()) {
                Some(t) => {
                    match &t.Value[0].clone() {
                        Some(e) => Ok(e.clone()),
                        None => Err(format!("Variable '{}' used before being Assigned", t.Identifier))
                    }
                },
                None => Err(format!("Undeclared Variable: {:?}", s)),
            },

            Expr::SUBS(s, i) => {
                let symbol_table = SymbolTable.lock().unwrap();
                let symbol = symbol_table.get(s.0.as_str()).ok_or_else(|| format!("Undeclared variable: {:?}", s))?;
                let copySymbol = symbol.clone();
                drop(symbol_table);
                self.get_array_cell(&copySymbol, i)
            },
            Expr::BinaryOp(expr0, binOp, expr1) => {
                let result0 = self.parse_expr(expr0)?;
                let result1 = self.parse_expr(expr1)?;
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
        if parsed_declared_size.0 < elements.len() as i16 {
            return Err(format!("Array overflow detected\nExpected a maximum of '{}' elements, got assigned {} elements.", parsed_declared_size.0, elements.len()));
        }

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
        if parsed_declared_size.0 < elements.len() as i16 {
            return Err(format!("Array overflow detected\nExpected a maximum of '{}' elements, got assigned {} elements.", parsed_declared_size.0, elements.len()));
        }
        Ok(())   
    }

    fn validate_array(&mut self, type_decl: &Types, arr: &ArrayDecl) -> Result<(), String> {
        // println!("{:?}", arr);
        match arr {
            ArrayDecl::Simple(name, size_expr) => {
                let size = self.evaluate_array_size(size_expr)?;
                match SymbolTable.lock().unwrap().get_mut(name.0.as_str()) {
                    Some(symbol) => {
                        symbol.size = Some(size);
                        for i in 0..size {
                            symbol.Value.push(None);
                        }
                    },
                    None => return Err(format!("Undeclared variable '{}' at ({}:{}).", name.0, name.1.0, name.1.1)),
                };
                SemanticRules::validate_array_declaration(name.clone(), type_decl, size)
            },
            ArrayDecl::Initialized(name, size_expr, values) => {
                let size = self.evaluate_array_size(size_expr)?;
                // Additional type checking for initialized arrays
                self.validate_array_initialization(type_decl, size_expr, values)?;
                let symbol_table = SymbolTable.lock().unwrap();
                let symbol = symbol_table
                    .get(name.0.as_str())
                    .ok_or_else(|| format!("Undeclared variable '{}' at ({}:{}).", name.0, name.1.0, name.1.1))?;

                drop(symbol_table);
                let mut vector: Vec<Option<TypeValue>> = vec!();
                for value in values {
                    let parsedValue = self.parse_expr(value)?;
                    vector.push(Some(parsedValue));
                }
                
                let mut index = 0;
                while vector.len() < size as usize {
                    vector.push(vector[index].clone());
                    index += 1;
                }

                let mut symbol_table = SymbolTable.lock().unwrap();
                let symbol = symbol_table.get_mut(name.0.as_str()).unwrap();
                symbol.Value = vector;
                symbol.size = Some(size);

                SemanticRules::validate_array_declaration(name.clone(), type_decl, size)
            },
            ArrayDecl::InitializedString(name, size_expr, value) => {
                let size = self.evaluate_array_size(size_expr)?;
                let value = &value.0[1..value.0.len() - 1];
                self.validate_array_string_initialization(type_decl, size_expr, value)?;
                let mut vector;
                if value.chars().count() == 0 {
                    vector = vec!(Some(TypeValue::Char(('\0', (0, 0)))));
                }
                else {
                    vector = value
                        .chars()
                        .map(|ch| Some(TypeValue::Char((ch, (0, 0)))))
                        .collect();
                }
                let mut index = 0;
                while vector.len() < size as usize {
                    vector.push(vector[index].clone());
                    index += 1;
                }
                
                match SymbolTable.lock().unwrap().get_mut(name.0.as_str()) {
                    Some(e) => {
                        e.size = Some(size);
                        e.Value = vector
                    },
                    None => return Err(format!("Undeclared variable '{}' at ({}:{}).", name.0, name.1.0, name.1.1)),
                };

                SemanticRules::validate_array_declaration(name.clone(), type_decl, size)
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
        match SymbolTable.lock().unwrap().get_mut(&Identifier.0) {
            Some(e) => {
                e.Value[0] = Some(value.clone());
            }
            None => return Err(format!("Undeclared variable '{}' at ({}:{}).", Identifier.0, Identifier.1.0, Identifier.1.1)),
        };
        SemanticRules::validate_variable_declaration(
            constant.var.clone(),
            type_decl,
            true,
            Some(&value)
        )
    }

    fn analyze_instructions(&mut self, instructions: &Vec<Instruction>, runt_act : bool) -> Result<(), String> {
        for instruction in instructions {
            match instruction {
                Instruction::Assign(assignment) => self.validate_assignment(assignment,runt_act)?,
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
    fn validate_assignment(&mut self, assignment: &Assignment, runt_act : bool) -> Result<(), String> {
        // Check if variable exists in symbol table
        let mut symbol_table = SymbolTable.lock().unwrap();
        let symbol = symbol_table
            .get(&assignment.var.0)
            .ok_or_else(|| format!("Undeclared variable '{}' at ({}:{}).", assignment.var.0, assignment.var.1.0, assignment.var.1.1))?;

        drop(symbol_table);

        let expr_value = self.parse_expr(&assignment.expr)?;

        symbol_table = SymbolTable.lock().unwrap();
        let symbol = symbol_table.get(&assignment.var.0).unwrap().clone();
        let symbolType = symbol.Type.clone().unwrap();
        drop(symbol_table);
        
        match (symbolType.clone(), expr_value.clone()) {
            (Types::Integer, TypeValue::Integer(t)) => {},
            (Types::Char, TypeValue::Char(t2)) => {},
            (Types::Float, TypeValue::Float(t3)) => {},
            _ => return Err(format!("Cannot insert value of type {:?} into an array of type {:?} at ({}:{}).", expr_value.clone(), symbolType.clone(), assignment.var.1.0, assignment.var.1.1)),
        }
        
        let mut index: i16 = 0;
        match &assignment.index {
            None => index = 0,
            Some(e) => {
                let expressionResult = &self.parse_expr(&e)?; 
                match expressionResult {
                    TypeValue::Integer(i) => {
                        let size = symbol.size
                            .clone()
                            .ok_or_else(|| format!("Index Assignment used with Non-Array variable '{}'.", symbol.Identifier))?;
                        if i.clone().0 >= size {
                            return Err(format!("Index out of bounds, Array of size {}, Got {}.", size, i.0));
                        }
                        index = i.clone().0;
                    }
                    _ => Err("Invalid Array size type.".to_string())?
                }
            }
        }
        if !runt_act {
            let mut symbol_table = SymbolTable.lock().unwrap();
            let symbol = symbol_table.get_mut(&assignment.var.0).unwrap();
            symbol.Value[index as usize] = Some(expr_value);
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
        self.analyze_instructions(&if_stmt.then_block,true)?;

        // Validate else block instructions if present
        if let Some(else_instructions) = &if_stmt.else_block {
            self.analyze_instructions(else_instructions, true)?;
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
            operator: RelOp::Lt(for_loop.init.var.1.0, for_loop.init.var.1.1), // Default to less than, but this might need to be adjusted based on your language semantics
            right: for_loop.condition.clone() // Placeholder right side
        });

        SemanticRules::validate_condition(&condition, &mut type_check_closure)?;
        self.validate_assignment(&for_loop.init,false)?;
        // Validate loop body instructions
        self.analyze_instructions(&for_loop.body, true)?;

        Ok(())
    }

    fn validate_read(&mut self, read_stmt: &ReadStmt) -> Result<(), String> {
        // For READ, the expression should be a variable
        let Identifier = &read_stmt.variable;

        let mut index = 0;
        let mut symbol_table = SymbolTable.lock().unwrap();
        let symbol = symbol_table.get_mut(Identifier.0.as_str()).ok_or_else(|| format!("Undefined variable '{}' in READ at ({}:{}).", Identifier.0, Identifier.1.0, Identifier.1.1))?.clone();
        drop(symbol_table);

        match read_stmt.index.clone() {
            None => index = 0,
            Some(e) => {
                let exprResult = self.evaluate_array_size(&e)?;
                if exprResult >= symbol.size.ok_or_else(|| format!("Index Assignment used with Non-Array variable '{}'.", symbol.Identifier))? {
                    return Err(format!("Index Assignment is out of bounds: {}", index));
                }
                index = exprResult;
                vec![None::<Option<TypeValue>>];
            }
        }
        
        // Need to implement the index into the program later, just need to figure out the problem with nabil
        match SymbolTable.lock().unwrap().get_mut(Identifier.0.as_str()) {
            None => return Err(format!("Undeclared variable '{}' inside READ instruction at ({}:{}).", Identifier.0, Identifier.1.0, Identifier.1.1)),
            Some(symbol) => {
                let symbolType = symbol
                    .Type.clone()
                    .ok_or_else(|| format!("Cannot READ into constant '{}' at ({}:{}).", Identifier.0, Identifier.1.0, Identifier.1.1))?;
                
                match symbolType {
                    Types::Integer => symbol.Value[index as usize] = Some(TypeValue::Integer((0, (0,0)))),
                    Types::Float => symbol.Value[index as usize] = Some(TypeValue::Float((0.0, (0, 0)))),
                    Types::Char => symbol.Value[index as usize] = Some(TypeValue::Char(('\0', (0, 0)))),
                    Types::Array(_, _) => {}
                }
            }
        };
                // todo!();
        Ok(())
        
    }

    fn validate_write(&mut self, write_stmt: &WriteStmt) -> Result<(), String> {
        // Validate each element in the write statement
        for element in &write_stmt.elements {
            match element {
                WriteElement::String(_) => {
                    // String literals are always valid
                    continue;
                },
                WriteElement::Variable(var, expr) => {
                    // Check if variable exists in symbol table
                    SymbolTable.lock().unwrap().get(var.0.as_str()).ok_or_else(|| format!("Undefined variable '{}' in WRITE at ({}:{}).", var.0, var.1.0, var.1.1))?;
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
                    SymbolTable.lock().unwrap().get(var.0.as_str()).ok_or_else(|| format!("Undefined variable '{}' in WRITE at ({}:{}).", var.0, var.1.0, var.1.1))?;
                },
                Expr::SUBS(var, expr) => {
                    return match SymbolTable.lock().unwrap().get(var.0.as_str()) {
                        Some(symbol) => match self.get_array_cell(symbol, expr) {
                            Ok(t) => Ok(()),
                            Err(msg) => Err(msg),
                        },
                        None => return Err(format!("Undefined variable '{}' in WRITE at ({}:{}).", var.0, var.1.0, var.1.1)),
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
                match SymbolTable.lock().unwrap().get(var.0.as_str()) {
                    Some(symbol) => { 
                        match symbol.Type.clone() {
                            Some(t) => Ok(t),
                            None => Err(format!("No type for variable '{}' in WRITE at ({}:{}).", var.0, var.1.0, var.1.1))
                        }
                    },
                    None => Err(format!("Undefined variable '{}' at ({}:{}).", var.0, var.1.0, var.1.1)),
                }
            },
            Expr::SUBS(var, expr) => {
                match SymbolTable.lock().unwrap().get(var.0.as_str()) {
                    Some(symbol) => match symbol.Type.clone() {
                        Some(t) => Ok(t),
                        None => Err(format!("No type for variable '{}' in WRITE at ({}:{}).", var.0, var.1.0, var.1.1))
                    },
                    None => return Err(format!("Undefined variable '{}' in WRITE at ({}:{}).", var.0, var.1.0, var.1.1)),
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
                if i.0 <= 0 { 
                    return Err(format!("Non-Positive Array size detected at ({}:{}).", i.1.0, i.1.1));
                }
                Ok(i.0)
            }
            _ => Err("Non-Integer Array size detected.".to_string()),
        }
    }
}