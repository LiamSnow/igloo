use litrs::{FloatLit, IntegerLit, ParseError, StringLit};
use logos::Logos;
use std::fmt::{self, Debug};
use std::mem::discriminant;
use std::num::ParseIntError;
use std::str::Chars;

#[derive(Default, Debug, Clone)]
pub enum LexicalError {
    Literal(ParseError),
    ParseInt(ParseIntError),
    #[default]
    InvalidToken,
}

#[derive(Logos, Clone, Debug, PartialEq)]
#[logos(skip r"#.*\n?", skip r"//[^\n]*\n?", skip r"/\*(?:[^*]|\*[^/])*\*/")]
#[logos(skip r"[ \t\n\f]+")]
#[logos(error = LexicalError)]
pub enum Token<'a> {
    #[regex(r#"[\p{XID_Start}_]\p{XID_Continue}*"#, |lex| lex.slice())]
    Ident(&'a str),
    #[regex(
        r"0[bB][01][01_]*|0[oO][0-7][0-7_]*|0[xX][0-9a-fA-F][0-9a-fA-F_]*|[0-9][0-9_]*",
        parse_int,
        priority = 3
    )]
    Int(IntegerLit<&'a str>),
    #[regex(
        r"[0-9][0-9_]*\.[0-9][0-9_]*([eE][+-]?[0-9][0-9_]*)?|[0-9][0-9_]*[eE][+-]?[0-9][0-9_]*",
        parse_float
    )]
    Float(FloatLit<&'a str>),
    #[token("false", |_| false)]
    #[token("true", |_| true)]
    Bool(bool),
    #[regex(r#""((?:[^"\\]|\\.)*)""#, parse_string)]
    #[regex(r#"r"([^"]*)""#, parse_string)]
    #[regex(r##"r#"##, parse_raw_string)]
    String(StringLit<&'a str>),

    #[token("let")]
    Let,
    #[token("const")]
    Const,
    #[token("while")]
    While,
    #[token("for")]
    For,
    #[token("in")]
    In,
    #[token("if")]
    If,
    #[token("else")]
    Else,
    #[token("fn")]
    Fn,
    #[token("struct")]
    Struct,
    #[token("enum")]
    Enum,
    #[token("type")]
    Type,
    #[token("dashboard")]
    Dashboard,
    #[token("element")]
    Element,
    #[token("use")]
    Use,
    #[token("mod")]
    Mod,
    #[token("return")]
    Return,
    #[token("break")]
    Break,
    #[token("continue")]
    Continue,

    #[token("=")]
    Eq,

    #[token("(")]
    OpenParen,
    #[token(")")]
    CloseParen,
    #[token("[")]
    OpenBracket,
    #[token("]")]
    CloseBracket,
    #[token("{")]
    OpenBrace,
    #[token("}")]
    CloseBrace,

    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("%")]
    Percent,
    #[token("**")]
    StarStar,
    #[token("<<")]
    Shl,
    #[token(">>")]
    Shr,

    #[token("+=")]
    PlusEq,
    #[token("-=")]
    MinusEq,
    #[token("*=")]
    StarEq,
    #[token("/=")]
    SlashEq,
    #[token("%=")]
    PercentEq,
    #[token("**=")]
    StarStarEq,
    #[token("^=")]
    CarotEq,
    #[token("&=")]
    AndEq,
    #[token("|=")]
    OrEq,
    #[token("<<=")]
    ShlEq,
    #[token(">>=")]
    ShrEq,

    #[token("==")]
    EqEq,
    #[token("!=")]
    Neq,
    #[token("<")]
    Lt,
    #[token("<=")]
    Le,
    #[token(">")]
    Gt,
    #[token(">=")]
    Ge,

    #[token("&&")]
    AndAnd,
    #[token("||")]
    OrOr,
    #[token("^")]
    Caret,
    #[token("&")]
    And,
    #[token("|")]
    Or,

    #[token("as")]
    As,

    #[token("int")]
    IntType,
    #[token("float")]
    FloatType,
    #[token("bool")]
    BoolType,
    #[token("string")]
    StringType,

    #[token(";")]
    SemiColon,
    #[token("?")]
    Question,
    #[token("!")]
    Bang,
    #[token(":")]
    Colon,
    #[token("::")]
    ColonColon,
    #[token(",")]
    Comma,
    #[token(".")]
    Period,
    #[token("->")]
    RArrow,
    #[token("=>")]
    FatArrow,
    #[token("..")]
    DotDot,
    #[token("..=")]
    DotDotEq,
    #[token("~")]
    Tilde,
}

impl<'a> fmt::Display for Token<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

fn parse_int<'a>(lex: &mut logos::Lexer<'a, Token<'a>>) -> Result<IntegerLit<&'a str>, ParseError> {
    IntegerLit::parse(lex.slice())
}

fn parse_float<'a>(lex: &mut logos::Lexer<'a, Token<'a>>) -> Result<FloatLit<&'a str>, ParseError> {
    FloatLit::parse(lex.slice())
}

fn parse_string<'a>(
    lex: &mut logos::Lexer<'a, Token<'a>>,
) -> Result<StringLit<&'a str>, ParseError> {
    StringLit::parse(lex.slice())
}

fn parse_raw_string<'a>(
    lex: &mut logos::Lexer<'a, Token<'a>>,
) -> Result<StringLit<&'a str>, ParseError> {
    lex.bump(find_raw_string_len(lex.remainder().chars()));
    parse_string(lex)
}

fn find_raw_string_len(mut chars: Chars<'_>) -> usize {
    let mut i = 0;

    while Some('#') == chars.next() {
        i += 1;
    }
    let level = i + 1;

    while let Some(c) = chars.next() {
        i += 1;
        if c == '"' {
            let mut is_term = true;
            for _ in 0..level {
                i += 1;
                if Some('#') != chars.next() {
                    is_term = false;
                    break;
                }
            }

            if is_term {
                break;
            }
        }
    }

    i + 1
}

impl PartialEq for LexicalError {
    fn eq(&self, other: &Self) -> bool {
        use LexicalError::*;
        match (self, other) {
            (Literal(_), Literal(_)) => true,
            _ => discriminant(self) == discriminant(other),
        }
    }
}

impl From<ParseError> for LexicalError {
    fn from(err: litrs::ParseError) -> Self {
        LexicalError::Literal(err)
    }
}

impl From<ParseIntError> for LexicalError {
    fn from(err: ParseIntError) -> Self {
        LexicalError::ParseInt(err)
    }
}
