pub mod ast;
#[cfg(test)]
mod test;

use bumpalo::Bump;
use lalrpop_util::{ParseError, lalrpop_mod};
lalrpop_mod!(grammar);

use crate::lexer::Lexer;
use crate::lexer::tokens::{LexicalError, Token};
use ast::Statement;
use grammar::MainParser;

pub fn parse<'a>(
    source: &'a str,
    arena: &'a Bump,
) -> Result<&'a [Statement<'a>], ParseError<usize, Token<'a>, LexicalError>> {
    let lexer = Lexer::new(source);
    let parser = MainParser::new();
    parser.parse(source, arena, lexer)
}
