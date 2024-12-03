use std::collections::HashMap;
use std::sync::Mutex;
use lazy_static::lazy_static;
use std::sync::atomic::{AtomicBool};

// Global static flag (this can be adjusted or removed as needed)
pub static IB_FLAG: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Debug, PartialEq)] // PartialEq for equality comparisons
pub enum Types {
    Integer,
    Float,
    Char,
    Array(Box<Types>, i16), // Array with element type and size
}

#[derive(Clone, Debug, PartialEq)] // PartialEq for comparisons
pub enum TypeValue {
    Integer(i16),
    Float(f32),
    Char(char),
    Array(Vec<TypeValue>), // Array value representation
}

#[derive(Debug, Clone)]
pub struct Symbol {
    pub Identifier: String,
    pub Type: Option<Types>,
    pub Is_Constant: Option<bool>,
    pub Address: Option<usize>,
    pub Value: Option<TypeValue>,
}

pub struct SymbolTable {
    symbols: HashMap<String, Symbol>, // Change to hold single symbols for each identifier
    current_address: usize,
}

lazy_static! {
    static ref SYMBOL_TABLE: Mutex<SymbolTable> = Mutex::new(SymbolTable::new());
}

impl Symbol {
    pub fn new(
        Identifier: String,
        Type: Option<Types>,
        Is_Constant: Option<bool>,
        Address: Option<usize>,
        Value: Option<TypeValue>,
    ) -> Self {
        Symbol {
            Identifier,
            Type,
            Is_Constant,
            Address,
            Value,
        }
    }
}

impl SymbolTable {
    pub fn new() -> Self {
        SymbolTable {
            symbols: HashMap::new(),
            current_address: 0,
        }
    }

    // Singleton-like access to the global symbol table
    pub fn get_instance() -> &'static Mutex<SymbolTable> {
        &SYMBOL_TABLE
    }

    // Insert a new symbol, allowing only one symbol per identifier
    pub fn insert(&mut self, symbol: Symbol) -> Result<(), String> {
        if self.symbols.contains_key(&symbol.Identifier) {
            return Err(format!("Duplicate identifier '{}'", symbol.Identifier));
        }
        self.symbols.insert(symbol.Identifier.clone(), symbol);
        Ok(())
    }

    pub fn update(&mut self, identifier: &str, value: TypeValue) -> Result<(), String> {
        if let Some(symbol) = self.symbols.get_mut(identifier) {
            symbol.Value = Some(value);
            Ok(())
        } else {
            Err(format!("Symbol '{}' not found in the table", identifier))
        }
    }

    // Lookup a symbol by its identifier
    pub fn lookup(&self, identifier: &str) -> Option<&Symbol> {
        self.symbols.get(identifier)
    }

    // Remove a symbol by its identifier
    pub fn remove(&mut self, identifier: &str) -> Option<Symbol> {
        self.symbols.remove(identifier)
    }

    // Get the next available memory address
    pub fn get_next_address(&mut self) -> usize {
        let addr = self.current_address;
        self.current_address += 1;
        addr
    }

    // Print all symbols in the table
    pub fn print_table(&self) {
        println!("Symbol Table Contents:");
        for (identifier, symbol) in &self.symbols {
            println!("{}", symbol);
        }
    }
}

// Implement Display for Symbol to improve debugging
impl std::fmt::Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "\tIdentifier: \"{}\"\n\tType: {:?}\n\tConstant: {:?}\n\tAddress: {:?}\n\tValue: {:?}\n",
            self.Identifier, self.Type, self.Is_Constant, self.Address, self.Value
        )
    }
}

