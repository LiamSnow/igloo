use derive_more::Display;
use litrs::{FloatLit, IntegerLit, ParseError, StringLit};
use logos::Logos;
use std::fmt::Debug;
use std::str::Chars;

#[derive(thiserror::Error, Default, Debug, Clone)]
pub enum LexicalError {
    #[error("Error parsing literal: {0}")]
    Literal(#[from] ParseError),
    #[default]
    #[error("Invalid token")]
    InvalidToken,
    #[error("Mistaken keyword '{0}'")]
    MistakenKeyword(String),
}

#[derive(Logos, Clone, Debug, PartialEq, Display)]
#[logos(skip r"//[^\n]*\n?", skip r"/\*(?:[^*]|\*[^/])*\*/")]
#[logos(skip r"[ \t\n\f]+")]
#[logos(error = LexicalError)]
pub enum Token<'a> {
    #[display("identifier '{}'", _0)]
    #[regex(r#"[\p{XID_Start}_]\p{XID_Continue}*"#, |lex| lex.slice())]
    Ident(&'a str),

    #[display("integer {}", _0)]
    #[regex(
        r"0[bB][01][01_]*|0[oO][0-7][0-7_]*|0[xX][0-9a-fA-F][0-9a-fA-F_]*|[0-9][0-9_]*",
        parse_int,
        priority = 3
    )]
    Int(IntegerLit<&'a str>),

    #[display("float {}", _0)]
    #[regex(
        r"[0-9][0-9_]*\.[0-9][0-9_]*([eE][+-]?[0-9][0-9_]*)?|[0-9][0-9_]*[eE][+-]?[0-9][0-9_]*",
        parse_float
    )]
    Float(FloatLit<&'a str>),

    #[display("boolean {}", _0)]
    #[token("false", |_| false)]
    #[token("true", |_| true)]
    Bool(bool),

    #[display("string {}", _0)]
    #[regex(r#""((?:[^"\\]|\\.)*)""#, parse_string)]
    #[regex(r#"r"([^"]*)""#, parse_string)]
    #[regex(r##"r#"##, parse_raw_string)]
    String(StringLit<&'a str>),

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

    #[display("'~'")]
    #[token("~")]
    Tilde,

    #[display("mistaken keyword '{_0}'")]
    #[regex(r"(var|function|func|def|elif|class|interface|null|nil|None|undefined|void|self|this|mut|async|await|pass|unless|switch|match|case|True|False|foreach|do|loop|until|repeat|auto|dynamic|number|str|public|private|protected|pub|static|try|catch|finally|except|throw|raise|and|or|not|is|new|import|require|export|namespace|package|goto|super|extends|implements)")]
    MistakenKeyword(&'a str),

    #[display("invalid character")]
    Error,
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
            (Literal(a), Literal(b)) => a.span() == b.span(),
            (InvalidToken, InvalidToken) => true,
            _ => false,
        }
    }
}

impl<'a> Token<'a> {
    pub fn is_value(&self) -> bool {
        matches!(
            self,
            Token::Ident(_) | Token::Int(_) | Token::Float(_) | Token::Bool(_) | Token::String(_)
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
