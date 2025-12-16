//! Feature complete Python F-Strings, except for [#unsupported_features] and [#extra_features]
//!
//! The F-String is made up of two core parts:
//!  1. *Strings*: The normal string parse
//!      - handled just like string::parse_string()
//!  2. *Interpolations* (Replacement Fields)
//!
//! # Interpolation Syntax
//! ::= `"{" <Expr> (":" <Format>)? "}"`
//!
//! `Expr`: Any expression, ex `1+2`
//!
//! `Format`: See [FStringFormat]
//!
//!
//! # Unsupported Features
//!  - Conversion flags (`!r`, `!s`, `!a`)
//!  - Datetime formatting
//!  - Replacements in format section
//!  - Python 3.8 `=` flat (expression printing)
//!  - Comments in interpolations
//!
//! # Extra Features
//!  - Escape sequences in interpolation blocks
//!

use crate::lexer::string::{StringParser, check_closing_hashes};
use crate::lexer::tokens::{LexicalError, Token};
use bumpalo::collections::Vec;
use derive_more::Display;
use logos::Lexer;
use logos::Logos;

/// Struct is outlined in order of format spec. All fields optional.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct FStringFormat {
    /// defaults to space
    pub fill_char: Option<char>,
    pub align: Option<Align>,
    pub sign: Sign,
    /// `#`
    /// adds prefixes (`0b`, `0x`, `0X`) for binary/hex/octal
    /// guarantees a decimal point for floats
    pub alt_form: bool,
    /// `0`
    /// equiv. to `fill_char=0` + `align=AfterSign`
    pub zero_pad: bool,
    /// minimum total field width
    pub width: Option<u32>,
    /// thousands separator
    pub grouping: Option<Grouping>,
    /// `.<Integer>`
    /// floats: # of digits after decimal place
    /// strings: max # of characters
    pub precision: Option<u32>,
    pub type_spec: Option<TypeSpec>,
}

#[derive(Clone, Copy, Debug, PartialEq, Display)]
pub enum Align {
    /// default for strings
    #[display("<")]
    Left,
    /// default for numbers
    #[display(">")]
    Right,
    #[display("^")]
    Center,
    /// padding after sign, before digits (numbers only)
    #[display("=")]
    AfterSign,
}

#[derive(Clone, Copy, Debug, PartialEq, Default, Display)]
pub enum Sign {
    /// show sign for positive and negative numbers
    #[display("+")]
    Always,
    /// only show sign on negative numbers
    #[default]
    #[display("-")]
    OnlyNegative,
    /// space for positive, minus for negative
    #[display(" ")]
    SpacePositiveMinusNegative,
}

#[derive(Clone, Copy, Debug, PartialEq, Display)]
pub enum Grouping {
    #[display(",")]
    Comma,
    #[display("_")]
    Underscore,
}

#[derive(Clone, Copy, Debug, PartialEq, Display)]
pub enum TypeSpec {
    // string
    #[display("s")]
    String,

    // int
    #[display("b")]
    Binary,
    #[display("c")]
    Character,
    #[display("d")]
    Decimal,
    #[display("o")]
    Octal,
    #[display("x")]
    HexLower,
    #[display("X")]
    HexUpper,

    // float
    #[display("e")]
    ExpLower,
    #[display("E")]
    ExpUpper,
    #[display("f")]
    FixedLower,
    #[display("F")]
    FixedUpper,
    #[display("g")]
    GeneralLower,
    #[display("G")]
    GeneralUpper,
    #[display("%")]
    Percentage,

    // float or int
    #[display("n")]
    Number,
}

pub fn parse_fstring<'a>(
    lex: &mut Lexer<'a, Token<'a>>,
) -> Result<Vec<'a, Token<'a>>, LexicalError> {
    parse_fstring_impl(lex, StringParser::default(), 0)
}

pub fn parse_raw_fstring<'a>(
    lex: &mut Lexer<'a, Token<'a>>,
) -> Result<Vec<'a, Token<'a>>, LexicalError> {
    let parser = StringParser {
        is_raw: true,
        ..Default::default()
    };
    parse_fstring_impl(lex, parser, 0)
}

pub fn parse_raw_hash_fstring<'a>(
    lex: &mut Lexer<'a, Token<'a>>,
) -> Result<Vec<'a, Token<'a>>, LexicalError> {
    let remainder = lex.remainder();

    // already matched rf# or fr# -> start at 1
    let num_hashes = 1 + remainder.bytes().take_while(|&b| b == b'#').count();

    // verify opening quote exists after the hashes
    if remainder.as_bytes().get(num_hashes - 1) != Some(&b'"') {
        return Err(LexicalError::RawHashStringNoQuote);
    }

    let parser = StringParser {
        is_raw: true,
        i: num_hashes,
        last_copy: num_hashes,
    };

    parse_fstring_impl(lex, parser, num_hashes)
}

/// See docs at top of file
///
/// This is a mix of a parser and lexer:
///  - Interpolation block and Expressions are only lexed (parsing handled by LALRPOP)
///  - Format block is parsed
///
/// # Output
///  - For each interp block, the braces are consumed and instead FStringInterp(Start|End) is emitted
///  - For the format section, the colon and remainined chars are consumed and FStringFormat is emitted
///
/// ## Examples
/// "there is {var} lines" => [::String("there is "), ::FStringInterpStart, ::Ident("var"), ::FStringInterpEnd, ::String(" lines")]
///
/// "{var:.2f}" => [
///     ::FStringInterpStart,
///     ::Ident("var"),
///     ::FStringFormat(FStringFormat {
///         precision: Some(2),
///         type_spec: Some(TypeSpec::FixedLower),
///         ..Default::default()
///     }),
///     ::FStringInterpEnd
/// ]
///
fn parse_fstring_impl<'a>(
    lex: &mut Lexer<'a, Token<'a>>,
    mut parser: StringParser,
    num_hashes: usize,
) -> Result<Vec<'a, Token<'a>>, LexicalError> {
    let mut toks = Vec::with_capacity_in(10, lex.extras.arena.unwrap());
    let remainder = lex.remainder();
    let mut string_buffer: Option<String> = None;

    while parser.i < remainder.len() {
        if parser.handle_char(remainder, &mut string_buffer)? {
            continue;
        }

        match remainder.as_bytes()[parser.i] {
            // escaped (`{{`)
            b'{' if remainder.as_bytes().get(parser.i + 1) == Some(&b'{') => {
                let buf = ensure_buffer(&mut string_buffer);
                buf.push_str(&remainder[parser.last_copy..parser.i]);
                buf.push('{');
                parser.i += 2;
                parser.last_copy = parser.i;
            }
            b'{' => {
                flush_fstring_buffer(
                    &mut toks,
                    &mut string_buffer,
                    &remainder[parser.last_copy..parser.i],
                    lex,
                );

                parser.i += 1;
                toks.push(Token::FStringInterpStart);

                let consumed = parse_interpolation(&mut toks, &remainder[parser.i..], lex)?;
                parser.i += consumed;

                parser.last_copy = parser.i;
                string_buffer = None;
            }

            // escaped (`}}`)
            b'}' if remainder.as_bytes().get(parser.i + 1) == Some(&b'}') => {
                let buf = ensure_buffer(&mut string_buffer);
                buf.push_str(&remainder[parser.last_copy..parser.i]);
                buf.push('}');
                parser.i += 2;
                parser.last_copy = parser.i;
            }
            // shouldn't have `}` here
            // if valid, would've been in parse_interpolation
            b'}' => {
                return Err(LexicalError::FStringUnexpectedClosingBrace);
            }

            b'"' => {
                if num_hashes > 0 && !check_closing_hashes(&remainder[parser.i + 1..], num_hashes) {
                    parser.i += 1;
                    continue;
                }

                flush_fstring_buffer(
                    &mut toks,
                    &mut string_buffer,
                    &remainder[parser.last_copy..parser.i],
                    lex,
                );
                lex.bump(parser.i + 1 + num_hashes);
                return Ok(toks);
            }
            _ => parser.i += 1,
        }
    }

    Err(LexicalError::UnterminatedString)
}

fn parse_interpolation<'a>(
    toks: &mut Vec<'a, Token<'a>>,
    input: &'a str,
    parent_lex: &Lexer<'a, Token<'a>>,
) -> Result<usize, LexicalError> {
    let mut depth = 1;
    // track ternaries
    // each time we see a ? -> skip next :
    // so we don't think its the format section
    let mut ternary_depth = 0;
    let mut inner_lex = Token::lexer(input);
    inner_lex.extras = parent_lex.extras.clone();

    while let Some(result) = inner_lex.next() {
        let span = inner_lex.span();

        match result {
            Ok(Token::OpenBrace) => {
                depth += 1;
                toks.push(Token::OpenBrace);
            }
            Ok(Token::CloseBrace) => {
                depth -= 1;
                if depth == 0 {
                    toks.push(Token::FStringInterpEnd);
                    return Ok(span.end);
                }
                toks.push(Token::CloseBrace);
            }
            Ok(Token::Question) => {
                ternary_depth += 1;
                toks.push(Token::Question);
            }
            Ok(Token::Colon) if depth == 1 && ternary_depth == 0 => {
                let format_start = span.end;

                let (format_str, closing_pos) = find_closing_brace(&input[format_start..])?;
                let format = parse_format_spec(format_str)?;
                toks.push(Token::FStringFormat(format));
                toks.push(Token::FStringInterpEnd);

                return Ok(format_start + closing_pos + 1);
            }
            Ok(Token::Colon) => {
                if ternary_depth > 0 {
                    ternary_depth -= 1;
                }
                toks.push(Token::Colon);
            }
            Ok(tok) => {
                toks.push(tok);
            }
            Err(e) => {
                return Err(e);
            }
        }
    }

    Err(LexicalError::FStringMissingClosingBrace)
}

fn find_closing_brace(input: &str) -> Result<(&str, usize), LexicalError> {
    // format section doesn't contain braces -> just find first `}`
    if let Some(pos) = input.bytes().position(|b| b == b'}') {
        Ok((&input[..pos], pos))
    } else {
        Err(LexicalError::FStringMissingClosingBrace)
    }
}

fn parse_format_spec(input: &str) -> Result<FStringFormat, LexicalError> {
    let mut fmt = FStringFormat::default();
    let bytes = input.as_bytes();
    let mut i = 0;

    if i >= bytes.len() {
        return Ok(fmt);
    }

    // lookahead for align, if found ->  we know char is fill
    if i + 1 < bytes.len() {
        let is_align = matches!(bytes[i + 1], b'<' | b'>' | b'^' | b'=');
        if is_align {
            fmt.fill_char = Some(bytes[i] as char);
            i += 1;
        }
    }

    if i < bytes.len() {
        fmt.align = match bytes[i] {
            b'<' => {
                i += 1;
                Some(Align::Left)
            }
            b'>' => {
                i += 1;
                Some(Align::Right)
            }
            b'^' => {
                i += 1;
                Some(Align::Center)
            }
            b'=' => {
                i += 1;
                Some(Align::AfterSign)
            }
            _ => None,
        };
    }

    if i < bytes.len() {
        match bytes[i] {
            b'+' => {
                fmt.sign = Sign::Always;
                i += 1;
            }
            b'-' => {
                fmt.sign = Sign::OnlyNegative;
                i += 1;
            }
            b' ' => {
                fmt.sign = Sign::SpacePositiveMinusNegative;
                i += 1;
            }
            _ => {}
        }
    }

    if i < bytes.len() && bytes[i] == b'#' {
        fmt.alt_form = true;
        i += 1;
    }

    if i < bytes.len() && bytes[i] == b'0' {
        fmt.zero_pad = true;
        i += 1;
    }

    if i < bytes.len() && bytes[i].is_ascii_digit() {
        let start = i;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
        let width_str = std::str::from_utf8(&bytes[start..i]).unwrap();
        fmt.width = Some(width_str.parse().unwrap());
    }

    if i < bytes.len() {
        match bytes[i] {
            b',' => {
                fmt.grouping = Some(Grouping::Comma);
                i += 1;
            }
            b'_' => {
                fmt.grouping = Some(Grouping::Underscore);
                i += 1;
            }
            _ => {}
        }
    }

    if i < bytes.len() && bytes[i] == b'.' {
        i += 1;
        let start = i;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
        if start == i {
            return Err(LexicalError::FStringEmptyPrecision);
        }
        let precision_str = std::str::from_utf8(&bytes[start..i]).unwrap();
        fmt.precision = Some(precision_str.parse().unwrap());
    }

    if i < bytes.len() {
        fmt.type_spec = Some(match bytes[i] {
            b's' => TypeSpec::String,
            b'b' => TypeSpec::Binary,
            b'c' => TypeSpec::Character,
            b'd' => TypeSpec::Decimal,
            b'o' => TypeSpec::Octal,
            b'x' => TypeSpec::HexLower,
            b'X' => TypeSpec::HexUpper,
            b'e' => TypeSpec::ExpLower,
            b'E' => TypeSpec::ExpUpper,
            b'f' => TypeSpec::FixedLower,
            b'F' => TypeSpec::FixedUpper,
            b'g' => TypeSpec::GeneralLower,
            b'G' => TypeSpec::GeneralUpper,
            b'%' => TypeSpec::Percentage,
            b'n' => TypeSpec::Number,
            c => return Err(LexicalError::FStringInvalidTypeSpec(c as char)),
        });
        i += 1;
    }

    if i != bytes.len() {
        return Err(LexicalError::FStringUnexpectedChar(bytes[i] as char));
    }

    Ok(fmt)
}

fn ensure_buffer(buffer: &mut Option<String>) -> &mut String {
    buffer.get_or_insert_with(|| String::with_capacity(10))
}

fn flush_fstring_buffer<'a>(
    toks: &mut Vec<'a, Token<'a>>,
    buffer: &mut Option<String>,
    remaining_slice: &'a str,
    lex: &Lexer<'a, Token<'a>>,
) {
    let content = if let Some(s) = buffer {
        s.push_str(remaining_slice);
        lex.extras.arena.unwrap().alloc_str(s) as &'a str
    } else if !remaining_slice.is_empty() {
        remaining_slice
    } else {
        // dont emit empty
        // ex. `f"{a}{b}"` shouldn't emit any string parts
        return;
    };

    if !content.is_empty() {
        toks.push(Token::String(content));
    }
}

impl std::fmt::Display for FStringFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(c) = self.fill_char {
            write!(f, "{c}")?;
        }
        if let Some(a) = self.align {
            write!(f, "{a}")?;
        }
        write!(f, "{}", self.sign)?;
        if self.alt_form {
            write!(f, "#")?;
        }
        if self.zero_pad {
            write!(f, "0")?;
        }
        if let Some(w) = self.width {
            write!(f, "{w}")?;
        }
        if let Some(g) = self.grouping {
            write!(f, "{g}")?;
        }
        if let Some(p) = self.precision {
            write!(f, ".{p}")?;
        }
        if let Some(t) = self.type_spec {
            write!(f, ".{t}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::tokens::{LexerExtras, Token};
    use bumpalo::Bump;
    use logos::Logos;

    fn lex_fstring<'a>(input: &'a str, arena: &'a Bump) -> Vec<'a, Token<'a>> {
        let mut lexer = Token::lexer(input);
        lexer.extras = LexerExtras { arena: Some(arena) };

        match lexer.next().unwrap() {
            Ok(Token::FString(tokens)) => tokens,
            Err(e) => panic!("Unexpected error: {:?}", e),
            _ => panic!("Expected FString token"),
        }
    }

    #[test]
    fn test_simple_interpolation() {
        let arena = Bump::new();
        let tokens = lex_fstring(r#"f"cat: {cat}""#, &arena);

        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[0], Token::String("cat: "));
        assert_eq!(tokens[1], Token::FStringInterpStart);
        assert_eq!(tokens[2], Token::Ident("cat"));
        assert_eq!(tokens[3], Token::FStringInterpEnd);
    }

    #[test]
    fn test_multiple_interpolations() {
        let arena = Bump::new();
        let tokens = lex_fstring(r#"f"{a}{b}""#, &arena);

        assert_eq!(tokens.len(), 6);
        assert_eq!(tokens[0], Token::FStringInterpStart);
        assert_eq!(tokens[1], Token::Ident("a"));
        assert_eq!(tokens[2], Token::FStringInterpEnd);
        assert_eq!(tokens[3], Token::FStringInterpStart);
        assert_eq!(tokens[4], Token::Ident("b"));
        assert_eq!(tokens[5], Token::FStringInterpEnd);
    }

    #[test]
    fn test_escaped_braces() {
        let arena = Bump::new();
        let tokens = lex_fstring(r#"f"{{escaped}} and {{more}}""#, &arena);

        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::String("{escaped} and {more}"));
    }

    #[test]
    fn test_nested_braces_in_expression() {
        let arena = Bump::new();
        let tokens = lex_fstring(r#"f"result: {if x { 10 } else { 20 }}""#, &arena);

        assert_eq!(tokens[0], Token::String("result: "));
        assert_eq!(tokens[1], Token::FStringInterpStart);
        assert_eq!(tokens[2], Token::If);
        assert_eq!(tokens[3], Token::Ident("x"));
        assert_eq!(tokens[4], Token::OpenBrace);
        assert_eq!(tokens[tokens.len() - 1], Token::FStringInterpEnd);
    }

    #[test]
    fn test_format_precision() {
        let arena = Bump::new();
        let tokens = lex_fstring(r#"f"{pi:.2f}""#, &arena);

        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[0], Token::FStringInterpStart);
        assert_eq!(tokens[1], Token::Ident("pi"));

        match &tokens[2] {
            Token::FStringFormat(fmt) => {
                assert_eq!(fmt.precision, Some(2));
                assert_eq!(fmt.type_spec, Some(TypeSpec::FixedLower));
            }
            _ => panic!("Expected FStringFormat token"),
        }

        assert_eq!(tokens[3], Token::FStringInterpEnd);
    }

    #[test]
    fn test_format_width_and_align() {
        let arena = Bump::new();
        let tokens = lex_fstring(r#"f"{name:>10}""#, &arena);

        assert_eq!(tokens.len(), 4);
        match &tokens[2] {
            Token::FStringFormat(fmt) => {
                assert_eq!(fmt.align, Some(Align::Right));
                assert_eq!(fmt.width, Some(10));
            }
            _ => panic!("Expected FStringFormat token"),
        }
    }

    #[test]
    fn test_format_fill_and_align() {
        let arena = Bump::new();
        let tokens = lex_fstring(r#"f"{x:*^20}""#, &arena);

        match &tokens[2] {
            Token::FStringFormat(fmt) => {
                assert_eq!(fmt.fill_char, Some('*'));
                assert_eq!(fmt.align, Some(Align::Center));
                assert_eq!(fmt.width, Some(20));
            }
            _ => panic!("Expected FStringFormat token"),
        }
    }

    #[test]
    fn test_format_sign_and_alt() {
        let arena = Bump::new();
        let tokens = lex_fstring(r#"f"{num:+#x}""#, &arena);

        match &tokens[2] {
            Token::FStringFormat(fmt) => {
                assert_eq!(fmt.sign, Sign::Always);
                assert!(fmt.alt_form);
                assert_eq!(fmt.type_spec, Some(TypeSpec::HexLower));
            }
            _ => panic!("Expected FStringFormat token"),
        }
    }

    #[test]
    fn test_format_grouping() {
        let arena = Bump::new();
        let tokens = lex_fstring(r#"f"{big:,}""#, &arena);

        match &tokens[2] {
            Token::FStringFormat(fmt) => {
                assert_eq!(fmt.grouping, Some(Grouping::Comma));
            }
            _ => panic!("Expected FStringFormat token"),
        }
    }

    #[test]
    fn test_format_complex() {
        let arena = Bump::new();
        let tokens = lex_fstring(r#"f"{value:+#010,.2f}""#, &arena);

        match &tokens[2] {
            Token::FStringFormat(fmt) => {
                assert_eq!(fmt.sign, Sign::Always);
                assert!(fmt.alt_form);
                assert!(fmt.zero_pad);
                assert_eq!(fmt.width, Some(10));
                assert_eq!(fmt.grouping, Some(Grouping::Comma));
                assert_eq!(fmt.precision, Some(2));
                assert_eq!(fmt.type_spec, Some(TypeSpec::FixedLower));
            }
            _ => panic!("Expected FStringFormat token"),
        }
    }

    #[test]
    fn test_ternary_in_interpolation() {
        let arena = Bump::new();
        let tokens = lex_fstring(r#"f"{a > b ? a : b}""#, &arena);

        assert_eq!(tokens[0], Token::FStringInterpStart);
        assert_eq!(tokens[1], Token::Ident("a"));
        assert_eq!(tokens[2], Token::Gt);
        assert_eq!(tokens[3], Token::Ident("b"));
        assert_eq!(tokens[4], Token::Question);
        assert_eq!(tokens[5], Token::Ident("a"));
        assert_eq!(tokens[6], Token::Colon);
        assert_eq!(tokens[7], Token::Ident("b"));
        assert_eq!(tokens[8], Token::FStringInterpEnd);
    }

    #[test]
    fn test_nested_ternary() {
        let arena = Bump::new();
        let tokens = lex_fstring(r#"f"{a > b ? (a > c ? a : c) : b}""#, &arena);

        // colons should be emitted as tokens, not treated as format specs
        let colon_count = tokens.iter().filter(|t| matches!(t, Token::Colon)).count();
        assert_eq!(colon_count, 2);
        assert_eq!(tokens[tokens.len() - 1], Token::FStringInterpEnd);
    }

    #[test]
    fn test_raw_fstring() {
        let arena = Bump::new();
        let tokens = lex_fstring(r#"rf"path: {path}\n""#, &arena);

        assert_eq!(tokens[0], Token::String("path: "));
        assert_eq!(tokens[1], Token::FStringInterpStart);
        assert_eq!(tokens[2], Token::Ident("path"));
        assert_eq!(tokens[3], Token::FStringInterpEnd);
        assert_eq!(tokens[4], Token::String(r"\n"));
    }

    #[test]
    fn test_raw_fstring_fr() {
        let arena = Bump::new();
        let tokens = lex_fstring(r#"fr"value: {x}\t""#, &arena);

        assert_eq!(tokens[0], Token::String("value: "));
        assert_eq!(tokens[1], Token::FStringInterpStart);
        assert_eq!(tokens[2], Token::Ident("x"));
        assert_eq!(tokens[3], Token::FStringInterpEnd);
        assert_eq!(tokens[4], Token::String(r"\t"));
    }

    #[test]
    fn test_raw_hash_fstring() {
        let arena = Bump::new();
        let tokens = lex_fstring(r##"rf#"test: {x} with "quotes""#"##, &arena);

        assert_eq!(tokens[0], Token::String(r#"test: "#));
        assert_eq!(tokens[1], Token::FStringInterpStart);
        assert_eq!(tokens[2], Token::Ident("x"));
        assert_eq!(tokens[3], Token::FStringInterpEnd);
        assert_eq!(tokens[4], Token::String(r#" with "quotes""#));
    }

    #[test]
    fn test_raw_hash_fstring_double() {
        let arena = Bump::new();
        let tokens = lex_fstring(r###"rf##"value: {y} has "#" in it"##"###, &arena);

        assert_eq!(tokens[0], Token::String(r##"value: "##));
        assert_eq!(tokens[1], Token::FStringInterpStart);
        assert_eq!(tokens[2], Token::Ident("y"));
        assert_eq!(tokens[3], Token::FStringInterpEnd);
        assert_eq!(tokens[4], Token::String(r##" has "#" in it"##));
    }

    #[test]
    fn test_raw_fstring_with_format() {
        let arena = Bump::new();
        let tokens = lex_fstring(r#"rf"{val:.2f}\n""#, &arena);

        match &tokens[2] {
            Token::FStringFormat(fmt) => {
                assert_eq!(fmt.precision, Some(2));
                assert_eq!(fmt.type_spec, Some(TypeSpec::FixedLower));
            }
            _ => panic!("Expected FStringFormat token"),
        }
        assert_eq!(tokens[4], Token::String(r"\n"));
    }
}
