use crate::lexer::tokens::{LexicalError, Number, Token};
use logos::Lexer;

pub fn parse_number<'a>(lex: &mut Lexer<'a, Token<'a>>) -> Result<Number, LexicalError> {
    let first_digit = lex.slice();
    let remainder = lex.remainder();

    if first_digit == "0"
        && let Some(&b) = remainder.as_bytes().first()
    {
        match b {
            b'x' | b'X' => return parse_hex(remainder, lex),
            b'b' | b'B' => return parse_binary(remainder, lex),
            b'o' | b'O' => return parse_octal(remainder, lex),
            _ => {}
        }
    }

    parse_decimal(first_digit, remainder, lex)
}

fn parse_hex<'a>(remainder: &str, lex: &mut Lexer<'a, Token<'a>>) -> Result<Number, LexicalError> {
    let bytes = remainder.as_bytes();
    let mut i = 1;
    let mut value: i64 = 0;
    let mut has_digits = false;

    while i < bytes.len() {
        match bytes[i] {
            b'_' => {
                i += 1;
                continue;
            }
            b'0'..=b'9' => {
                let digit = (bytes[i] - b'0') as i64;
                value = value
                    .checked_mul(16)
                    .and_then(|v| v.checked_add(digit))
                    .ok_or(LexicalError::IntegerOverflow)?;
                has_digits = true;
                i += 1;
            }
            b'a'..=b'f' => {
                let digit = (bytes[i] - b'a' + 10) as i64;
                value = value
                    .checked_mul(16)
                    .and_then(|v| v.checked_add(digit))
                    .ok_or(LexicalError::IntegerOverflow)?;
                has_digits = true;
                i += 1;
            }
            b'A'..=b'F' => {
                let digit = (bytes[i] - b'A' + 10) as i64;
                value = value
                    .checked_mul(16)
                    .and_then(|v| v.checked_add(digit))
                    .ok_or(LexicalError::IntegerOverflow)?;
                has_digits = true;
                i += 1;
            }
            _ => break,
        }
    }

    if !has_digits {
        return Err(LexicalError::NoDigitsAfterPrefix);
    }

    lex.bump(i);
    Ok(Number::Int(value))
}

fn parse_binary<'a>(
    remainder: &str,
    lex: &mut Lexer<'a, Token<'a>>,
) -> Result<Number, LexicalError> {
    let bytes = remainder.as_bytes();
    let mut i = 1;
    let mut value: i64 = 0;
    let mut has_digits = false;

    while i < bytes.len() {
        match bytes[i] {
            b'_' => {
                i += 1;
                continue;
            }
            b'0' | b'1' => {
                let digit = (bytes[i] - b'0') as i64;
                value = value
                    .checked_mul(2)
                    .and_then(|v| v.checked_add(digit))
                    .ok_or(LexicalError::IntegerOverflow)?;
                has_digits = true;
                i += 1;
            }
            b'2'..=b'9' => {
                return Err(LexicalError::InvalidDigit {
                    digit: bytes[i] as char,
                    base: 2,
                    max: '1',
                });
            }
            _ => break,
        }
    }

    if !has_digits {
        return Err(LexicalError::NoDigitsAfterPrefix);
    }

    lex.bump(i);
    Ok(Number::Int(value))
}

fn parse_octal<'a>(
    remainder: &str,
    lex: &mut Lexer<'a, Token<'a>>,
) -> Result<Number, LexicalError> {
    let bytes = remainder.as_bytes();
    let mut i = 1;
    let mut value: i64 = 0;
    let mut has_digits = false;

    while i < bytes.len() {
        match bytes[i] {
            b'_' => {
                i += 1;
                continue;
            }
            b'0'..=b'7' => {
                let digit = (bytes[i] - b'0') as i64;
                value = value
                    .checked_mul(8)
                    .and_then(|v| v.checked_add(digit))
                    .ok_or(LexicalError::IntegerOverflow)?;
                has_digits = true;
                i += 1;
            }
            b'8' | b'9' => {
                return Err(LexicalError::InvalidDigit {
                    digit: bytes[i] as char,
                    base: 8,
                    max: '7',
                });
            }
            _ => break,
        }
    }

    if !has_digits {
        return Err(LexicalError::NoDigitsAfterPrefix);
    }

    lex.bump(i);
    Ok(Number::Int(value))
}

fn parse_decimal<'a>(
    first_digit: &str,
    remainder: &str,
    lex: &mut Lexer<'a, Token<'a>>,
) -> Result<Number, LexicalError> {
    let bytes = remainder.as_bytes();
    let mut i = 0;
    let mut int_value: i64 = (first_digit.as_bytes()[0] - b'0') as i64;

    // integer part
    while i < bytes.len() {
        match bytes[i] {
            b'_' => {
                i += 1;
                continue;
            }
            b'0'..=b'9' => {
                let digit = (bytes[i] - b'0') as i64;
                int_value = int_value
                    .checked_mul(10)
                    .and_then(|v| v.checked_add(digit))
                    .ok_or(LexicalError::IntegerOverflow)?;
                i += 1;
            }
            _ => break,
        }
    }

    // EOF
    if i >= bytes.len() {
        lex.bump(i);
        return Ok(Number::Int(int_value));
    }

    match bytes[i] {
        b'.' => {
            // EOF
            if i + 1 >= bytes.len() {
                lex.bump(i + 1);
                return Ok(Number::Float(int_value as f64));
            }

            match bytes[i + 1] {
                b'0'..=b'9' => {
                    // normal `3.14`
                    parse_float_with_fraction(int_value, bytes, i, lex)
                }
                b'.' => {
                    // range `3..10` -> parse int part only so `..` token can parse
                    lex.bump(i);
                    Ok(Number::Int(int_value))
                }
                b'f' => {
                    // `3.f`
                    lex.bump(i + 2);
                    Ok(Number::Float(int_value as f64))
                }
                _ => {
                    // `3.`
                    lex.bump(i + 1);
                    Ok(Number::Float(int_value as f64))
                }
            }
        }
        b'e' | b'E' => parse_float_with_exponent(int_value as f64, bytes, i, lex),
        b'f' => {
            // `3f`
            lex.bump(i + 1);
            Ok(Number::Float(int_value as f64))
        }
        _ => {
            lex.bump(i);
            Ok(Number::Int(int_value))
        }
    }
}

fn parse_float_with_fraction<'a>(
    int_part: i64,
    bytes: &[u8],
    start_i: usize,
    lex: &mut Lexer<'a, Token<'a>>,
) -> Result<Number, LexicalError> {
    let mut i = start_i + 1;
    let mut frac_value: f64 = 0.0;
    let mut frac_digits = 0;

    while i < bytes.len() {
        match bytes[i] {
            b'_' => {
                i += 1;
                continue;
            }
            b'0'..=b'9' => {
                let digit = (bytes[i] - b'0') as f64;
                frac_value = frac_value * 10.0 + digit;
                frac_digits += 1;
                i += 1;
            }
            _ => break,
        }
    }

    let value = int_part as f64 + frac_value / 10f64.powi(frac_digits);

    if i < bytes.len() && (bytes[i] == b'e' || bytes[i] == b'E') {
        return parse_float_with_exponent(value, bytes, i, lex);
    }

    if i < bytes.len() && bytes[i] == b'f' {
        i += 1;
    }

    if value.is_infinite() {
        return Err(LexicalError::FloatOverflow);
    }

    lex.bump(i);
    Ok(Number::Float(value))
}

fn parse_float_with_exponent<'a>(
    base_value: f64,
    bytes: &[u8],
    start_i: usize,
    lex: &mut Lexer<'a, Token<'a>>,
) -> Result<Number, LexicalError> {
    let mut i = start_i + 1;

    if i >= bytes.len() {
        return Err(LexicalError::MissingExponent);
    }

    let negative_exp = match bytes[i] {
        b'+' => {
            i += 1;
            false
        }
        b'-' => {
            i += 1;
            true
        }
        _ => false,
    };

    if i >= bytes.len() {
        return Err(LexicalError::MissingExponent);
    }

    let mut exp_value: i32 = 0;
    let mut has_exp_digits = false;

    while i < bytes.len() {
        match bytes[i] {
            b'_' => {
                i += 1;
                continue;
            }
            b'0'..=b'9' => {
                let digit = (bytes[i] - b'0') as i32;
                exp_value = exp_value
                    .checked_mul(10)
                    .and_then(|v| v.checked_add(digit))
                    .ok_or(LexicalError::ExponentOverflow)?;
                has_exp_digits = true;
                i += 1;
            }
            b'f' => {
                if !has_exp_digits {
                    return Err(LexicalError::MissingExponent);
                }
                i += 1;
                break;
            }
            _ => break,
        }
    }

    if !has_exp_digits {
        return Err(LexicalError::MissingExponent);
    }

    let exp = if negative_exp { -exp_value } else { exp_value };
    let value = base_value * 10f64.powi(exp);

    if value.is_infinite() {
        return Err(LexicalError::FloatOverflow);
    }

    lex.bump(i);
    Ok(Number::Float(value))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::tokens::{LexerExtras, Token};
    use bumpalo::Bump;
    use logos::Logos;

    fn lex_number(input: &str) -> Result<Number, LexicalError> {
        let arena = Bump::new();
        let mut lexer = Token::lexer(input);
        lexer.extras = LexerExtras {
            arena: Some(&arena),
        };

        match lexer.next().unwrap() {
            Ok(Token::Number(n)) => Ok(n),
            Err(e) => Err(e),
            _ => panic!("Expected Number token"),
        }
    }

    #[test]
    fn test_integers() {
        assert_eq!(lex_number("0").unwrap(), Number::Int(0));
        assert_eq!(lex_number("5").unwrap(), Number::Int(5));
        assert_eq!(lex_number("42").unwrap(), Number::Int(42));
        assert_eq!(lex_number("123456789").unwrap(), Number::Int(123456789));
        assert_eq!(lex_number("5_000_000").unwrap(), Number::Int(5000000));
        assert_eq!(lex_number("1_2_3").unwrap(), Number::Int(123));
    }

    #[test]
    fn test_hex() {
        assert_eq!(lex_number("0x0").unwrap(), Number::Int(0));
        assert_eq!(lex_number("0x7F").unwrap(), Number::Int(127));
        assert_eq!(lex_number("0xFF").unwrap(), Number::Int(255));
        assert_eq!(lex_number("0xABCD").unwrap(), Number::Int(0xABCD));
        assert_eq!(lex_number("0xabcd").unwrap(), Number::Int(0xabcd));
        assert_eq!(lex_number("0xAB_CD").unwrap(), Number::Int(0xABCD));
        assert_eq!(lex_number("0X10").unwrap(), Number::Int(16));
    }

    #[test]
    fn test_binary() {
        assert_eq!(lex_number("0b0").unwrap(), Number::Int(0));
        assert_eq!(lex_number("0b1").unwrap(), Number::Int(1));
        assert_eq!(lex_number("0b101010").unwrap(), Number::Int(42));
        assert_eq!(lex_number("0b1010_1010").unwrap(), Number::Int(170));
        assert_eq!(lex_number("0B11").unwrap(), Number::Int(3));
    }

    #[test]
    fn test_octal() {
        assert_eq!(lex_number("0o0").unwrap(), Number::Int(0));
        assert_eq!(lex_number("0o7").unwrap(), Number::Int(7));
        assert_eq!(lex_number("0o123").unwrap(), Number::Int(83));
        assert_eq!(lex_number("0o12_34").unwrap(), Number::Int(668));
        assert_eq!(lex_number("0O10").unwrap(), Number::Int(8));
    }

    #[test]
    fn test_floats() {
        assert_eq!(lex_number("3.12").unwrap(), Number::Float(3.12));
        assert_eq!(lex_number("0.5").unwrap(), Number::Float(0.5));
        assert_eq!(lex_number("8.").unwrap(), Number::Float(8.0));
        assert_eq!(lex_number("3.f").unwrap(), Number::Float(3.0));
        assert_eq!(lex_number("10f").unwrap(), Number::Float(10.0));
        assert_eq!(lex_number("1_2.3_4").unwrap(), Number::Float(12.34));
    }

    #[test]
    fn test_scientific() {
        assert_eq!(lex_number("1e10").unwrap(), Number::Float(1e10));
        assert_eq!(lex_number("1E10").unwrap(), Number::Float(1e10));
        assert_eq!(lex_number("135e12").unwrap(), Number::Float(135e12));
        assert_eq!(lex_number("1e-5").unwrap(), Number::Float(1e-5));
        assert_eq!(lex_number("2.5e3").unwrap(), Number::Float(2500.0));
        assert_eq!(lex_number("1e+2").unwrap(), Number::Float(100.0));
        assert_eq!(lex_number("1_0e1_0").unwrap(), Number::Float(1e11));
        assert_eq!(lex_number("5e2f").unwrap(), Number::Float(500.0));
    }

    #[test]
    fn test_errors() {
        assert_eq!(
            lex_number("0x").unwrap_err(),
            LexicalError::NoDigitsAfterPrefix
        );
        assert_eq!(
            lex_number("0b").unwrap_err(),
            LexicalError::NoDigitsAfterPrefix
        );
        assert_eq!(
            lex_number("0o").unwrap_err(),
            LexicalError::NoDigitsAfterPrefix
        );

        assert_eq!(
            lex_number("0b2").unwrap_err(),
            LexicalError::InvalidDigit {
                digit: '2',
                base: 2,
                max: '1'
            }
        );
        assert_eq!(
            lex_number("0o8").unwrap_err(),
            LexicalError::InvalidDigit {
                digit: '8',
                base: 8,
                max: '7'
            }
        );

        assert_eq!(lex_number("1e").unwrap_err(), LexicalError::MissingExponent);
        assert_eq!(
            lex_number("1e+").unwrap_err(),
            LexicalError::MissingExponent
        );
        assert_eq!(
            lex_number("1e-").unwrap_err(),
            LexicalError::MissingExponent
        );
    }
}
