use bumpalo::{Bump, collections::Vec};
use derive_more::Display;
use logos::Logos;
use std::fmt::Debug;

use crate::lexer::{fstring::*, number::*, string::*};

#[derive(Clone, Default)]
pub struct LexerExtras<'a> {
    // default is required, to we just unwrap this
    pub arena: Option<&'a Bump>,
}

#[derive(thiserror::Error, Default, Debug, Clone, PartialEq, Eq)]
pub enum LexicalError {
    #[default]
    #[error("Invalid token")]
    InvalidToken,
    #[error("Mistaken keyword '{0}'")]
    MistakenKeyword(String),

    #[error("Unterminated string")]
    UnterminatedString,
    #[error(r##"Unterminated raw string `r""` or `r#""#`"##)]
    UnterminatedRawString,
    #[error(r##"Raw hash string `r#""#` missing opening quote"##)]
    RawHashStringNoQuote,
    #[error(
        r"Incomplete escape sequence in string, \
        maybe you ended a string with a `\` accidentally?"
    )]
    ShortEscapeSequence,
    #[error("`\\{0}` is not a valid escape sequence")]
    InvalidEscapeSequence(char),
    #[error(
        r"Incomplete hex escape sequence (`\xHH`) in string, \
        maybe you ended a string with a `\x` accidentally?"
    )]
    ShortHexEscapeSequence,
    #[error(r"Hex escape sequence `\x{0:2X}` is too big. Must <= 0x7F")]
    HexEscapeSequenceTooBig(u8),
    #[error("Invalid hex digit in escape sequence")]
    InvalidHexDigit,
    #[error(r"Unicode escape sequence `\u{{...}}` missing closing brace")]
    UnicodeEscapeMissingCloseBrace,
    #[error(r"Unicode escape sequence `\u{{}}` cannot be empty")]
    EmptyUnicodeEscape,
    #[error("Invalid unicode code point: 0x{0:X}")]
    InvalidUnicodeCodePoint(u32),
    #[error("Hex number too large in unicode escape")]
    UnicodeEscapeOverflow,

    #[error("Unexpected closing brace `}}` in f-string")]
    FStringUnexpectedClosingBrace,
    #[error("Expected closing brace `}}` in f-string")]
    FStringMissingClosingBrace,
    #[error("Empty precision in format spec (expected digits after '.')")]
    FStringEmptyPrecision,
    #[error("Invalid type specifier '{0}' in format spec")]
    FStringInvalidTypeSpec(char),
    #[error("Unexpected character '{0}' in format spec")]
    FStringUnexpectedChar(char),

    #[error("Integer overflow")]
    IntegerOverflow,
    #[error("Invalid digit '{digit}' in base {base} number (expected digits 0-{max})")]
    InvalidDigit { digit: char, base: u32, max: char },
    #[error("Expected digits after number prefix")]
    NoDigitsAfterPrefix,
    #[error("Float overflow")]
    FloatOverflow,
    #[error("Missing exponent after 'e' or 'E'")]
    MissingExponent,
    #[error("Exponent overflow")]
    ExponentOverflow,
}

#[derive(Logos, Clone, Debug, PartialEq, Display)]
#[logos(skip r"//[^\n]*\n?", skip r"/\*(?:[^*]|\*[^/])*\*/")]
#[logos(skip r"[ \t\n\f]+")]
#[logos(error = LexicalError)]
#[logos(extras = LexerExtras<'s>)]
pub enum Token<'a> {
    #[display("identifier '{_0}'")]
    #[regex(r#"[\p{XID_Start}_]\p{XID_Continue}*"#, |lex| lex.slice())]
    Ident(&'a str),

    #[display("{_0}")]
    #[regex(r"[0-9]", parse_number)]
    Number(Number),

    #[display("{_0}")]
    #[token("false", |_| false)]
    #[token("true", |_| true)]
    Bool(bool),

    #[display("string {_0}")]
    #[token("\"", parse_string)]
    #[token("r\"", parse_raw_string)]
    #[token("r#", parse_raw_hash_string)]
    String(&'a str),

    #[display("fstring `{_0:?}`")]
    #[token("f\"", parse_fstring)]
    #[token("rf\"", parse_raw_fstring)]
    #[token("fr\"", parse_raw_fstring)]
    #[token("rf#", parse_raw_hash_fstring)]
    #[token("fr#", parse_raw_hash_fstring)]
    FString(Vec<'a, Token<'a>>),
    #[display("{{")]
    FStringInterpStart,
    #[display("}}")]
    FStringInterpEnd,
    #[display(":{_0}")]
    FStringFormat(FStringFormat),

    #[display("'let'")]
    #[token("let")]
    Let,

    #[display("'const'")]
    #[token("const")]
    Const,

    #[display("'while'")]
    #[token("while")]
    While,

    #[display("'for'")]
    #[token("for")]
    For,

    #[display("'in'")]
    #[token("in")]
    In,

    #[display("'if'")]
    #[token("if")]
    If,

    #[display("'else'")]
    #[token("else")]
    Else,

    #[display("'fn'")]
    #[token("fn")]
    Fn,

    #[display("'struct'")]
    #[token("struct")]
    Struct,

    #[display("'enum'")]
    #[token("enum")]
    Enum,

    #[display("'type'")]
    #[token("type")]
    Type,

    #[display("'dashboard'")]
    #[token("dashboard")]
    Dashboard,

    #[display("'element'")]
    #[token("element")]
    Element,

    #[display("'use'")]
    #[token("use")]
    Use,

    #[display("'mod'")]
    #[token("mod")]
    Mod,

    #[display("'return'")]
    #[token("return")]
    Return,

    #[display("'break'")]
    #[token("break")]
    Break,

    #[display("'continue'")]
    #[token("continue")]
    Continue,

    #[display("'Bind'")]
    #[token("Bind")]
    Bind,
    #[display("'Observe'")]
    #[token("Observe")]
    Observe,
    #[display("'FilterSet'")]
    #[token("FilterSet")]
    FilterSet,
    #[display("'where'")]
    #[token("where")]
    Where,

    #[display("'='")]
    #[token("=")]
    Eq,

    #[display("'('")]
    #[token("(")]
    OpenParen,

    #[display("')'")]
    #[token(")")]
    CloseParen,

    #[display("'['")]
    #[token("[")]
    OpenBracket,

    #[display("']'")]
    #[token("]")]
    CloseBracket,

    #[display("'{{'")]
    #[token("{")]
    OpenBrace,

    #[display("'}}'")]
    #[token("}")]
    CloseBrace,

    #[display("'+'")]
    #[token("+")]
    Plus,

    #[display("'-'")]
    #[token("-")]
    Minus,

    #[display("'*'")]
    #[token("*")]
    Star,

    #[display("'/'")]
    #[token("/")]
    Slash,

    #[display("'%'")]
    #[token("%")]
    Percent,

    #[display("'**'")]
    #[token("**")]
    StarStar,

    #[display("'<<'")]
    #[token("<<")]
    Shl,

    #[display("'>>'")]
    #[token(">>")]
    Shr,

    #[display("'+='")]
    #[token("+=")]
    PlusEq,

    #[display("'-='")]
    #[token("-=")]
    MinusEq,

    #[display("'*='")]
    #[token("*=")]
    StarEq,

    #[display("'/='")]
    #[token("/=")]
    SlashEq,

    #[display("'%='")]
    #[token("%=")]
    PercentEq,

    #[display("'**='")]
    #[token("**=")]
    StarStarEq,

    #[display("'^='")]
    #[token("^=")]
    CarotEq,

    #[display("'&='")]
    #[token("&=")]
    AndEq,

    #[display("'|='")]
    #[token("|=")]
    OrEq,

    #[display("'<<='")]
    #[token("<<=")]
    ShlEq,

    #[display("'>>='")]
    #[token(">>=")]
    ShrEq,

    #[display("'=='")]
    #[token("==")]
    EqEq,

    #[display("'!='")]
    #[token("!=")]
    Neq,

    #[display("'<'")]
    #[token("<")]
    Lt,

    #[display("'<='")]
    #[token("<=")]
    Le,

    #[display("'>'")]
    #[token(">")]
    Gt,

    #[display("'>='")]
    #[token(">=")]
    Ge,

    #[display("'&&'")]
    #[token("&&")]
    AndAnd,

    #[display("'||'")]
    #[token("||")]
    OrOr,

    #[display("'^'")]
    #[token("^")]
    Caret,

    #[display("'&'")]
    #[token("&")]
    And,

    #[display("'|'")]
    #[token("|")]
    Or,

    #[display("'as'")]
    #[token("as")]
    As,

    #[display("'int'")]
    #[token("int")]
    IntType,

    #[display("'float'")]
    #[token("float")]
    FloatType,

    #[display("'bool'")]
    #[token("bool")]
    BoolType,

    #[display("'string'")]
    #[token("string")]
    StringType,

    #[display("';'")]
    #[token(";")]
    SemiColon,

    #[display("'?'")]
    #[token("?")]
    Question,

    #[display("'!'")]
    #[token("!")]
    Bang,

    #[display("':'")]
    #[token(":")]
    Colon,

    #[display("'::'")]
    #[token("::")]
    ColonColon,

    #[display("','")]
    #[token(",")]
    Comma,

    #[display("'.'")]
    #[token(".")]
    Period,

    #[display("'->'")]
    #[token("->")]
    RArrow,

    #[display("'=>'")]
    #[token("=>")]
    FatArrow,

    #[display("'..'")]
    #[token("..")]
    DotDot,

    #[display("'..='")]
    #[token("..=")]
    DotDotEq,

    #[display("mistaken keyword '{_0}'")]
    #[regex(r"(var|function|func|def|elif|class|interface|null|nil|None|undefined|void|self|this|mut|async|await|pass|unless|switch|match|case|True|False|foreach|do|loop|until|repeat|auto|dynamic|number|str|public|private|protected|pub|static|try|catch|finally|except|throw|raise|and|or|not|is|new|import|require|export|namespace|package|goto|super|extends|implements)")]
    MistakenKeyword(&'a str),

    #[display("invalid character")]
    Error,
}

#[derive(Clone, Debug, PartialEq, Display)]
pub enum Number {
    #[display("float {_0}")]
    Float(f64),
    #[display("int {_0}")]
    Int(i64),
}

impl<'a> Token<'a> {
    pub fn is_value(&self) -> bool {
        matches!(
            self,
            Token::Ident(_) | Token::Number(_) | Token::Bool(_) | Token::String(_)
        )
    }

    pub fn is_operator(&self) -> bool {
        matches!(
            self,
            Token::Plus
                | Token::Minus
                | Token::Star
                | Token::Slash
                | Token::Percent
                | Token::StarStar
                | Token::Shl
                | Token::Shr
                | Token::EqEq
                | Token::Neq
                | Token::Lt
                | Token::Le
                | Token::Gt
                | Token::Ge
                | Token::AndAnd
                | Token::OrOr
                | Token::Caret
                | Token::And
                | Token::Or
        )
    }

    pub fn is_separator(&self) -> bool {
        matches!(
            self,
            Token::SemiColon | Token::Comma | Token::CloseParen | Token::CloseBrace
        )
    }

    pub fn is_keyword(&self) -> bool {
        matches!(
            self,
            Token::Let
                | Token::Const
                | Token::While
                | Token::For
                | Token::In
                | Token::If
                | Token::Else
                | Token::Fn
                | Token::Struct
                | Token::Enum
                | Token::Type
                | Token::Dashboard
                | Token::Element
                | Token::Use
                | Token::Mod
                | Token::Return
                | Token::Break
                | Token::Continue
        )
    }
}
