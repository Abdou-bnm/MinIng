# Custom Language Compiler

This is a compiler implementation that includes lexical, syntactic,semantic analysis, and machine code for a custom programming language named MinING. The compiler is written in Rust and uses LOGOS as a lexer generator and LALRPOP for parsing.

## Features

- Lexical analysis with token identification
- Syntax parsing
- Semantic analysis
- Symbol table generation
- Support for various data types:
    - INTEGER
    - FLOAT
    - CHAR
    - Arrays of these types
- Program structure with global variables, declarations, and instructions
- Control structures (IF-ELSE, FOR loops)
- Input/Output operations (READ, WRITE)

## Installation

1. Make sure you have Rust installed on your system. If not, install it from [rustup.rs](https://rustup.rs/)
2. Clone this repository:
   ```bash
   git clone https://github.com/Abdou-bnm/MinIng/
   cd MinIng
   ```
3. Build the project:
   ```bash
   cargo build
   ```

## Usage

The compiler can be run in two modes:

### 1. File Input Mode

To compile a program from a file:
```bash
cargo run path/to/your/program.txt
```

### 2. Default Example Mode

To run the built-in example program:
```bash
cargo run
```

## Program Structure

Programs should follow this basic structure:

```
VAR_GLOBAL {
    // Global variable declarations
}
DECLARATION {
    // Constant declarations
}
INSTRUCTION {
    // Program instructions
}
```

## Example Program

Here's a simple example program:

```
VAR_GLOBAL {
    INTEGER X = 1;
    FLOAT Y = 2.0;
    CHAR Z[5] = "Hello";
}
DECLARATION {
    CONST INTEGER MAX = 100;
}
INSTRUCTION {
    READ(X);
    IF(X > 0) {
        Y = Y + X;
    } ELSE {
        Y = 0;
    }
    WRITE("Result: ", Y);
}
```

## Language Features

### Variable Declarations
- Global variables: Declared in VAR_GLOBAL section
- Constants: Declared in DECLARATION section with CONST keyword
- Supported types: INTEGER, FLOAT, CHAR
- Array declarations supported with size in brackets

### Operations
- Arithmetic: +, -, *, /
- Comparison: >, <, >=, <=, ==
- Assignment: =, +=

### Control Structures
- IF-ELSE statements
- FOR loops with format: FOR(var = start : step : end)

### Input/Output
- READ(variable): Read input into a variable
- WRITE(expression): Output an expression
- Support for string literals in WRITE statements

### Comments
Use %% for single-line comments:
```
%% This is a comment
```

## Compiler Output

The compiler provides detailed feedback at each stage:

1. Lexical Analysis
    - Shows all tokens found
    - Reports any lexical errors

2. Syntactic Analysis
    - Confirms successful parsing
    - Reports syntax errors

3. Semantic Analysis
    - Displays the Abstract Syntax Tree (AST)
    - Shows program structure
    - Reports semantic errors

4. Symbol Table
    - Displays all symbols and their properties

## Error Handling

The compiler provides clear error messages for:
- File reading errors
- Lexical errors (invalid tokens)
- Syntax errors (invalid program structure)
- Semantic errors (type mismatches, undefined variables, etc.)

## Development

This compiler is built using several Rust crates:
- LALRPOP for parser generation
- Logos for lexical analysis
- Colored for terminal output formatting

## Contributers
 - KARA Nabil
 - MEDJBER Abderrahim
 - BENAMIROUCHE Abderaouf
 - HIRECHE Hichem
 - CHABATI Rayan
 - NAILI Walid
## Contributing

Feel free to submit issues and enhancement requests!

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details
