use logos::{Logos, SpannedIter};

pub mod tokens;
use tokens::{LexicalError, Token};

pub type Spanned<Tok, Loc, Error> = Result<(Loc, Tok, Loc), Error>;

pub struct Lexer<'a> {
    token_stream: SpannedIter<'a, Token<'a>>,
}

impl<'a> Lexer<'a> {
    pub fn new(a: &'a str) -> Self {
        Self {
            token_stream: Token::lexer(a).spanned(),
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
