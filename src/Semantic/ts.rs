use std::collections::HashMap;
use std::fmt::format;
use std::ops::Deref;
use std::sync::Mutex;
use logos::Source;
use once_cell::sync::Lazy;
use crate::Parser::ast::TypeValue;
use crate::Semantic::ts;
// Global static flag (this can be adjusted or removed as needed)
#[derive(Clone, Debug, PartialEq)] // PartialEq for equality comparisons
pub enum Types {
    Integer,
    Float,
    Char,
    Array(Box<Types>, i16), // Array with element type and size
}

#[derive(Debug, Clone)]
pub struct Symbol {
    pub Identifier: String,
    pub Type: Option<Types>,
    pub Is_Constant: Option<bool>,
    pub Address: Option<usize>,
    pub Value: Option<Vec<TypeValue>>,
    pub size: Option<i16>,  // ONLY USED IN ARRAYS
}
impl Symbol {
    pub fn new(
        Identifier: String,
        Type: Option<Types>,
        Is_Constant: Option<bool>,
        Address: Option<usize>,
        Value: Option<Vec<TypeValue>>,
        size: Option<i16>,
    ) -> Self {
        Symbol {
            Identifier,
            Type,
            Is_Constant,
            Address,
            Value,
            size,
        }
    }
}

// Insert a new symbol, allowing only one symbol per identifier
pub fn insert(symbolTable: &Lazy<Mutex<HashMap<String, ts::Symbol>>>, symbol: Symbol) -> Result<(), String> {
    if symbolTable.lock().unwrap().contains_key(&symbol.Identifier) {
        return Err(format!("Duplicate identifier '{}'", symbol.Identifier));
    }
    symbolTable.lock().unwrap().insert(symbol.Identifier.clone(), symbol);
    Ok(())
}

pub fn update(symbolTable: &Lazy<Mutex<HashMap<String, ts::Symbol>>>, identifier: &str, value: &Vec<TypeValue>) -> Result<(), String> {
    if let Some(symbol) = symbolTable.lock().unwrap().get_mut(identifier) {
        symbol.Value = Some(value.clone());
        Ok(())
    } else {
        Err(format!("Symbol '{}' not found in the table", identifier))
    }
}

// Remove a symbol by its identifier
pub fn remove(symbolTable: &Lazy<Mutex<HashMap<String, ts::Symbol>>>, identifier: &str) -> Option<Symbol> {
    symbolTable.lock().unwrap().remove(identifier)
}

impl std::fmt::Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {

        // Format each field, truncating or padding as needed
        let identifier = format!("{:.17}", self.Identifier);
        let type_str = match &self.Type {
            None => "N/A",
            Some(t) => match t {
                Types::Integer => "INTEGER",
                Types::Float => "FLOAT",
                Types::Char => "CHAR",
                _ => ""
            }
        }.chars().take(17).collect::<String>();

        let size_str = self.size.map_or("N/A".to_string(), |s| format!("{}", s)).chars().take(17).collect::<String>();
        let constant_str = self.Is_Constant.map_or("N/A".to_string(), |c| format!("{}", c)).chars().take(17).collect::<String>();
        let address_str = self.Address.map_or("N/A".to_string(), |a| format!("{}", a)).chars().take(17).collect::<String>();
        let value_str = self.Value.as_ref().map_or("N/A".to_string(), |v| format!("{:?}", v)).chars().take(17).collect::<String>();
        let empty_string = "".to_string();
        let mut value_arr: Vec<String> = vec![];

        match &self.Value {
            None => value_arr.push("N/A".to_string()),
            Some(vec) => {
                let mut string = "".to_string();
                for value in vec {
                    let element: String;
                    match value {
                        TypeValue::Integer(i) => element = format!("{}, ", i),
                        TypeValue::Float(f) => element = format!("{}, ", f),
                        TypeValue::Char(c) => { 
                            if c.clone() as u16 == 0 {
                                element = "'\\0', ".to_string()
                            }
                            else { 
                                element = format!("'{}', ", c)
                            }
                        },
                        _ => element = "".to_string(),
                    }
                    if string.len() + element.len() > 17 {
                        value_arr.push(string);
                        string = element.clone();
                    }
                    else {
                        string += element.as_str();
                    }
                }
                value_arr.push(string[..(string.len() - 2)].to_string());
            }
        }

        // Write the row with formatted data
        writeln!(f, "| {:<17} | {:<17} | {:<17} | {:<17} | {:<17} | {:<17} |",
                 identifier, type_str, size_str, constant_str, address_str, value_arr[0])?;
        for line in &value_arr[1..] {
                writeln!(f, "| {:<17} | {:<17} | {:<17} | {:<17} | {:<17} | {:<17} |",
                         empty_string, empty_string, empty_string, empty_string, empty_string, line)?;
        }

        Ok(())
    }
}

// Update print_table function to improve readability
pub fn print_table(symbolTable: &Lazy<Mutex<HashMap<String, ts::Symbol>>>) {
    println!("\nSymbol Table Contents:");
    let border = "+-------------------+-------------------+-------------------+-------------------+-------------------+-------------------+";
    let headers = "| Identifier        | Type              | Size              | Constant          | Address           | Value             |";

    println!("{}", border);
    println!("{}", headers);
    println!("{}", border);
    for (key, value) in symbolTable.lock().unwrap().iter() {
        print!("{}", value);
        println!("{}", border);
    }
}