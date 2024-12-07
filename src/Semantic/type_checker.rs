use crate::Parser::ast::TypeValue;
use crate::Semantic::ts::Types;

pub struct TypeChecker;

impl TypeChecker {
    pub fn check_arithmetic_compatibility(left: &Types, right: &Types) -> Result<Types, String> {
        match (left, right) {
            (Types::Char, Types::Char) => Ok(Types::Char),
            (Types::Integer, Types::Integer) => Ok(Types::Integer),
            (Types::Float, Types::Float) => Ok(Types::Float),
            _ => Err("Incompatible types for arithmetic operation".to_string())
        }
    }

    pub fn check_assignment_compatibility(variable_type: &Types, value_type: &Types) -> Result<(), String> {
        match (variable_type, value_type) {
            (Types::Integer, Types::Integer) => Ok(()),
            (Types::Float, Types::Float) => Ok(()),
            (Types::Char, Types::Char) => Ok(()),
            (Types::Array(var_type, var_size), Types::Array(val_type, val_size)) 
                if **var_type == **val_type && *var_size == *val_size => Ok(()),
            _ => Err(format!("Cannot assign {:?} to {:?}", value_type, variable_type))
        }
    }

    pub fn infer_expression_type(value: &TypeValue) -> Types {
        match value {
            TypeValue::Integer(_) => Types::Integer,
            TypeValue::Float(_) => Types::Float,
            TypeValue::Char(_) => Types::Char,
            TypeValue::Array(arr) if arr.iter().all(|v| matches!(v, TypeValue::Integer(_))) => 
                Types::Array(Box::new(Types::Integer), arr.len() as i16),
            TypeValue::Array(arr) if arr.iter().all(|v| matches!(v, TypeValue::Float(_))) => 
                Types::Array(Box::new(Types::Float), arr.len() as i16),
            TypeValue::Array(arr) if arr.iter().all(|v| matches!(v, TypeValue::Char(_))) => 
                Types::Array(Box::new(Types::Char), arr.len() as i16),
            _ => panic!("Mixed type arrays are not supported")
        }
    }

    // Additional helper methods can be added here
    pub fn are_types_compatible(type1: &Types, type2: &Types) -> bool {
        match (type1, type2) {
            (Types::Integer, Types::Integer) => true,
            (Types::Float, Types::Float) => true,
            (Types::Char, Types::Char) => true,
            (Types::Array(t1, s1), Types::Array(t2, s2)) => t1 == t2 && s1 == s2,
            _ => false
        }
    }
}

