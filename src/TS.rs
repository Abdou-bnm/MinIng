#![allow(nonstandard_style)]

use crate::lexer::Type;
use std::sync::atomic::{AtomicBool, Ordering};

// IB stands for InstructionBlock, A value of true indicates we have completed the VAR_GLOBAL and the declarations BLOCK, and are currently in the instructions block..
pub static IB_FLAG: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Debug)]
pub enum Types {
    Integer,
    Float,
    Char,
    Array
}

#[derive(Clone, Debug)]
pub enum TypeValue {
    Integer(i16),
    Float(f32),
    Char(char),
    Array(Vec<TypeValue>)
}
#[derive(Debug)]
pub struct Symbol {
    pub Identifier: String,
    pub Type: Option<Types>,
    pub Is_Constant: Option<bool>,
    pub Address: Option<u64>,
    pub Value: Option<TypeValue>,
}

impl Symbol {
    pub fn new(Identifier: String, Type: Option<Types>, Is_Constant: Option<bool>, Address: Option<u64>, Value: Option<TypeValue> ) -> Symbol {
        Symbol {
            Identifier,
            Type,
            Is_Constant,
            Address,
            Value
        }
    }
}

use std::fmt;

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Identifier: {}\n\tIdentifier: \"{}\"\n\tType: {:?}\n\tConstant: {:?}\n\tAddress: {:?}\n\tValue: {:?}\n",
            self.Identifier, self.Identifier, self.Type, self.Is_Constant, self.Address, self.Value
        )
    }
}
