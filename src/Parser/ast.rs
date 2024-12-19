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
    Variable(Type, Vec<Variable>),     // Variables with a type and a list of variables
    // USED ADEC INSTEAD OF ARRAY, AS REQUIRED BY THE TEACHER
    ADEC(Type, Vec<ArrayDecl>),     // Array declarations
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
    Simple((String, (usize, usize))),                // Simple variable (e.g., x)
    Initialized((String, (usize, usize)), Expr),     // Initialized variable (e.g., x = 10)   // Initialized variable (e.g., x = 10)
}

// Assignment structure: Variable assignment to an expression
#[derive(Debug, Clone)]
pub struct Assignment {
    pub var: (String, (usize, usize)),
    pub index: Option<Expr>,
    pub expr: Expr,
}

impl Assignment {
    pub fn new(var: (String, (usize, usize)), index: Option<Expr>, expr: Expr) -> Self {
        Assignment { var, index, expr }
    }
}

#[derive(Debug)]
pub enum ArrayDecl {
    Simple((String, (usize, usize)), Expr),
    Initialized((String, (usize, usize)), Expr, Vec<Expr>),
    InitializedString((String, (usize, usize)), Expr, (String, (usize, usize))),
}

// Expressions that can be literals, variables, or binary operations
#[derive(Debug, Clone)]
pub enum Expr {
    BinaryOp(Box<Expr>, BinOp, Box<Expr>),    // Binary operation (e.g., a + b)
    Variable((String, (usize, usize))),                         // Variable (e.g., x)
    SUBS((String, (usize, usize)), Box<Expr>),
    Literal(TypeValue),                         // Numeric or char literal
}

// Operations for binary expressions
#[derive(Debug,Clone)]
pub enum BinOp {
    Add(usize, usize),
    Sub(usize, usize),
    Mul(usize, usize),
    Div(usize, usize),
}

impl std::fmt::Display for BinOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let symbol = match self {
            BinOp::Add(line, column) => "+",
            BinOp::Sub(line, column) => "-",
            BinOp::Mul(line, column) => "*",
            BinOp::Div(line, column) => "/",
        };
        write!(f, "{}", symbol)
    }
}

// Literals (integers, floats, or characters)
#[derive(Clone, Debug, PartialEq)] // PartialEq for comparisons
pub enum TypeValue {
    Integer((i16, (usize, usize))),
    Float((f32, (usize, usize))),
    Char((char, (usize, usize))),
    Array(Vec<TypeValue>), // Array value representation
}

// Instruction types: Assignment, If statement, For loop, Read, Write
#[derive(Debug,Clone)]
pub enum Instruction {
    Assign(Assignment),
    If(IfStmt),
    For(ForStmt),
    Read(ReadStmt),
    Write(WriteStmt),
}

// If statement structure
#[derive(Debug, Clone)]
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
#[derive(Debug,Clone)]
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
#[derive(Debug, Clone)]
pub struct ReadStmt {
    pub variable: (String, (usize, usize)),
    pub index: Option<Expr>,
}

impl ReadStmt {
    pub fn new(variable: (String, (usize, usize)), index: Option<Expr>) -> Self {
        ReadStmt { variable, index }
    }
}

// Write statement (writes an element or expression)
#[derive(Debug, Clone)]
pub struct WriteStmt {
    pub elements: Vec<WriteElement>,
}

impl WriteStmt {
    pub fn new(elements: Vec<WriteElement>) -> Self {
        WriteStmt { elements }
    }
}

// Write elements (either a string or a variable)
#[derive(Debug,Clone)]
pub enum WriteElement {
    String((String, (usize, usize))),
    Variable((String, (usize, usize)), Option<Expr>),
}

// Conditions used in If statements and loops
#[derive(Debug, Clone)]
pub enum Condition {
    Not(Box<Condition>),          // Negation (e.g., !condition)
    Logic(Box<Condition>, LogOp, Box<Condition>),  // Logical AND/OR (e.g., cond1 && cond2)
    Basic(BasicCond),             // Basic condition (e.g., x > 5)
}

// Basic conditions for relational operations
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
pub enum RelOp {
    Gt(usize, usize),  // Greater than
    Lt(usize, usize),  // Less than
    Ge(usize, usize),  // Greater than or equal to
    Le(usize, usize),  // Less than or equal to
    Eq(usize, usize),  // Equal to
    Ne(usize, usize),  // Not equal to
}

// Logical operators for boolean operations
#[derive(Debug, Clone)]
pub enum LogOp {
    And(usize, usize), // Logical AND (&&)
    Or(usize, usize),  // Logical OR (||)
}