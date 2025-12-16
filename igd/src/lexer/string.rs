use crate::lexer::tokens::{LexicalError, Token};
use logos::Lexer;

#[derive(Default)]
pub struct StringParser {
    pub i: usize,
    pub last_copy: usize,
    pub is_raw: bool,
}

impl StringParser {
    /// Handles common string parsing logic (escapes, line continuation, normalization)
    /// Returns true if this character was handled, false if caller should handle it
    pub fn handle_char(
        &mut self,
        remainder: &str,
        string_buffer: &mut Option<String>,
    ) -> Result<bool, LexicalError> {
        match remainder.as_bytes()[self.i] {
            // string continuation (backslash followed by newline)
            b'\\'
                if !self.is_raw
                    && matches!(
                        remainder.as_bytes().get(self.i + 1),
                        Some(&b'\n') | Some(&b'\r')
                    ) =>
            {
                let buf = ensure_unescaped(string_buffer);
                buf.push_str(&remainder[self.last_copy..self.i]);

                self.i += 1; // skip backslash

                // skip \n, \r, or \r\n
                if remainder.as_bytes()[self.i] == b'\r' {
                    self.i += 1;
                    if remainder.as_bytes().get(self.i) == Some(&b'\n') {
                        self.i += 1;
                    }
                } else {
                    self.i += 1;
                }

                self.i += skip_whitespace(&remainder[self.i..]);
                self.last_copy = self.i;
                Ok(true)
            }

            // regular escape sequence
            b'\\' if !self.is_raw => {
                let buf = ensure_unescaped(string_buffer);
                buf.push_str(&remainder[self.last_copy..self.i]);

                let (ch, len) = parse_escape(&remainder[self.i..])?;
                buf.push(ch);
                self.i += len;
                self.last_copy = self.i;
                Ok(true)
            }

            // carriage return normalization (\r or \r\n -> \n)
            b'\r' => {
                let buf = ensure_unescaped(string_buffer);
                buf.push_str(&remainder[self.last_copy..self.i]);

                self.i += 1;
                if remainder.as_bytes().get(self.i) == Some(&b'\n') {
                    self.i += 1;
                }
                buf.push('\n');
                self.last_copy = self.i;
                Ok(true)
            }

            // caller should handle
            _ => Ok(false),
        }
    }
}

/// Parse a regular string literal starting after the opening quote
/// Handles escape sequences, string continuation, and line ending normalization
/// Cow (zero-copy if no escape sequences exist, otherwise allocates in Bump)
pub fn parse_string<'a>(lex: &mut Lexer<'a, Token<'a>>) -> Result<&'a str, LexicalError> {
    let remainder = lex.remainder();
    let mut parser = StringParser::default();
    let mut unescaped: Option<String> = None;

    while parser.i < remainder.len() {
        if parser.handle_char(remainder, &mut unescaped)? {
            // was already handled -> skip
            continue;
        }

        match remainder.as_bytes()[parser.i] {
            b'"' => {
                lex.bump(parser.i + 1);
                return Ok(finalize_string(
                    unescaped,
                    &remainder[parser.last_copy..parser.i],
                    lex,
                ));
            }
            _ => parser.i += 1,
        }
    }

    Err(LexicalError::UnterminatedString)
}

/// Parse a raw string literal (`r"..."`)
/// No escape sequences, but line endings are normalized
pub fn parse_raw_string<'a>(lex: &mut Lexer<'a, Token<'a>>) -> Result<&'a str, LexicalError> {
    parse_raw_string_with_hashes(lex, 0)
}

/// Parse a raw string literal with hash delimiters: `r#"..."#`, `r##"..."##`, etc.
pub fn parse_raw_hash_string<'a>(lex: &mut Lexer<'a, Token<'a>>) -> Result<&'a str, LexicalError> {
    let remainder = lex.remainder();

    // already matched r# -> start at 1
    let num_hashes = 1 + remainder.bytes().take_while(|&b| b == b'#').count();

    // verify opening quote exists after the hashes
    if remainder.as_bytes().get(num_hashes - 1) != Some(&b'"') {
        return Err(LexicalError::RawHashStringNoQuote);
    }

    parse_raw_string_with_hashes(lex, num_hashes)
}

/// Parse raw string with specified number of hash delimiters
fn parse_raw_string_with_hashes<'a>(
    lex: &mut Lexer<'a, Token<'a>>,
    num_hashes: usize,
) -> Result<&'a str, LexicalError> {
    let remainder = lex.remainder();
    let start = num_hashes;
    let mut parser = StringParser {
        is_raw: true,
        i: start, // start after opening quote
        last_copy: start,
    };
    let mut normalized: Option<String> = None;

    while parser.i < remainder.len() {
        if parser.handle_char(remainder, &mut normalized)? {
            // was already handled -> skip
            continue;
        }

        match remainder.as_bytes()[parser.i] {
            b'"' => {
                if check_closing_hashes(&remainder[parser.i + 1..], num_hashes) {
                    let end_pos = parser.i + 1 + num_hashes;
                    lex.bump(end_pos);

                    return Ok(finalize_string(
                        normalized,
                        &remainder[parser.last_copy..parser.i],
                        lex,
                    ));
                }
                parser.i += 1;
            }
            _ => parser.i += 1,
        }
    }

    Err(LexicalError::UnterminatedRawString)
}

/// Check if the input starts with exactly `count` hash symbols
pub fn check_closing_hashes(input: &str, count: usize) -> bool {
    input.len() >= count
        && input[..count].bytes().all(|b| b == b'#')
        && input.as_bytes().get(count).is_none_or(|&b| b != b'#')
}

/// Parse an escape sequence starting at a backslash
/// Returns the character and the number of bytes consumed
pub fn parse_escape(input: &str) -> Result<(char, usize), LexicalError> {
    let bytes = input.as_bytes();

    if bytes.len() < 2 {
        return Err(LexicalError::ShortEscapeSequence);
    }

    match bytes[1] {
        b'\'' => Ok(('\'', 2)),
        b'"' => Ok(('"', 2)),

        b'n' => Ok(('\n', 2)),
        b'r' => Ok(('\r', 2)),
        b't' => Ok(('\t', 2)),
        b'\\' => Ok(('\\', 2)),
        b'0' => Ok(('\0', 2)),

        b'x' => parse_hex_escape(input),

        b'u' => parse_unicode_escape(input, false),
        b'U' => parse_unicode_escape(input, true),

        c => Err(LexicalError::InvalidEscapeSequence(c as char)),
    }
}

/// Parse a hex escape (`\xHH`)
fn parse_hex_escape(input: &str) -> Result<(char, usize), LexicalError> {
    let bytes = input.as_bytes();

    if bytes.len() < 4 {
        return Err(LexicalError::ShortHexEscapeSequence);
    }

    let first = hex_digit_value(bytes[2]).ok_or(LexicalError::InvalidHexDigit)?;
    let second = hex_digit_value(bytes[3]).ok_or(LexicalError::InvalidHexDigit)?;
    let value = first * 16 + second;

    if value > 0x7F {
        return Err(LexicalError::HexEscapeSequenceTooBig(value));
    }

    Ok((value as char, 4))
}

/// Parse unicode escapes in many formats
fn parse_unicode_escape(input: &str, is_big_u: bool) -> Result<(char, usize), LexicalError> {
    let bytes = input.as_bytes();

    // `\u{...}`
    if !is_big_u && bytes.get(2) == Some(&b'{') {
        let closing = input
            .bytes()
            .position(|b| b == b'}')
            .ok_or(LexicalError::UnicodeEscapeMissingCloseBrace)?;

        let hex_str = &input[3..closing];
        if hex_str.is_empty() {
            return Err(LexicalError::EmptyUnicodeEscape);
        }

        let value = parse_hex_number(hex_str)?;
        let ch = char::from_u32(value).ok_or(LexicalError::InvalidUnicodeCodePoint(value))?;

        return Ok((ch, closing + 1));
    }

    // `\uXXXX` or `\UXXXXXXXX`
    let num_digits = if is_big_u { 8 } else { 4 };
    let start = 2;
    let end = start + num_digits;

    if bytes.len() < end {
        return Err(LexicalError::ShortEscapeSequence);
    }

    let hex_str = &input[start..end];
    let value = parse_hex_number(hex_str)?;
    let ch = char::from_u32(value).ok_or(LexicalError::InvalidUnicodeCodePoint(value))?;

    Ok((ch, end))
}

/// Parse a hex number string (allowing underscores as separators)
fn parse_hex_number(s: &str) -> Result<u32, LexicalError> {
    let mut value: u32 = 0;

    for b in s.bytes() {
        if b == b'_' {
            continue;
        }

        let digit = hex_digit_value(b).ok_or(LexicalError::InvalidHexDigit)?;
        value = value
            .checked_mul(16)
            .and_then(|v| v.checked_add(digit as u32))
            .ok_or(LexicalError::UnicodeEscapeOverflow)?;
    }

    Ok(value)
}

/// Convert a hex digit character to its numeric value (0-15)
fn hex_digit_value(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

/// Skip whitespace characters (space, tab, newline) for string continuation
pub fn skip_whitespace(input: &str) -> usize {
    input
        .bytes()
        .take_while(|&b| matches!(b, b' ' | b'\t' | b'\n'))
        .count()
}

fn ensure_unescaped(unescaped: &mut Option<String>) -> &mut String {
    unescaped.get_or_insert_with(|| String::with_capacity(10))
}

/// returns a slice of input or allocate in Bump
fn finalize_string<'a>(
    mut unescaped: Option<String>,
    remaining: &'a str,
    lex: &Lexer<'a, Token<'a>>,
) -> &'a str {
    if let Some(ref mut s) = unescaped {
        s.push_str(remaining);
        lex.extras.arena.unwrap().alloc_str(s)
    } else {
        remaining
    }
}

#[cfg(test)]
mod tests {
    use crate::lexer::tokens::LexerExtras;

    use super::*;
    use bumpalo::Bump;
    use logos::Logos;

    fn lex_string(input: &str) -> Result<String, LexicalError> {
        let arena = Bump::new();
        let mut lexer = Token::lexer(input);
        lexer.extras = LexerExtras {
            arena: Some(&arena),
        };

        match lexer.next().unwrap() {
            Ok(Token::String(s)) => Ok(s.to_string()),
            Err(e) => Err(e),
            _ => panic!(),
        }
    }

    #[test]
    fn test_regular_strings() {
        assert_eq!(lex_string(r#""hello""#).unwrap(), "hello");

        assert_eq!(
            lex_string(r#""a\nb\tc\\d\"e\'f""#).unwrap(),
            "a\nb\tc\\d\"e'f"
        );
        assert_eq!(lex_string(r#""\x41\x42""#).unwrap(), "AB");
        assert_eq!(lex_string(r#""\0null""#).unwrap(), "\0null");

        assert_eq!(lex_string("\"hello\\\n  world\"").unwrap(), "helloworld");
        assert_eq!(lex_string("\"hello\\\r\n  world\"").unwrap(), "helloworld");

        assert_eq!(lex_string("\"hello\rworld\"").unwrap(), "hello\nworld");
        assert_eq!(lex_string("\"hello\r\nworld\"").unwrap(), "hello\nworld");
    }

    #[test]
    fn test_unicode_escapes() {
        assert_eq!(lex_string(r#""\u0041""#).unwrap(), "A");

        assert_eq!(lex_string(r#""\U00000041""#).unwrap(), "A");

        assert_eq!(lex_string(r#""\u{41}""#).unwrap(), "A");
        assert_eq!(lex_string(r#""\u{1F600}""#).unwrap(), "😀");
        assert_eq!(lex_string(r#""\u{1_F6_00}""#).unwrap(), "😀");
    }

    #[test]
    fn test_raw_strings() {
        assert_eq!(lex_string(r#"r"hello\nworld""#).unwrap(), r"hello\nworld");

        assert_eq!(lex_string("r\"hello\rworld\"").unwrap(), "hello\nworld");

        assert_eq!(
            lex_string(r##"r#"hello"world"#"##).unwrap(),
            r#"hello"world"#
        );

        assert_eq!(
            lex_string(r###"r##"hello"#world"##"###).unwrap(),
            r##"hello"#world"##
        );
        assert_eq!(
            lex_string(r####"r###"a"##b"###"####).unwrap(),
            r###"a"##b"###
        );

        assert_eq!(
            lex_string(r###"r##"test"#more"##"###).unwrap(),
            r##"test"#more"##
        );
    }

    #[test]
    fn test_errors() {
        assert_eq!(
            lex_string(r#""hello"#),
            Err(LexicalError::UnterminatedString)
        );
        assert_eq!(lex_string(r#""\""#), Err(LexicalError::UnterminatedString));
        assert_eq!(
            lex_string(r#"r"hello"#),
            Err(LexicalError::UnterminatedRawString)
        );
        assert_eq!(
            lex_string(r##"r#"hello"##),
            Err(LexicalError::UnterminatedRawString)
        );

        assert_eq!(
            lex_string(r#""\q""#),
            Err(LexicalError::InvalidEscapeSequence('q'))
        );

        assert_eq!(lex_string(r#""\xGG""#), Err(LexicalError::InvalidHexDigit));
        assert_eq!(
            lex_string(r#""\x""#),
            Err(LexicalError::ShortHexEscapeSequence)
        );
        assert_eq!(
            lex_string(r#""\xFF""#),
            Err(LexicalError::HexEscapeSequenceTooBig(0xFF))
        );

        assert_eq!(
            lex_string(r#""\u{GGGG}""#),
            Err(LexicalError::InvalidHexDigit)
        );
        assert_eq!(
            lex_string(r#""\u{110000}""#),
            Err(LexicalError::InvalidUnicodeCodePoint(0x110000))
        );
        assert_eq!(
            lex_string(r#""\u{}""#),
            Err(LexicalError::EmptyUnicodeEscape)
        );
        assert_eq!(
            lex_string(r#""\u{123""#),
            Err(LexicalError::UnicodeEscapeMissingCloseBrace)
        );

        assert_eq!(
            lex_string(r####"r##test"##"####),
            Err(LexicalError::RawHashStringNoQuote)
        );
    }
}
