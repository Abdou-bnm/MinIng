use std::collections::HashMap;
use std::sync::Mutex;
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


// Implement Display for Symbol to improve debugging
// impl std::fmt::Display for Symbol {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(
//             f,
//             "\tIdentifier: \"{}\"\n\tType: {:?}\n\tSize: {:?}\n\tConstant: {:?}\n\tAddress: {:?}\n\tValue: {:?}\n",
//             self.Identifier, self.Type, self.size, self.Is_Constant, self.Address, self.Value
//         )
//     }
// }
impl std::fmt::Display for Symbol {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {

        let border = "+-------------------+-------------------+-------------------+-------------------+-------------------+-------------------+";
        let headers = "| Identifier        | Type              | Size              | Constant          | Address           | Value             |";

        // Write the top border
        writeln!(f, "{}", border)?;
        writeln!(f, "{}", headers)?;
        writeln!(f, "{}", border)?;

        // Format each field, truncating or padding as needed
        let identifier = format!("{:.17}", self.Identifier);
        let type_str = format!("{:?}", self.Type).chars().take(17).collect::<String>();
        let size_str = self.size.map_or("N/A".to_string(), |s| format!("{}", s)).chars().take(17).collect::<String>();
        let constant_str = self.Is_Constant.map_or("N/A".to_string(), |c| format!("{}", c)).chars().take(17).collect::<String>();
        let address_str = self.Address.map_or("N/A".to_string(), |a| format!("{}", a)).chars().take(17).collect::<String>();
        let value_str = self.Value.as_ref().map_or("N/A".to_string(), |v| format!("{:?}", v)).chars().take(17).collect::<String>();

        // Write the row with formatted data
        writeln!(f, "| {:<17} | {:<17} | {:<17} | {:<17} | {:<17} | {:<17} |",
                 identifier, type_str, size_str, constant_str, address_str, value_str)?;

        // Write the bottom border
        writeln!(f, "{}", border)?;

        Ok(())
    }
}

// Update print_table function to improve readability
pub fn print_table(symbolTable: &Lazy<Mutex<HashMap<String, ts::Symbol>>>) {
    println!("\nSymbol Table Contents:");
    for (key, value) in symbolTable.lock().unwrap().iter() {
        println!("{}", value);
    }
}