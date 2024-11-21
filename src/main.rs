#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]
use crate::lexer::Token;

mod tests;
mod lexer;
mod error;
mod symbol_table;

fn main(){
    let mut a  = 5;

    {
        a= 6;
    }
    println!("a={}",a);
}
