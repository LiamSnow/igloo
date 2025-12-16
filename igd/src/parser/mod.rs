pub mod ast;
#[cfg(test)]
mod test;

use bumpalo::Bump;
use lalrpop_util::lalrpop_mod;
lalrpop_mod!(grammar);

use crate::error::IgdError;
use crate::lexer::Lexer;
use ast::Statement;
use grammar::MainParser;

pub fn parse<'a>(source: &'a str, arena: &'a Bump) -> Result<&'a [Statement<'a>], IgdError> {
    let lexer = Lexer::new(source, arena);
    let parser = MainParser::new();
    let mut errors = Vec::new();

    match parser.parse(source, arena, &mut errors, lexer) {
        Ok(ast) => {
            if !errors.is_empty() {
                // minor error
                Err(IgdError::from_error_recovery(errors, source))
            } else {
                Ok(ast)
            }
        }
        // fatal error
        Err(e) => Err(IgdError::from_lalrpop_error(e, source)),
    }
}
