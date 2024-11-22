#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(nonstandard_style)]

use std::collections::HashMap;
use crate::lexer::Token;

// mod tests;
mod lexer;
mod error;
mod TS;
mod tests;

fn main(){
    let mut a  = HashMap::new();
    a.insert("key1", 4);
    println!("a = {}",a["key"]);
}
