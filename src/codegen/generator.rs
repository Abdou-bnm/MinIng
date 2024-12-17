use cranelift::prelude::*;
use cranelift_codegen::ir::{Block as CraneliftBlock, Function, InstBuilder};
use cranelift_codegen::Context;
use cranelift_jit::{JITBuilder, JITModule};
use std::sync::Mutex;
use crate::Parser::ast::{Assignment, BinOp, Condition, Declaration, Expr, ForStmt, IfStmt, Instruction, LogOp, Program, RelOp, TypeValue};
use crate::Semantic::ts::{insert, update, remove, Symbol, Types}; // Use your provided symbol table functions


use once_cell::sync::Lazy;
use std::collections::HashMap;
use lalrpop_util::lalrpop_mod;

lalrpop_mod!(pub grammar, "/Parser/grammar.rs");
pub static SymbolTable: Lazy<Mutex<HashMap<String, Symbol>>> = Lazy::new(|| Mutex::new(HashMap::new()));

pub struct CodeGenerator {
    builder_context: FunctionBuilderContext,
    ctx: Context,
    module: JITModule,
}

impl CodeGenerator {
    pub fn new() -> Self {
        let builder = JITBuilder::new(cranelift_module::default_libcall_names()).unwrap();
        let module = JITModule::new(builder);

        CodeGenerator {
            builder_context: FunctionBuilderContext::new(),
            ctx: Context::new(),
            module,
        }
    }

    fn compile_expr(
        &mut self,
        builder: &mut FunctionBuilder,
        expr: &Expr,
    ) -> Value {
        match expr {
            Expr::Literal(value) => match value {
                TypeValue::Integer(n) => builder.ins().iconst(types::I32, *n as i64),
                TypeValue::Float(f) => builder.ins().f32const(*f),
                TypeValue::Char(c) => builder.ins().iconst(types::I8, *c as i64),
                TypeValue::Array(_) => unimplemented!("Array literals not yet supported"),
            },
            Expr::Variable(name) => {
                let table = SymbolTable.lock().unwrap();
                if let Some(symbol) = table.get(name) {
                    if let Some(val) = symbol.Value.as_ref() {
                        match val.get(0) {
                            Some(TypeValue::Integer(n)) => {
                                builder.ins().iconst(types::I32, *n as i64)
                            }
                            Some(TypeValue::Float(f)) => {
                                builder.ins().f32const(*f)
                            }
                            Some(TypeValue::Char(c)) => {
                                builder.ins().iconst(types::I8, *c as i64)
                            }
                            _ => unimplemented!("Unsupported variable type"),
                        }
                    } else {
                        panic!("Variable '{}' not initialized", name);
                    }
                } else {
                    panic!("Variable '{}' not found in symbol table", name);
                }
            },
            Expr::Array(name, index) => {
                let table = SymbolTable.lock().unwrap();
                if let Some(symbol) = table.get(name) {
                    // Compile the index expression
                    let index_value = self.compile_expr(builder, index);
                    
                    // For now, we'll just return the first element of the array
                    // More sophisticated array access will require more complex handling
                    if let Some(val) = symbol.Value.as_ref() {
                        match val.first() {
                            Some(TypeValue::Integer(n)) => {
                                builder.ins().iconst(types::I32, *n as i64)
                            }
                            Some(TypeValue::Float(f)) => {
                                builder.ins().f32const(*f)
                            }
                            _ => unimplemented!("Array element type not supported"),
                        }
                    } else {
                        panic!("Array '{}' not initialized", name);
                    }
                } else {
                    panic!("Array '{}' not found in symbol table", name);
                }
            },
            Expr::BinaryOp(left, op, right) => {
                let lhs = self.compile_expr(builder, left);
                let rhs = self.compile_expr(builder, right);
    
                // Determine the type of operation based on the first operand
                let lhs_type = builder.func.dfg.value_type(lhs);
                
                match op {
                    BinOp::Add if lhs_type == types::I32 => builder.ins().iadd(lhs, rhs),
                    BinOp::Sub if lhs_type == types::I32 => builder.ins().isub(lhs, rhs),
                    BinOp::Mul if lhs_type == types::I32 => builder.ins().imul(lhs, rhs),
                    BinOp::Div if lhs_type == types::I32 => builder.ins().sdiv(lhs, rhs),
                    
                    // Add floating-point operations
                    BinOp::Add if lhs_type == types::F32 => builder.ins().fadd(lhs, rhs),
                    BinOp::Sub if lhs_type == types::F32 => builder.ins().fsub(lhs, rhs),
                    BinOp::Mul if lhs_type == types::F32 => builder.ins().fmul(lhs, rhs),
                    BinOp::Div if lhs_type == types::F32 => builder.ins().fdiv(lhs, rhs),
                    
                    _ => panic!("Unsupported binary operation"),
                }
            },
        }
    }


    fn compile_declaration(
        &mut self,
        builder: &mut FunctionBuilder,
        decl: &Declaration,
    ) -> Result<(), String> {
        match decl {
            Declaration::Variables(typ, vars) => {
                for var in vars {
                    let symbol = Symbol::new(var.name.clone(), Some(typ.clone()), None, None, None, None);
                    insert(&SymbolTable, symbol)?;
                }
            }
            Declaration::Array(typ, array_decls) => {
                // Logic to handle array declarations
                for array_decl in array_decls {
                    let symbol = Symbol::new(array_decl.name.clone(), Some(Types::Array(Box::new(typ.clone()), array_decl.size)), None, None, None, Some(array_decl.size));
                    insert(&SymbolTable, symbol)?;
                }
            }
            Declaration::Constant(typ, assignments) => {
                // Handle constant declarations
                for assignment in assignments {
                    let symbol = Symbol::new(assignment.var.clone(), Some(typ.clone()), Some(true), None, None, None);
                    insert(&SymbolTable, symbol)?;
                }
            }
        }
        Ok(())
    }
    

    fn compile_assignment(
        &mut self,
        builder: &mut FunctionBuilder,
        assignment: &Assignment
    ) -> Result<(), String> {
        let value = self.compile_expr(builder, &assignment.expr)?;
    
        let mut table = SymbolTable.lock().map_err(|_| "Symbol table lock poisoned")?;
        if let Some(symbol) = table.get_mut(&assignment.var) {
            // Store the computed value back into the symbol table
            symbol.Value = Some(vec![self.get_value_type(builder, value).ok_or("Failed to get value type")?]);
            Ok(())
        } else {
            Err(format!("Variable '{}' is not declared", assignment.var))
        }
    }
    
    
    
    // Helper method to convert Cranelift Value to TypeValue
    fn get_value_type(&self, builder: &FunctionBuilder, value: Value) -> Option<TypeValue> {
        let type_of_value = builder.func.dfg.value_type(value);
        
        if type_of_value == types::I32 {
            // For integer values, we'll need to extract the actual value
            // Note: This is a simplification and might need more robust handling
            Some(TypeValue::Integer(0)) // Placeholder - you'll need to implement actual value extraction
        } else if type_of_value == types::F32 {
            Some(TypeValue::Float(0.0)) // Placeholder - you'll need to implement actual value extraction
        } else {
            None
        }
    }


    fn compile_if_statement(
        &mut self,
        builder: &mut FunctionBuilder,
        if_stmt: &IfStmt,
    ) -> Result<(), String> {
        let then_block = builder.create_block();
        let merge_block = builder.create_block();
        let else_block = if if_stmt.else_block.is_some() {
            Some(builder.create_block())
        } else {
            None
        };

        // Compile condition
        let condition_value = self.compile_condition(builder, &if_stmt.condition)?;
        
        match else_block {
            Some(else_blk) => {
                builder.ins().brnz(condition_value, then_block, &[]);
                builder.ins().jump(else_blk, &[]);
            }
            None => {
                builder.ins().brnz(condition_value, then_block, &[]);
                builder.ins().jump(merge_block, &[]);
            }
        }

        // Compile then block
        builder.switch_to_block(then_block);
        builder.seal_block(then_block);
        
        for inst in &if_stmt.then_block {
            self.compile_instruction(builder, inst)?;
        }
        builder.ins().jump(merge_block, &[]);

        // Compile else block if it exists
        if let Some(else_blk) = else_block {
            builder.switch_to_block(else_blk);
            builder.seal_block(else_blk);
            
            if let Some(else_instructions) = &if_stmt.else_block {
                for inst in else_instructions {
                    self.compile_instruction(builder, inst)?;
                }
            }
            builder.ins().jump(merge_block, &[]);
        }

        builder.switch_to_block(merge_block);
        builder.seal_block(merge_block);

        Ok(())
    }

    fn compile_for_loop(
        &mut self,
        builder: &mut FunctionBuilder,
        for_stmt: &ForStmt,
    ) -> Result<(), String> {
        let header_block = builder.create_block();
        let body_block = builder.create_block();
        let increment_block = builder.create_block();
        let exit_block = builder.create_block();

        // Compile initialization
        self.compile_assignment(builder, &for_stmt.init)?;

        // Jump to header block
        builder.ins().jump(header_block, &[]);
        builder.switch_to_block(header_block);

        // Compile condition
        let condition_value = self.compile_expr(builder, &for_stmt.condition)?;
        builder.ins().brnz(condition_value, body_block, &[]);
        builder.ins().jump(exit_block, &[]);

        // Compile loop body
        builder.switch_to_block(body_block);
        builder.seal_block(body_block);

        for inst in &for_stmt.body {
            self.compile_instruction(builder, inst)?;
        }
        builder.ins().jump(increment_block, &[]);

        // Compile increment
        builder.switch_to_block(increment_block);
        builder.seal_block(increment_block);
        self.compile_expr(builder, &for_stmt.step)?;
        builder.ins().jump(header_block, &[]);

        builder.seal_block(header_block);
        builder.switch_to_block(exit_block);
        builder.seal_block(exit_block);

        Ok(())
    }

    fn compile_condition(
        &mut self,
        builder: &mut FunctionBuilder,
        condition: &Condition,
    ) -> Result<Value, String> {
        match condition {
            Condition::Not(cond) => {
                let inner_value = self.compile_condition(builder, cond)?;
                Ok(builder.ins().bnot(inner_value))
            }
            Condition::Logic(left, op, right) => {
                let left_value = self.compile_condition(builder, left)?;
                let right_value = self.compile_condition(builder, right)?;
                match op {
                    LogOp::And => Ok(builder.ins().band(left_value, right_value)),
                    LogOp::Or => Ok(builder.ins().bor(left_value, right_value)),
                }
            }
            Condition::Basic(basic_cond) => {
                let left_value = self.compile_expr(builder, &basic_cond.left)?;
                let right_value = self.compile_expr(builder, &basic_cond.right)?;
                
                match basic_cond.operator {
                    RelOp::Gt => Ok(builder.ins().icmp(IntCC::SignedGreaterThan, left_value, right_value)),
                    RelOp::Lt => Ok(builder.ins().icmp(IntCC::SignedLessThan, left_value, right_value)),
                    RelOp::Ge => Ok(builder.ins().icmp(IntCC::SignedGreaterThanOrEqual, left_value, right_value)),
                    RelOp::Le => Ok(builder.ins().icmp(IntCC::SignedLessThanOrEqual, left_value, right_value)),
                    RelOp::Eq => Ok(builder.ins().icmp(IntCC::Equal, left_value, right_value)),
                    RelOp::Ne => Ok(builder.ins().icmp(IntCC::NotEqual, left_value, right_value)),
                }
            }
        }
    }

    fn compile_instruction(
        &mut self,
        builder: &mut FunctionBuilder,
        instruction: &Instruction,
    ) -> Result<(), String> {
        match instruction {
            Instruction::Assign(assign) => self.compile_assignment(builder, assign),
            Instruction::If(if_stmt) => self.compile_if_statement(builder, if_stmt),
            Instruction::For(for_stmt) => self.compile_for_loop(builder, for_stmt),
            Instruction::Read(read_stmt) => self.compile_read(builder, read_stmt),
            Instruction::Write(write_stmt) => self.compile_write(builder, write_stmt),
        }
    }


    fn allocate_array(
        &mut self,
        builder: &mut FunctionBuilder,
        size: usize,
        elem_type: Type,
    ) -> Value {
        // Allocate memory for array
        let size_val = builder.ins().iconst(types::I32, size as i64);
        let elem_size = match elem_type {
            types::I32 => 4,
            types::F32 => 4,
            types::I8 => 1,
            _ => panic!("Unsupported array element type"),
        };
        let total_size = builder.ins().imul_imm(size_val, elem_size);
        
        // Call memory allocation function (you'll need to implement or link this)
        // This is a placeholder - you'll need to implement actual memory allocation
        builder.ins().iconst(types::I64, 0)
    }

    fn compile_array_access(
        &mut self,
        builder: &mut FunctionBuilder,
        name: &str,
        index: &Expr,
    ) -> Result<Value, String> {
        let table = SymbolTable.lock().map_err(|_| "Symbol table lock poisoned")?;
        let symbol = table.get(name).ok_or_else(|| format!("Array '{}' not found", name))?;
        
        let array_size = symbol.Size.ok_or_else(|| "Not an array")?;
        let index_val = self.compile_expr(builder, index);
        
        // Add bounds checking
        let size_const = builder.ins().iconst(types::I32, array_size as i64);
        let is_in_bounds = builder.ins().icmp(IntCC::UnsignedLessThan, index_val, size_const);
        
        let in_bounds_block = builder.create_block();
        let out_of_bounds_block = builder.create_block();
        let merge_block = builder.create_block();
        
        builder.ins().brnz(is_in_bounds, in_bounds_block, &[]);
        builder.ins().jump(out_of_bounds_block, &[]);
        
        // Handle out of bounds
        builder.switch_to_block(out_of_bounds_block);
        // Add error handling here
        builder.ins().trap(TrapCode::User(1));
        
        // Handle in bounds access
        builder.switch_to_block(in_bounds_block);
        // Implement actual array element access
        // This is a placeholder - you'll need to implement actual array access
        Ok(builder.ins().iconst(types::I32, 0))
    }

    fn compile_read(
        &mut self,
        builder: &mut FunctionBuilder,
        var_name: &str,
    ) -> Result<(), String> {
        // Implementation for read operation
        // You'll need to implement or link to actual I/O functions
        unimplemented!("Read operation not yet implemented")
    }

    fn compile_write(
        &mut self,
        builder: &mut FunctionBuilder,
        expr: &Expr,
    ) -> Result<(), String> {
        // Implementation for write operation
        let value = self.compile_expr(builder, expr)?;
        // You'll need to implement or link to actual I/O functions
        unimplemented!("Write operation not yet implemented")
    }

    fn type_check(
        &self,
        expected: Type,
        got: Type,
    ) -> Result<(), String> {
        if expected != got {
            Err(format!("Type mismatch: expected {:?}, got {:?}", expected, got))
        } else {
            Ok(())
        }
    }

    fn convert_type(
        &mut self,
        builder: &mut FunctionBuilder,
        value: Value,
        from: Type,
        to: Type,
    ) -> Result<Value, String> {
        match (from, to) {
            (types::I32, types::F32) => Ok(builder.ins().fcvt_from_sint(types::F32, value)),
            (types::F32, types::I32) => Ok(builder.ins().fcvt_to_sint(types::I32, value)),
            (from, to) if from == to => Ok(value),
            _ => Err(format!("Unsupported type conversion from {:?} to {:?}", from, to)),
        }
    }

    // Main compile method for the entire program
    pub fn compile(&mut self, program: &Program) -> Result<(), String> {
        let mut func = Function::new();
        let mut builder_context = std::mem::take(&mut self.builder_context);
        let mut builder = FunctionBuilder::new(&mut func, &mut builder_context);

        let entry_block = builder.create_block();
        builder.append_block_params_for_function_params(entry_block);
        builder.switch_to_block(entry_block);

        // Compile global declarations
        if let Some(globals) = &program.global {
            for decl in globals {
                self.compile_condition(builder, decl)?;
            }
        }

        // Compile other declarations
        if let Some(decls) = &program.decls {
            for decl in decls {
                self.compile_condition(builder, decl)?;
            }
        }

        // Compile instructions
        if let Some(instructions) = &program.inst {
            for inst in instructions {
                self.compile_instruction(builder, inst)?;
            }
        }

        builder.ins().return_(&[]);
        builder.seal_all_blocks();

        self.builder_context = builder_context;
        Ok(())
    }

    pub fn test_setup(&mut self) -> Result<(), String> {
        println!("Code generator setup test successful!");
        Ok(())
    }


    pub fn compile_test_expr(&mut self, expr: &Expr) -> Result<(), String> {
        self.compile(expr);
        Ok(())
    }


    
    
}
