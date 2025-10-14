use litrs::{ByteLit, ByteStringLit, CharLit, FloatLit, IntegerLit, ParseError, StringLit};
use logos::Logos;
use std::borrow::Cow;
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
#[logos(skip r"\f+")]
#[logos(skip r"//[^\n]*\n?", skip r"/\*(?:[^*]|\*[^/])*\*/")]
#[logos(error = LexicalError)]
pub enum Token<'source> {
    #[regex(r"\r?\n")]
    NewLine,
    #[regex(r"[ \t]+")]
    Whitespace,

    #[regex(r#"[\p{XID_Start}_]\p{XID_Continue}*"#, |lex| lex.slice())]
    Ident(&'source str),
    #[regex(
        "(0b|0o|0x)?[0-9a-fA-F][0-9a-fA-F_]*([iuf](8|16|32|64|128|size))?",
        parse_int,
        priority = 3
    )]
    Int(IntegerLit<&'source str>),
    #[regex(
        r"[0-9][0-9_]*(\.[0-9][0-9_]*)?([eE][+-]?[0-9][0-9_]*)?(f32|f64)?",
        parse_float
    )]
    Float(FloatLit<&'source str>),
    #[token("false", |_| false)]
    #[token("true", |_| true)]
    Bool(bool),
    #[regex(r"'((?:[^'\\]|\\.)+)'", parse_char)]
    Char(char),
    #[regex(r"b'((?:[^'\\]|\\.)+)'", parse_byte)]
    Byte(u8),
    #[regex(r#""((?:[^"\\]|\\.)*)""#, parse_string)]
    #[regex(r#"r"([^"]*)""#, parse_string)]
    #[regex(r##"r#"##, parse_raw_string)]
    String(Cow<'source, str>),
    #[regex(r#"b"((?:[^"\\]|\\.)*)""#, parse_byte_string)]
    #[regex(r#"br"((?:[^"\\]|\\.)*)""#, parse_byte_string)]
    #[regex(r##"br#"##, parse_raw_byte_string)]
    ByteString(Cow<'source, [u8]>),

    #[token("loop")]
    Loop,
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
    #[token("return")]
    Return,
    #[token("break")]
    Break,
    #[token("continue")]
    Continue,

    #[token("fn")]
    Fn,
    #[token("element")]
    Element,
    #[token("body:")]
    Body,

    #[token("const")]
    Const,
    #[token("var")]
    Var,

    #[token(r#"#include"#)]
    Include,

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

    #[token(";")]
    Semi,
    #[token(":")]
    Colon,
    #[token(",")]
    Comma,
    #[token(".")]
    Period,
    #[token("<-")]
    LArrow,
    #[token("->")]
    RArrow,
    #[token("..")]
    DotDot,
    #[token("..=")]
    DotDotEq,
    #[token(r#"#"#)]
    Pound,
    #[token("!")]
    Bang,
    #[token("?")]
    Question,

    #[token("as")]
    As,

    #[token("bool")]
    BoolType,
    #[token("string")]
    StringType,
    #[token("char")]
    CharType,

    #[token("i8")]
    I8,
    #[token("i16")]
    I16,
    #[token("i32")]
    I32,
    #[token("i64")]
    I64,
    #[token("i128")]
    I128,
    #[token("isize")]
    ISize,
    #[token("u8")]
    U8,
    #[token("u16")]
    U16,
    #[token("u32")]
    U32,
    #[token("u64")]
    U64,
    #[token("u128")]
    U128,
    #[token("usize")]
    USize,
    #[token("f32")]
    F32,
    #[token("f64")]
    F64,
}

impl<'source> fmt::Display for Token<'source> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

fn parse_int<'a, 'b>(
    lex: &'a mut logos::Lexer<'b, Token<'b>>,
) -> Result<IntegerLit<&'b str>, ParseError> {
    IntegerLit::parse(lex.slice())
}

fn parse_float<'a, 'b>(
    lex: &'a mut logos::Lexer<'b, Token<'b>>,
) -> Result<FloatLit<&'b str>, ParseError> {
    FloatLit::parse(lex.slice())
}

fn parse_char<'a, 'b>(lex: &'a mut logos::Lexer<'b, Token<'b>>) -> Result<char, ParseError> {
    Ok(CharLit::parse(lex.slice())?.value())
}

fn parse_byte<'a, 'b>(lex: &'a mut logos::Lexer<'b, Token<'b>>) -> Result<u8, ParseError> {
    Ok(ByteLit::parse(lex.slice())?.value())
}

fn parse_string<'a, 'b>(
    lex: &'a mut logos::Lexer<'b, Token<'b>>,
) -> Result<Cow<'b, str>, ParseError> {
    Ok(StringLit::parse(lex.slice())?.into_value())
}

fn parse_raw_string<'a, 'b>(
    lex: &'a mut logos::Lexer<'b, Token<'b>>,
) -> Result<Cow<'b, str>, ParseError> {
    lex.bump(find_raw_string_len(lex.remainder().chars()));
    parse_string(lex)
}

fn parse_byte_string<'a, 'b>(
    lex: &'a mut logos::Lexer<'b, Token<'b>>,
) -> Result<Cow<'b, [u8]>, ParseError> {
    Ok(ByteStringLit::parse(lex.slice())?.into_value())
}

fn parse_raw_byte_string<'a, 'b>(
    lex: &'a mut logos::Lexer<'b, Token<'b>>,
) -> Result<Cow<'b, [u8]>, ParseError> {
    lex.bump(find_raw_string_len(lex.remainder().chars()));
    parse_byte_string(lex)
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
