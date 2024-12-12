use std::fmt::Write;
use crate::Parser::ast::{
    Program, Declaration, Instruction, Expr, BinOp, Assignment, 
    IfStmt, ForStmt, ReadStmt, WriteStmt, WriteElement, 
    Condition, BasicCond, RelOp, Type, Variable, ArrayDecl, TypeValue, LogOp
};

pub struct CodeGenerator {
    output: String,
    indent_level: usize,
}

impl CodeGenerator {
    pub fn new() -> Self {
        CodeGenerator {
            output: String::new(),
            indent_level: 0,
        }
    }

    fn indent(&self) -> String {
        "    ".repeat(self.indent_level)
    }

    pub fn generate(&mut self, program: &Program) -> Result<String, String> {
        // Start with C-style boilerplate
        self.output.push_str("#include <stdio.h>\n");
        self.output.push_str("#include <stdbool.h>\n\n");

        // Generate global variable declarations
        if let Some(global_vars) = &program.global {
            self.generate_declarations(global_vars)?;
        }

        // Generate other declarations
        if let Some(decls) = &program.decls {
            self.generate_declarations(decls)?;
        }

        // Start main function
        self.output.push_str("int main() {\n");
        self.indent_level += 1;

        // Generate instructions
        if let Some(instructions) = &program.inst {
            self.generate_instructions(instructions)?;
        }

        // Close main function
        self.output.push_str("    return 0;\n");
        self.indent_level -= 1;
        self.output.push_str("}\n");

        Ok(self.output.clone())
    }

    fn generate_declarations(&mut self, declarations: &Vec<Declaration>) -> Result<(), String> {
        for decl in declarations {
            match decl {
                Declaration::Variables(type_decl, vars) => {
                    for var in vars {
                        match var {
                            Variable::Simple(name) => {
                                write!(self.output, "{}", self.indent()).unwrap();
                                self.generate_type(type_decl)?;
                                write!(self.output, " {};\n", name).unwrap();
                            },
                            Variable::Initialized(name, expr) => {
                                write!(self.output, "{}", self.indent()).unwrap();
                                self.generate_type(type_decl)?;
                                write!(self.output, " {} = ", name).unwrap();
                                self.generate_expression(expr)?;
                                self.output.push_str(";\n");
                            }
                        }
                    }
                },
                Declaration::Array(type_decl, arrays) => {
                    for arr in arrays {
                        match arr {
                            ArrayDecl::Simple(name, size_expr) => {
                                write!(self.output, "{}", self.indent()).unwrap();
                                self.generate_type(type_decl)?;
                                write!(self.output, " {}[", name).unwrap();
                                self.generate_expression(size_expr)?;
                                self.output.push_str("];\n");
                            },
                            ArrayDecl::Initialized(name, size_expr, values) => {
                                write!(self.output, "{}", self.indent()).unwrap();
                                self.generate_type(type_decl)?;
                                write!(self.output, " {} = {{", name).unwrap();
                                for (i, val) in values.iter().enumerate() {
                                    self.generate_expression(val)?;
                                    if i < values.len() - 1 {
                                        self.output.push_str(", ");
                                    }
                                }
                                self.output.push_str("};\n");
                            },
                            ArrayDecl::InitializedString(name, size_expr, value) => {
                                write!(self.output, "{}", self.indent()).unwrap();
                                self.generate_type(type_decl)?;
                                write!(self.output, " {}[] = {};\n", name, value).unwrap();
                            }
                        }
                    }
                },
                Declaration::Constant(type_decl, constants) => {
                    for constant in constants {
                        write!(self.output, "const ").unwrap();
                        self.generate_type(type_decl)?;
                        write!(self.output, " {} = ", constant.var).unwrap();
                        self.generate_expression(&constant.expr)?;
                        self.output.push_str(";\n");
                    }
                }
            }
        }
        Ok(())
    }

    fn generate_instructions(&mut self, instructions: &Vec<Instruction>) -> Result<(), String> {
        for instruction in instructions {
            match instruction {
                Instruction::Assign(assignment) => {
                    write!(self.output, "{}", self.indent()).unwrap();
                    self.generate_assignment(assignment)?;
                    self.output.push_str(";\n");
                },
                Instruction::If(if_stmt) => {
                    self.generate_if_statement(if_stmt)?;
                },
                Instruction::For(for_loop) => {
                    self.generate_for_loop(for_loop)?;
                },
                Instruction::Read(read_stmt) => {
                    write!(self.output, "{}", self.indent()).unwrap();
                    self.generate_read(read_stmt)?;
                    self.output.push_str(";\n");
                },
                Instruction::Write(write_stmt) => {
                    write!(self.output, "{}", self.indent()).unwrap();
                    self.generate_write(write_stmt)?;
                    self.output.push_str(";\n");
                }
            }
        }
        Ok(())
    }

    fn generate_type(&mut self, type_decl: &Type) -> Result<(), String> {
        match type_decl {
            Type::Integer => self.output.push_str("int"),
            Type::Float => self.output.push_str("float"),
            Type::Char => self.output.push_str("char")
        }
        Ok(())
    }

    fn generate_expression(&mut self, expr: &Expr) -> Result<(), String> {
        match expr {
            Expr::Literal(lit) => {
                match lit {
                    TypeValue::Integer(i) => write!(self.output, "{}", i).unwrap(),
                    TypeValue::Float(f) => write!(self.output, "{}", f).unwrap(),
                    TypeValue::Char(c) => write!(self.output, "'{}'", c).unwrap(),
                    TypeValue::Array(_) => return Err("Cannot generate code for array literal".to_string()),
                }
            },
            Expr::Variable(var) => self.output.push_str(var),
            Expr::Array(var, index) => {
                write!(self.output, "{}[", var).unwrap();
                self.generate_expression(index)?;
                self.output.push_str("]");
            },
            Expr::BinaryOp(left, op, right) => {
                self.output.push_str("(");
                self.generate_expression(left)?;
                match op {
                    BinOp::Add => self.output.push_str(" + "),
                    BinOp::Sub => self.output.push_str(" - "),
                    BinOp::Mul => self.output.push_str(" * "),
                    BinOp::Div => self.output.push_str(" / "),
                }
                self.generate_expression(right)?;
                self.output.push_str(")");
            }
        }
        Ok(())
    }

    fn generate_assignment(&mut self, assignment: &Assignment) -> Result<(), String> {
        if let Some(index) = &assignment.index {
            // Array assignment
            write!(self.output, "{}[", assignment.var).unwrap();
            self.generate_expression(index)?;
            write!(self.output, "] = ").unwrap();
        } else {
            // Simple variable assignment
            write!(self.output, "{} = ", assignment.var).unwrap();
        }
        self.generate_expression(&assignment.expr)?;
        Ok(())
    }

    fn generate_condition(&mut self, condition: &Condition) -> Result<(), String> {
        match condition {
            Condition::Not(inner) => {
                self.output.push_str("!(");
                self.generate_condition(inner)?;
                self.output.push_str(")");
            },
            Condition::Logic(left, op, right) => {
                self.output.push_str("(");
                self.generate_condition(left)?;
                match op {
                    LogOp::And => self.output.push_str(" && "),
                    LogOp::Or => self.output.push_str(" || "),
                }
                self.generate_condition(right)?;
                self.output.push_str(")");
            },
            Condition::Basic(basic_cond) => {
                self.generate_expression(&basic_cond.left)?;
                match basic_cond.operator {
                    RelOp::Gt => self.output.push_str(" > "),
                    RelOp::Lt => self.output.push_str(" < "),
                    RelOp::Ge => self.output.push_str(" >= "),
                    RelOp::Le => self.output.push_str(" <= "),
                    RelOp::Eq => self.output.push_str(" == "),
                    RelOp::Ne => self.output.push_str(" != "),
                }
                self.generate_expression(&basic_cond.right)?;
            }
        }
        Ok(())
    }

    fn generate_if_statement(&mut self, if_stmt: &IfStmt) -> Result<(), String> {
        write!(self.output, "{}if (", self.indent()).unwrap();
        self.generate_condition(&if_stmt.condition)?;
        self.output.push_str(") {\n");
        
        self.indent_level += 1;
        self.generate_instructions(&if_stmt.then_block)?;
        self.indent_level -= 1;
        
        write!(self.output, "{}", self.indent()).unwrap();
        self.output.push_str("}\n");
        
        if let Some(else_block) = &if_stmt.else_block {
            write!(self.output, "{}else {{\n", self.indent()).unwrap();
            self.indent_level += 1;
            self.generate_instructions(else_block)?;
            self.indent_level -= 1;
            write!(self.output, "{}", self.indent()).unwrap();
            self.output.push_str("}\n");
        }
        
        Ok(())
    }

    fn generate_for_loop(&mut self, for_loop: &ForStmt) -> Result<(), String> {
        write!(self.output, "{}for ({} = ", self.indent(), for_loop.init.var).unwrap();
        self.generate_expression(&for_loop.init.expr)?;
        self.output.push_str("; ");
        
        // Generate condition
        write!(self.output, "{} < ", for_loop.init.var).unwrap();
        self.generate_expression(&for_loop.condition)?;
        self.output.push_str("; ");
        
        // Generate step
        write!(self.output, "{} += ", for_loop.init.var).unwrap();
        self.generate_expression(&for_loop.step)?;
        self.output.push_str(") {\n");
        
        self.indent_level += 1;
        self.generate_instructions(&for_loop.body)?;
        self.indent_level -= 1;
        
        write!(self.output, "{}", self.indent()).unwrap();
        self.output.push_str("}\n");
        
        Ok(())
    }

    fn generate_read(&mut self, read_stmt: &ReadStmt) -> Result<(), String> {
        write!(self.output, "scanf(\"%d\", &{});", read_stmt.variable).unwrap();
        Ok(())
    }

    fn generate_write(&mut self, write_stmt: &WriteStmt) -> Result<(), String> {
        let mut format_str = String::new();
        let mut args = Vec::new();

        for element in &write_stmt.elements {
            match element {
                WriteElement::String(s) => {
                    format_str.push_str(s);
                },
                WriteElement::Variable(var) => {
                    format_str.push_str("%d ");
                    args.push(var);
                }
            }
        }

        if !args.is_empty() {
            write!(self.output, "printf(\"{}\", ", format_str).unwrap();
            for (i, arg) in args.iter().enumerate() {
                self.output.push_str(arg);
                if i < args.len() - 1 {
                    self.output.push_str(", ");
                }
            }
            self.output.push_str(")");
        } else {
            write!(self.output, "printf(\"{}\")", format_str).unwrap();
        }

        Ok(())
    }
}