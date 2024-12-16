use cranelift::prelude::*;
use cranelift_codegen::ir::{Function, InstBuilder};
use cranelift_codegen::Context;
use cranelift_jit::{JITBuilder, JITModule};
use std::sync::Mutex;
use crate::Parser::ast::{Expr, TypeValue, BinOp, Assignment,Condition, RelOp, LogOp};
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

    fn compile_assignment(
        &mut self,
        builder: &mut FunctionBuilder,
        assignment: &Assignment
    ) -> Result<(), String> {
        let value = self.compile_expr(builder, &assignment.expr);
        
        let mut table = SymbolTable.lock().map_err(|_| "Symbol table lock poisoned")?;
        if let Some(symbol) = table.get_mut(&assignment.var) {
            // Update the symbol's value
            symbol.Value = Some(vec![TypeValue::Integer(0)]); // Replace placeholder logic
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


    fn compile_condition(
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


    
    
}
