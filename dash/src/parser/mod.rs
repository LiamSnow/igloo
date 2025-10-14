pub mod ast;
pub mod lexer;
pub mod tokens;

use lalrpop_util::{ParseError, lalrpop_mod};
lalrpop_mod!(grammar);

use ast::GlobalStatement;
use grammar::MainParser;
use lexer::Lexer;
use tokens::{LexicalError, Token};

pub fn parse<'a>(
    source: &'a str,
) -> Result<Vec<GlobalStatement<'a>>, ParseError<usize, Token<'a>, LexicalError>> {
    let lexer = Lexer::new(source);
    let parser = MainParser::new();
    parser.parse(source, lexer)
}
