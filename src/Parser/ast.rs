// Define your AST structures here

// Program structure that holds the global variables, declarations, and instructions
#[derive(Debug)]
pub struct Program {
    pub global: Option<Vec<Declaration>>, // Optional global variable declarations
    pub decls: Option<Vec<Declaration>>,  // Optional other declarations
    pub inst: Option<Vec<Instruction>>,   // Optional instructions
}

impl Program {
    pub fn new(global: Option<Vec<Declaration>>, decls: Option<Vec<Declaration>>, inst: Option<Vec<Instruction>>) -> Self {
        Program { global, decls, inst }
    }
}

// Declaration types: Variables, Arrays, Constants
#[derive(Debug)]
pub enum Declaration {
    Variables(Type, Vec<Variable>),     // Variables with a type and a list of variables
    Array(Type, Vec<ArrayDecl>),        // Array declarations
    Constant(Type, Vec<Assignment>),    // Constant declarations
}

// Types for declarations
#[derive(Debug)]
pub enum Type {
    Integer,
    Float,
    Char,
}

// Variable types: Simple variables or initialized variables
#[derive(Debug)]
pub enum Variable {
    Simple(String),                // Simple variable (e.g., x)
    Initialized(String, Expr),     // Initialized variable (e.g., x = 10)
}

// Assignment structure: Variable assignment to an expression
#[derive(Debug)]
pub struct Assignment {
    pub var: String,
    pub index: Option<Expr>,
    pub expr: Expr,
}

impl Assignment {
    pub fn new(var: String, index: Option<Expr>, expr: Expr) -> Self {
        Assignment { var, index, expr }
    }
}

#[derive(Debug)]
pub enum ArrayDecl {
    Simple(String, Expr),
    Initialized(String, Expr, Vec<Expr>),
    InitializedString(String, Expr, String),
}

// Expressions that can be literals, variables, or binary operations
#[derive(Debug)]
pub enum Expr {
    BinaryOp(Box<Expr>, BinOp, Box<Expr>),   // Binary operation (e.g., a + b)
    Variable(String),                         // Variable (e.g., x)
    Literal(Literal),                         // Numeric or char literal
}

// Operations for binary expressions
#[derive(Debug)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
}

// Literals (integers, floats, or characters)
#[derive(Debug)]
pub enum Literal {
    Integer(i16),
    Float(f32),
    Char(char),
}

// Instruction types: Assignment, If statement, For loop, Read, Write
#[derive(Debug)]
pub enum Instruction {
    Assign(Assignment),
    If(IfStmt),
    For(ForStmt),
    Read(ReadStmt),
    Write(WriteStmt),
}

// If statement structure
#[derive(Debug)]
pub struct IfStmt {
    pub condition: Condition,
    pub then_block: Vec<Instruction>,
    pub else_block: Option<Vec<Instruction>>,
}

impl IfStmt {
    pub fn new(condition: Condition, then_block: Vec<Instruction>, else_block: Option<Vec<Instruction>>) -> Self {
        IfStmt { condition, then_block, else_block }
    }
}

// For loop structure
#[derive(Debug)]
pub struct ForStmt {
    pub init: Assignment,
    pub step: Expr,
    pub condition: Expr,
    pub body: Vec<Instruction>,
}

impl ForStmt {
    pub fn new(init: Assignment, step: Expr, condition: Expr, body: Vec<Instruction>) -> Self {
        ForStmt { init, step, condition, body }
    }
}

// Read statement (reads a variable)
#[derive(Debug)]
pub struct ReadStmt {
    pub variable: String,
}

impl ReadStmt {
    pub fn new(variable: String) -> Self {
        ReadStmt { variable }
    }
}

// Write statement (writes an element or expression)
#[derive(Debug)]
pub struct WriteStmt {
    pub elements: Vec<WriteElement>,
}

impl WriteStmt {
    pub fn new(elements: Vec<WriteElement>) -> Self {
        WriteStmt { elements }
    }
}

// Write elements (either a string or a variable)
#[derive(Debug)]
pub enum WriteElement {
    String(String),
    Variable(String),
}

// Conditions used in If statements and loops
#[derive(Debug)]
pub enum Condition {
    Not(Box<Condition>),          // Negation (e.g., !condition)
    Logic(Box<Condition>, LogOp, Box<Condition>),  // Logical AND/OR (e.g., cond1 && cond2)
    Basic(BasicCond),             // Basic condition (e.g., x > 5)
}

// Basic conditions for relational operations
#[derive(Debug)]
pub struct BasicCond {
    pub left: Expr,
    pub operator: RelOp,
    pub right: Expr,
}

impl BasicCond {
    pub fn new(left: Expr, operator: RelOp, right: Expr) -> Self {
        BasicCond { left, operator, right }
    }
}

// Relational operators for comparisons
#[derive(Debug)]
pub enum RelOp {
    Gt,  // Greater than
    Lt,  // Less than
    Ge,  // Greater than or equal to
    Le,  // Less than or equal to
    Eq,  // Equal to
    Ne,  // Not equal to
}

// Logical operators for boolean operations
#[derive(Debug)]
pub enum LogOp {
    And, // Logical AND (&&)
    Or,  // Logical OR (||)
}