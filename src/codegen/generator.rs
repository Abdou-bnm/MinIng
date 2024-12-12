use cranelift::prelude::*;
use cranelift_codegen::ir::{Function, InstBuilder};
use cranelift_codegen::Context;
use cranelift_jit::{JITBuilder, JITModule};
use crate::Parser::ast::{Expr, TypeValue, BinOp};

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
        &self,
        builder: &mut FunctionBuilder,
        expr: &Expr,
    ) -> Value {
        match expr {
            Expr::Literal(value) => match value {
                TypeValue::Integer(n) => builder.ins().iconst(types::I32, *n as i64),
                TypeValue::Float(f) => builder.ins().f32const(*f),
                _ => unimplemented!("Other literal types not yet supported"),
            },
            Expr::BinaryOp(left, op, right) => {
                let lhs = self.compile_expr(builder, left);
                let rhs = self.compile_expr(builder, right);
                
                match op {
                    BinOp::Add => builder.ins().iadd(lhs, rhs),
                    BinOp::Sub => builder.ins().isub(lhs, rhs),
                    BinOp::Mul => builder.ins().imul(lhs, rhs),
                    BinOp::Div => builder.ins().sdiv(lhs, rhs),
                }
            },
            _ => unimplemented!("Other expression types not yet supported"),
        }
    }

    pub fn compile_test_expr(&mut self, expr: &Expr) -> Result<(), String> {
        let mut func = Function::new();
        let mut builder_context = std::mem::take(&mut self.builder_context);
        let mut builder = FunctionBuilder::new(&mut func, &mut builder_context);
        
        let entry_block = builder.create_block();
        builder.append_block_params_for_function_params(entry_block);
        builder.switch_to_block(entry_block);
        
        let result = self.compile_expr(&mut builder, expr);
        builder.ins().return_(&[result]);
        builder.seal_all_blocks();
        
        // Finalize the function and store it in the context
        self.ctx.func = func;

        // Print the generated IR
        let mut ir_output = String::new();
        cranelift_codegen::write_function(&mut ir_output, &self.ctx.func)
            .map_err(|e| format!("Error writing function IR: {:?}", e))?;
        println!("Generated IR:\n{}", ir_output);

        // Hand back the modified builder_context
        self.builder_context = builder_context;

        Ok(())
    }

    pub fn test_setup(&mut self) -> Result<(), String> {
        let mut func = Function::new();
        let mut builder = FunctionBuilder::new(&mut func, &mut self.builder_context);

        
        let entry_block = builder.create_block();
        builder.append_block_params_for_function_params(entry_block);
        builder.switch_to_block(entry_block);
        
        let const_value = builder.ins().iconst(types::I32, 42);
        builder.ins().return_(&[const_value]);
        builder.seal_all_blocks();
        
        self.ctx.func = func;
        Ok(())
    }
}

