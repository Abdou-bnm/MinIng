use crate::Semantic::ts::{Types, TypeValue};
use crate::Semantic::type_checker::TypeChecker;

pub struct SemanticRules;

impl SemanticRules {
    pub fn validate_variable_declaration(
        name: &str, 
        symbol_type: &Types, 
        is_constant: bool, 
        value: Option<&TypeValue>
    ) -> Result<(), String> {
        // Check variable name length
        if name.len() > 8 {
            return Err(format!("Identifier '{}' cannot exceed 8 characters", name));
        }

        // Validate constant initialization
        if is_constant && value.is_none() {
            return Err(format!("Constant '{}' must be initialized at declaration", name));
        }

        // Type checking for initialization
        if let Some(val) = value {
            let val_type = TypeChecker::infer_expression_type(val);
            TypeChecker::check_assignment_compatibility(symbol_type, &val_type)?;
        }

        Ok(())
    }

    pub fn validate_array_declaration(
        name: &str, 
        element_type: &Types, 
        size: usize
    ) -> Result<(), String> {
        // Check array name length
        if name.len() > 8 {
            return Err(format!("Identifier '{}' cannot exceed 8 characters", name));
        }

        // Validate array size
        if size == 0 {
            return Err(format!("Array '{}' must have a positive size", name));
        }

        // Validate array type
        match element_type {
            Types::Integer | Types::Float | Types::Char => Ok(()),
            _ => Err(format!("Invalid array type for '{}'", name))
        }
    }

    pub fn validate_condition(condition_type: &Types) -> Result<(), String> {
        // In MinING, conditions should resolve to an integer (0 or 1)
        match condition_type {
            Types::Integer => Ok(()),
            _ => Err("Condition must be an integer expression".to_string())
        }
    }
}
