use cranelift::prelude::*;
use cranelift_codegen::ir::{Function, GlobalValue, InstBuilder};
use cranelift_codegen::Context;
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{default_libcall_names, DataContext, Linkage, Module, ModuleError};
use std::sync::Mutex;
use crate::Parser::ast::{Assignment, BinOp, Condition, Expr, ForStmt, IfStmt, Instruction, LogOp, Loop, ReadStmt, RelOp, Statement, TypeValue, WriteElement, WriteStmt};
use crate::Semantic::ts::{insert, update, remove, Symbol, Types}; // Use your provided symbol table functions


use once_cell::sync::Lazy;
use std::collections::HashMap;
use lalrpop_util::lalrpop_mod;

lalrpop_mod!(pub grammar, "/Parser/grammar.rs");
pub static SymbolTable: Lazy<Mutex<HashMap<String, Symbol>>> = Lazy::new(|| Mutex::new(HashMap::new()));

pub struct CodeGenerator {
    pub ctx: Context,
    pub builder_context: FunctionBuilderContext,
    module: JITModule,
    variables: HashMap<String, Variable>,
    variable_count: usize,  // Changed from next_variable_index
    string_counter: usize,
    print_signature: Signature,
    print_string_signature: Signature,
    read_signature: Signature,
}

impl CodeGenerator {
    pub fn new() -> Self {
        let builder = JITBuilder::new(default_libcall_names()).unwrap();
        let module = JITModule::new(builder);
        let ctx = Context::new();
        let builder_context = FunctionBuilderContext::new();

        // Initialize signatures for external functions
        let mut sig_read = module.make_signature();
        sig_read.returns.push(AbiParam::new(types::I32));
        
        let mut sig_print = module.make_signature();
        sig_print.params.push(AbiParam::new(types::I32));
        
        let mut sig_print_str = module.make_signature();
        sig_print_str.params.push(AbiParam::new(types::I64)); // Pointer to string

        CodeGenerator {
            ctx,
            builder_context,
            module,
            variables: HashMap::new(),
            variable_count: 0,
            string_counter: 0,
            print_signature: sig_print,
            print_string_signature: sig_print_str,
            read_signature: sig_read,
        }
    }

    pub fn compile_expr(
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

    pub fn compile_assignment(
        &mut self,
        builder: &mut FunctionBuilder,
        assignment: &Assignment
    ) -> Result<(), String> {
        // Compile the expression first
        let value = self.compile_expr(builder, &assignment.expr);
        
        // Get or create the variable
        let var = match self.variables.get(&assignment.var) {
            Some(&v) => v,
            None => {
                // Create a new variable if it doesn't exist
                let new_var = Variable::new(self.get_variable_index());
                builder.declare_var(new_var, types::I32);
                self.variables.insert(assignment.var.clone(), new_var);
                new_var
            }
        };
    
        // Define/update the variable with the new value
        builder.def_var(var, value);
        
        // Update the symbol table
        let mut table = SymbolTable.lock().map_err(|_| "Symbol table lock poisoned")?;
        if let Some(symbol) = table.get_mut(&assignment.var) {
            // Convert the Cranelift value to TypeValue
            // For now, we're assuming all values are integers
            let type_value = match builder.func.dfg.value_type(value) {
                types::I32 => TypeValue::Integer(0), // Placeholder value
                types::F32 => TypeValue::Float(0.0), // Placeholder value
                _ => return Err("Unsupported type".to_string()),
            };
            
            symbol.Value = Some(vec![type_value]);
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


    pub fn compile_condition(
        &mut self, 
        builder: &mut FunctionBuilder, 
        condition: &Condition
    ) -> Value {
        match condition {
            Condition::Not(inner_condition) => {
                let inner_value = self.compile_condition(builder, inner_condition);
                // Create the constant first, separately
                let one = builder.ins().iconst(types::I32, 1);
                // Then use it in the XOR operation
                builder.ins().bxor(inner_value, one)
            },
            Condition::Logic(left, op, right) => {
                let left_value = self.compile_condition(builder, left);
                let right_value = self.compile_condition(builder, right);
                
                match op {
                    LogOp::And => builder.ins().band(left_value, right_value),
                    LogOp::Or => builder.ins().bor(left_value, right_value),
                }
            },
            Condition::Basic(basic_cond) => {
                let left = self.compile_expr(builder, &basic_cond.left);
                let right = self.compile_expr(builder, &basic_cond.right);
                
                match basic_cond.operator {
                    RelOp::Gt => builder.ins().icmp(IntCC::SignedGreaterThan, left, right),
                    RelOp::Lt => builder.ins().icmp(IntCC::SignedLessThan, left, right),
                    RelOp::Ge => builder.ins().icmp(IntCC::SignedGreaterThanOrEqual, left, right),
                    RelOp::Le => builder.ins().icmp(IntCC::SignedLessThanOrEqual, left, right),
                    RelOp::Eq => builder.ins().icmp(IntCC::Equal, left, right),
                    RelOp::Ne => builder.ins().icmp(IntCC::NotEqual, left, right),
                }
            }
        }
    }
    
    
    
    pub fn compile(&mut self, expr: &Expr) {
        let mut func = Function::new();
        let mut builder_context = std::mem::take(&mut self.builder_context);
        let mut builder = FunctionBuilder::new(&mut func, &mut builder_context);

        let entry_block = builder.create_block();
        builder.append_block_params_for_function_params(entry_block);
        builder.switch_to_block(entry_block);

        self.compile_expr(&mut builder, expr);
        
        builder.ins().return_(&[]);

        builder.seal_all_blocks();

        println!("{:?}", func);
    }

    pub fn test_setup(&mut self) -> Result<(), String> {
        println!("Code generator setup test successful!");
        Ok(())
    }


    pub fn compile_test_expr(&mut self, expr: &Expr) -> Result<(), String> {
        self.compile(expr);
        Ok(())
    }


    pub fn compile_if(
        &mut self,
        builder: &mut FunctionBuilder,
        if_stmt: &IfStmt
    ) {
        // Create blocks for the if structure
        let then_block = builder.create_block();
        let else_block = if if_stmt.else_block.is_some() {
            Some(builder.create_block())
        } else {
            None
        };
        let merge_block = builder.create_block();
    
        // Compile the condition
        let condition_value = self.compile_condition(builder, &if_stmt.condition);
        
        // Create zero constant for comparison
        let zero = builder.ins().iconst(types::I32, 0);
        let cond_not_zero = builder.ins().icmp(IntCC::NotEqual, condition_value, zero);
    
        // Branch based on condition
        match else_block {
            Some(else_blk) => {
                builder.ins().brif(cond_not_zero, then_block, &[], else_blk, &[]);
            }
            None => {
                builder.ins().brif(cond_not_zero, then_block, &[], merge_block, &[]);
            }
        }
    
        // Compile then block
        builder.switch_to_block(then_block);
        builder.seal_block(then_block);
        for instruction in &if_stmt.then_block {
            let _ = self.compile_instruction(builder, instruction);
        }
        builder.ins().jump(merge_block, &[]);
    
        // Compile else block if it exists
        if let Some(else_blk) = else_block {
            builder.switch_to_block(else_blk);
            builder.seal_block(else_blk);
            if let Some(else_instructions) = &if_stmt.else_block {
                for instruction in else_instructions {
                    let _ = self.compile_instruction(builder, instruction);
                }
            }
            builder.ins().jump(merge_block, &[]);
        }
    
        // Switch to merge block
        builder.switch_to_block(merge_block);
        builder.seal_block(merge_block);
    }
    


    pub fn compile_loop(
        &mut self, 
        builder: &mut FunctionBuilder, 
        loop_expr: &Loop
    ) {
        match loop_expr {
            Loop::While(condition, body) => {
                // Create the loop blocks
                let loop_header = builder.create_block();
                let loop_body = builder.create_block();
                let loop_end = builder.create_block();
    
                // Unconditionally branch to the loop_header
                builder.ins().jump(loop_header, &[]);
    
                // Switch to loop_header to evaluate the condition
                builder.switch_to_block(loop_header);
                builder.seal_block(loop_header);
    
                // Evaluate the loop condition
                let cond_value = self.compile_condition(builder, condition);
    
                // Create zero constant first
                let zero = builder.ins().iconst(types::I32, 0);
                
                // Then compare condition with zero
                let cond_not_zero = builder.ins().icmp(IntCC::NotEqual, cond_value, zero);
    
                // Branch based on condition
                builder.ins().brif(cond_not_zero, loop_body, &[], loop_end, &[]);
    
                // Loop body block
                builder.switch_to_block(loop_body);
                builder.seal_block(loop_body);
    
                self.compile_statement(builder, body);
    
                // Branch back to the loop_header to recheck the condition
                builder.ins().jump(loop_header, &[]);
    
                // End the loop
                builder.switch_to_block(loop_end);
                builder.seal_block(loop_end);
            }
        }
    }

    pub fn compile_for(
        &mut self,
        builder: &mut FunctionBuilder,
        for_stmt: &ForStmt
    ) {
        // Create blocks for the loop structure
        let header_block = builder.create_block();
        let body_block = builder.create_block();
        let exit_block = builder.create_block();
    
        // Compile initialization
        self.compile_assignment(builder, &for_stmt.init).unwrap();
        
        // Jump to header block
        builder.ins().jump(header_block, &[]);
        builder.switch_to_block(header_block);
    
        // Compile condition
        let condition_value = self.compile_expr(builder, &for_stmt.condition);
        let zero = builder.ins().iconst(types::I32, 0);
        let continue_condition = builder.ins().icmp(IntCC::NotEqual, condition_value, zero);
    
        // Conditional branch
        builder.ins().brif(continue_condition, body_block, &[], exit_block, &[]);
    
        // Compile loop body
        builder.switch_to_block(body_block);
        for instruction in &for_stmt.body {
            let _ = self.compile_instruction(builder, instruction);
        }
    
        // Compile step expression
        self.compile_expr(builder, &for_stmt.step);
    
        // Jump back to header
        builder.ins().jump(header_block, &[]);
    
        // Seal the loops
        builder.seal_block(header_block);
        builder.seal_block(body_block);
    
        // Switch to exit block
        builder.switch_to_block(exit_block);
        builder.seal_block(exit_block);
    }
    
    fn create_string_constant(&mut self, string: &str) -> Result<GlobalValue, ModuleError> {
        let string_id = self.module.declare_data(
            &format!("str_{}", self.string_counter),
            Linkage::Local,
            true,  // is_exported
            false, // is_writeable
        )?;
        
        let mut data_ctx = DataContext::new();
        data_ctx.define(string.as_bytes().to_vec().into_boxed_slice());
        self.module.define_data(string_id, &data_ctx)?;
        
        self.string_counter += 1;
        Ok(self.module.declare_data_in_func(string_id, &mut self.ctx.func))

    }

    fn get_variable_index(&mut self) -> usize {
        let index = self.variable_count;
        self.variable_count += 1;
        index
    }

    pub fn compile_read(
        &mut self,
        builder: &mut FunctionBuilder,
        read_stmt: &ReadStmt,
    ) -> Result<(), String> {
        // Declare the read function
        let read_func = self
            .module
            .declare_function("read_value", Linkage::Import, &self.read_signature)
            .map_err(|e| format!("Failed to declare read function: {}", e))?;
    
        let read_func_ref = self
            .module
            .declare_func_in_func(read_func, &mut builder.func); // Fixed mutable borrow here
        let call = builder.ins().call(read_func_ref, &[]);
        let value = builder.inst_results(call)[0];
    
        // Create a variable and store the value
        let var = Variable::new(self.get_variable_index());
    
        builder.declare_var(var, types::I32);
        builder.def_var(var, value);
    
        // Store the variable in our map
        self.variables.insert(read_stmt.variable.clone(), var);
    
        Ok(())
    }
    
    pub fn compile_write(
        &mut self,
        builder: &mut FunctionBuilder,
        write_stmt: &WriteStmt,
    ) -> Result<(), String> {
        for element in &write_stmt.elements {
            match element {
                WriteElement::String(string) => {
                    let string_global = self
                        .create_string_constant(string)
                        .map_err(|e| format!("Failed to create string constant: {}", e))?;
    
                    // Convert GlobalValue to Value
                    let string_value = builder.ins().global_value(types::I32, string_global); // Fixed type mismatch
    
                    let print_string_func = self
                        .module
                        .declare_function("print_string", Linkage::Import, &self.print_string_signature)
                        .map_err(|e| format!("Failed to declare print_string function: {}", e))?;
    
                    let func_ref = self
                        .module
                        .declare_func_in_func(print_string_func, &mut builder.func);
                    builder.ins().call(func_ref, &[string_value]); // Use `string_value` here
                }
                WriteElement::Variable(var_name) => {
                    if let Some(&var) = self.variables.get(var_name) {
                        let value = builder.use_var(var);
    
                        let print_func = self
                            .module
                            .declare_function("print_value", Linkage::Import, &self.print_signature)
                            .map_err(|e| format!("Failed to declare print function: {}", e))?;
    
                        let func_ref = self
                            .module
                            .declare_func_in_func(print_func, &mut builder.func);
                        builder.ins().call(func_ref, &[value]);
                    }
                }
            }
        }
        Ok(())
    }
    
    fn compile_instruction(
        &mut self,
        builder: &mut FunctionBuilder,
        instruction: &Instruction,
    ) -> Result<(), String> {
        match instruction {
            Instruction::Assign(assignment) => {
                self.compile_assignment(builder, assignment)?;
            }
            Instruction::If(if_stmt) => {
                self.compile_if(builder, if_stmt);
            }
            Instruction::For(for_stmt) => {
                self.compile_for(builder, for_stmt);
            }
            Instruction::Read(read_stmt) => {
                self.compile_read(builder, read_stmt)?;
            }
            Instruction::Write(write_stmt) => {
                self.compile_write(builder, write_stmt)?;
            }
        }
        Ok(())
    }
    
    fn compile_statement(
        &mut self,
        builder: &mut FunctionBuilder,
        stmt: &Statement,
    ) {
        match stmt {
            Statement::Expr(expr) => {
                self.compile_expr(builder, expr);
            }
            Statement::Assignment(assignment) => {
                self.compile_assignment(builder, assignment).unwrap();
            }
            Statement::Loop(loop_expr) => {
                self.compile_loop(builder, loop_expr);
            }
        }
    }
    
    
    
}