use bumpalo::Bump;
use logos::{Logos, SpannedIter};

pub mod fstring;
pub mod number;
pub mod string;
pub mod tokens;

use tokens::{LexerExtras, LexicalError, Token};

pub type Spanned<Tok, Loc, Error> = Result<(Loc, Tok, Loc), Error>;

pub struct Lexer<'a> {
    token_stream: SpannedIter<'a, Token<'a>>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str, arena: &'a Bump) -> Self {
        let mut logos_lexer = Token::lexer(input);
        logos_lexer.extras = LexerExtras { arena: Some(arena) };

        Self {
            token_stream: logos_lexer.spanned(),
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Spanned<Token<'a>, usize, LexicalError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.token_stream.next().map(|(token, span)| match token {
            Ok(tok) => Ok((span.start, tok, span.end)),
            Err(_) => Ok((span.start, Token::Error, span.end)),
        })
    }
}
