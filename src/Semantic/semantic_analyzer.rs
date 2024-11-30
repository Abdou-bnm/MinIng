use crate::Semantic::ts::{SymbolTable, Symbol, Types, TypeValue};
use crate::Semantic::type_checker::TypeChecker;
use crate::Semantic::semantic_rules::SemanticRules;
use crate::Semantic::error::SemanticError;

pub struct SemanticAnalyzer {
    symbol_table: SymbolTable,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        SemanticAnalyzer {
            symbol_table: SymbolTable::new(),
        }
    }

    pub fn analyze_variable_declaration(
        &mut self,
        name: String,
        symbol_type: Types,
        is_constant: bool,
        value: Option<TypeValue>,
    ) -> Result<(), SemanticError> {
        // Validate semantic rules
        SemanticRules::validate_variable_declaration(&name, &symbol_type, is_constant, value.as_ref())
            .map_err(|e| SemanticError::InvalidDeclaration(e.to_string()))?;

        // Check for duplicates in the symbol table
        if self.symbol_table.lookup(&name).is_some() {
            return Err(SemanticError::DuplicateVariableDeclaration(name));
        }

        // Create the symbol
        let symbol = Symbol::new(
            name.clone(),
            Some(symbol_type),
            Some(is_constant),
            Some(self.symbol_table.get_next_address()),
            value,
        );

        // Insert into the symbol table
        self.symbol_table
            .insert(symbol)
            .map_err(|e| SemanticError::SymbolTableError(e))
    }

    pub fn analyze_array_declaration(
        &mut self,
        name: String,
        element_type: Types,
        size: usize,
    ) -> Result<(), SemanticError> {
        // Validate array declaration
        SemanticRules::validate_array_declaration(&name, &element_type, size)
            .map_err(|e| SemanticError::InvalidDeclaration(e.to_string()))?;

        // Check for duplicates
        if self.symbol_table.lookup(&name).is_some() {
            return Err(SemanticError::DuplicateVariableDeclaration(name));
        }

        // Create array type and symbol
        let array_type = Types::Array(Box::new(element_type), size);
        let symbol = Symbol::new(
            name.clone(),
            Some(array_type),
            Some(false),
            Some(self.symbol_table.get_next_address()),
            None,
        );

        // Insert into the symbol table
        self.symbol_table
            .insert(symbol)
            .map_err(|e| SemanticError::SymbolTableError(e))
    }

    pub fn analyze_assignment(&mut self, name: &str, value: TypeValue) -> Result<(), SemanticError> {
        // Lookup the variable
        let symbol = self
            .symbol_table
            .lookup(name)
            .ok_or_else(|| SemanticError::UndeclaredVariable(name.to_string()))?;

        // Check if the symbol is constant
        if symbol.Is_Constant.unwrap_or(false) {
            return Err(SemanticError::ConstantModification(name.to_string()));
        }

        // Check type compatibility
        let value_type = TypeChecker::infer_expression_type(&value);
        TypeChecker::check_assignment_compatibility(symbol.Type.as_ref().unwrap(), &value_type)
        .map_err(|e| SemanticError::TypeMismatch(symbol.Type.as_ref().unwrap().clone(), value_type))?;

        // Update the symbol's value
        self.symbol_table
            .update(name, value)
            .map_err(|e| SemanticError::SymbolTableError(e))
    }

    pub fn analyze_condition(&self, condition: &Types) -> Result<(), SemanticError> {
        // Validate the condition type
        SemanticRules::validate_condition(condition)
            .map_err(|e| SemanticError::InvalidDeclaration(e.to_string()))
    }
}
