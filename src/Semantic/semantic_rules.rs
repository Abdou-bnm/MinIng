use crate::Parser::ast::{Condition, TypeValue};
use crate::Semantic::ts::Types;
use crate::Semantic::type_checker::TypeChecker;

pub struct SemanticRules;

impl SemanticRules {
    pub fn validate_variable_declaration(
        name: (String, (usize, usize)),
        symbol_type: &Types,
        is_constant: bool,
        value: Option<&TypeValue>
    ) -> Result<(), String> {
        // Check variable name length
        if name.0.len() > 8 {
            return Err(format!("Identifier '{}' cannot exceed 8 characters at ({}:{})", name.0, name.1.0, name.1.1));
        }
        // Validate constant initialization
        if is_constant && value.is_none() {
            return Err(format!("Constant '{}' must be initialized at declaration at ({}:{})", name.0, name.1.0, name.1.1));
        }

        // Type checking for initialization
        if let Some(val) = value {
            let val_type = TypeChecker::infer_expression_type(val);
            TypeChecker::check_assignment_compatibility(symbol_type, &val_type)?;
        }

        Ok(())
    }

    pub fn validate_array_declaration(
        name: (String, (usize, usize)),
        element_type: &Types,
        size: i16
    ) -> Result<(), String> {
        // Check array name length
        if name.0.len() > 8 {
            return Err(format!("Identifier '{}' cannot exceed 8 characters at ({}:{})", name.0, name.1.0, name.1.1))
        }

        // Validate array size
        if size <= 0 {
            return Err(format!("Array '{}' must have a positive size at ({}:{})", name.0, name.1.0, name.1.1));
        }

        // Validate array type
        match element_type {
            Types::Integer | Types::Float | Types::Char => Ok(()),
            _ => Err(format!("Invalid array type for '{}' at ({}:{})", name.0, name.1.0, name.1.1))
        }
    }
    
    pub fn validate_condition(condition: &Condition, type_check_func: &mut dyn FnMut(&Condition) -> Result<Types, String>) -> Result<(), String> {
        match condition {
            Condition::Not(inner_condition) => {
                // Recursive validation for negated condition
                Self::validate_condition(inner_condition, type_check_func)
            },
            Condition::Logic(left_cond, _, right_cond) => {
                // Validate both sides of logical conditions
                Self::validate_condition(left_cond, type_check_func)?;
                Self::validate_condition(right_cond, type_check_func)?;
                Ok(())
            },
            Condition::Basic(basic_cond) => {
                // Validate basic condition by checking its resolved type
                let condition_type = type_check_func(condition)?;

                // We expect the condition to resolve to an integer (0 or 1)
                match condition_type {
                    Types::Integer => Ok(()),
                    Types::Float => Ok(()),
                    _ => Err("Condition must resolve to an integer or float expression".to_string())
                }
            }
        }
    }
}