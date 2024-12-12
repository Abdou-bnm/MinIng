#[cfg(test)]
mod code_generator_tests {
    use crate::Lexer::lexer::Lexer;
    use crate::Parser::parser::Parser;
    use crate::SemanticAnalyzer::semantic_analyzer::SemanticAnalyzer;
    use crate::CodeGenerator::code_generator::CodeGenerator;
    use std::fs::{self, File};
    use std::io::{Write, Command};

    fn test_code_generation(input: &str) -> Result<String, String> {
        // Tokenize
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().map_err(|e| e.to_string())?;

        // Parse
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().map_err(|e| e.to_string())?;

        // Semantic Analysis
        let mut semantic_analyzer = SemanticAnalyzer::new();
        semantic_analyzer.analyze(&ast).map_err(|e| e.to_string())?;

        // Code Generation
        let mut code_generator = CodeGenerator::new();
        code_generator.generate(&ast)
    }

    fn compile_and_run_c_code(c_code: &str) -> Result<String, String> {
        // Write C code to a temporary file
        let mut temp_c_file = tempfile::NamedTempFile::new()
            .map_err(|e| format!("Failed to create temp C file: {}", e))?;
        
        temp_c_file.write_all(c_code.as_bytes())
            .map_err(|e| format!("Failed to write C code: {}", e))?;
        
        let c_path = temp_c_file.path();
        let out_path = c_path.with_extension("out");

        // Compile C code
        let compile_output = Command::new("gcc")
            .arg(c_path)
            .arg("-o")
            .arg(&out_path)
            .output()
            .map_err(|e| format!("Compilation failed: {}", e))?;
        
        if !compile_output.status.success() {
            return Err(format!("Compilation error: {}", 
                String::from_utf8_lossy(&compile_output.stderr)));
        }

        // Run compiled program
        let run_output = Command::new(&out_path)
            .output()
            .map_err(|e| format!("Execution failed: {}", e))?;
        
        if run_output.status.success() {
            Ok(String::from_utf8_lossy(&run_output.stdout).to_string())
        } else {
            Err(format!("Execution error: {}", 
                String::from_utf8_lossy(&run_output.stderr)))
        }
    }

    #[test]
    fn test_simple_variable_declaration() {
        let input = "
            var int x;
            x = 10;
            write x;
        ";

        let c_code = test_code_generation(input).expect("Code generation failed");
        let output = compile_and_run_c_code(&c_code).expect("Compilation/execution failed");

        assert!(output.contains("10"));
    }

    #[test]
    fn test_if_statement() {
        let input = "
            var int x;
            x = 10;
            if x > 5 {
                write 1;
            } else {
                write 0;
            }
        ";

        let c_code = test_code_generation(input).expect("Code generation failed");
        let output = compile_and_run_c_code(&c_code).expect("Compilation/execution failed");

        assert!(output.contains("1"));
    }

    #[test]
    fn test_for_loop() {
        let input = "
            var int sum;
            sum = 0;
            for i = 0; i < 5; i += 1 {
                sum = sum + i;
            }
            write sum;
        ";

        let c_code = test_code_generation(input).expect("Code generation failed");
        let output = compile_and_run_c_code(&c_code).expect("Compilation/execution failed");

        assert!(output.contains("10")); // 0 + 1 + 2 + 3 + 4
    }
}