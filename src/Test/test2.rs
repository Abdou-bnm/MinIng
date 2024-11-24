#[cfg(test)]
mod Testing {
    use crate::Lexer::lexer::{Keyword, Token,SymbolTable};
    use crate::error::CustomError;
    use crate::Parser::*;
    use crate::Semantic::ts::*;
    use lalrpop_util::lalrpop_mod;
    lalrpop_mod!(pub grammar);
    #[test]
    fn little_test(){
        let expr =  grammar::ProgramParser::new();
    }
}