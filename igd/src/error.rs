use crate::lexer::tokens::{LexicalError, Token};
use lalrpop_util::{ErrorRecovery, ParseError};
use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

#[derive(Error, Debug, Diagnostic)]
pub enum IgdError {
    #[error("Parse error")]
    #[diagnostic(code(igd::parse_error))]
    ParseError {
        #[source_code]
        src: String,

        #[label("{label}")]
        span: SourceSpan,

        label: String,

        #[help]
        help: Option<String>,
    },

    #[error("Multiple parse errors found")]
    #[diagnostic(code(igd::multiple_errors))]
    MultipleErrors {
        #[source_code]
        src: String,

        #[related]
        errors: Vec<IgdError>,
    },
}

impl IgdError {
    pub fn from_lalrpop_error(error: ParseError<usize, Token, LexicalError>, source: &str) -> Self {
        match error {
            ParseError::InvalidToken { location } => Self::invalid_token(source, location),

            ParseError::UnrecognizedEof { location, expected } => {
                Self::unexpected_eof(source, location, expected)
            }

            ParseError::UnrecognizedToken { token, expected } => {
                Self::unrecognized_token(source, token, expected)
            }

            ParseError::ExtraToken { token } => Self::extra_token(source, token),

            ParseError::User { error } => Self::lexer_error(source, error),
        }
    }

    pub fn from_error_recovery(
        errors: Vec<ErrorRecovery<usize, Token, LexicalError>>,
        source: &str,
    ) -> Self {
        if errors.len() == 1 {
            Self::from_lalrpop_error(errors.into_iter().next().unwrap().error, source)
        } else {
            Self::multiple_errors(source, errors)
        }
    }

    fn invalid_token(source: &str, location: usize) -> Self {
        IgdError::ParseError {
            src: source.to_string(),
            span: (location, 1).into(),
            label: "invalid token".to_string(),
            help: None,
        }
    }

    fn unexpected_eof(source: &str, location: usize, expected: Vec<String>) -> Self {
        let help = if !expected.is_empty() {
            Some(format!("Expected {}", format_expected(&expected)))
        } else {
            Some("Unexpected end of file".to_string())
        };

        IgdError::ParseError {
            src: source.to_string(),
            span: (location, 0).into(),
            label: "unexpected end of file".to_string(),
            help,
        }
    }

    fn unrecognized_token(
        source: &str,
        token: (usize, Token, usize),
        expected: Vec<String>,
    ) -> Self {
        let (start, tok, end) = token;
        let (label, help) = build_error_message(&tok, &expected);

        IgdError::ParseError {
            src: source.to_string(),
            span: (start, end - start).into(),
            label,
            help,
        }
    }

    fn extra_token(source: &str, token: (usize, Token, usize)) -> Self {
        let (start, tok, end) = token;

        IgdError::ParseError {
            src: source.to_string(),
            span: (start, end - start).into(),
            label: format!("unexpected extra {}", tok),
            help: Some("This token should not be here".to_string()),
        }
    }

    fn lexer_error(source: &str, error: LexicalError) -> Self {
        IgdError::ParseError {
            src: source.to_string(),
            span: (0, 1).into(),
            label: format!("lexer error: {}", error),
            help: None,
        }
    }

    fn multiple_errors(
        source: &str,
        errors: Vec<ErrorRecovery<usize, Token, LexicalError>>,
    ) -> Self {
        IgdError::MultipleErrors {
            src: source.to_string(),
            errors: errors
                .into_iter()
                .map(|e| Self::from_lalrpop_error(e.error, source))
                .collect(),
        }
    }
}

fn build_error_message(token: &Token, expected: &[String]) -> (String, Option<String>) {
    if let Token::MistakenKeyword(kw) = token {
        return get_mistaken_keyword_message(kw);        
    }

    if let Some((label, help)) = check_missing_type_annotation(token, expected) {
        return (label, Some(help));
    }

    if let Some((label, help)) = check_missing_semicolon(token, expected) {
        return (label, Some(help));
    }

    if let Some((label, help)) = check_incomplete_expression(token, expected) {
        return (label, Some(help));
    }

    if let Some((label, help)) = check_invalid_character(token) {
        return (label, Some(help));
    }

    if let Some((label, help)) = check_missing_field_type(token, expected) {
        return (label, Some(help));
    }

    if let Token::Ident(name) = token
        && let Some(suggestion) = suggest_similar_keyword(name)
    {
        return (format!("unknown identifier '{}'", name), Some(suggestion));
    }

    let label = format!("unexpected {}", token);
    let help = if !expected.is_empty() {
        Some(format!("Expected {}", format_expected(expected)))
    } else {
        None
    };

    (label, help)
}

fn get_mistaken_keyword_message(keyword: &str) -> (String, Option<String>) {
    match keyword {
        "var" => (
            "unknown keyword 'var'".to_string(),
            Some("Use 'let' for variables in igd.\nTry: `let x = 5;`".to_string()),
        ),
        
        "function" | "def" | "func" => (
            format!("unknown keyword '{}'", keyword),
            Some("Use 'fn' to define functions.\nTry: `fn name() { ... }`".to_string()),
        ),
        
        "elif" => (
            "unknown keyword 'elif'".to_string(),
            Some("Use 'else if' for additional conditions.\nTry: `else if condition { ... }`".to_string()),
        ),

        "True" | "False" => (
            format!("unknown keyword '{}'", keyword),
            Some("igd uses lowercase booleans. Try `true` or `false`.".to_string()),
        ),
        
        "null" | "nil" | "None" | "undefined" => (
            format!("unknown keyword '{}'", keyword),
            Some("igd does not have null values.".to_string()),
        ),
        
        "void" => (
            "unknown keyword 'void'".to_string(),
            Some("Functions without return values don't need a return type.\nTry: `fn name() { ... }`".to_string()),
        ),
        
        "class" | "interface" => (
            format!("unknown keyword '{}'", keyword),
            Some("Use 'struct' to define data structures.\nTry: `struct Name { ... }`".to_string()),
        ),
        
        "self" | "this" => (
            format!("unknown keyword '{}'", keyword),
            Some(format!("'{}' is not a keyword in igd.\nStruct methods don't use explicit self/this parameters.", keyword)),
        ),
        
        "mut" => (
            "unknown keyword 'mut'".to_string(),
            Some("Variables in igd don't need a 'mut' keyword.\nUse 'let' for all variables: `let x = 5;`".to_string()),
        ),
        
        "async" | "await" => (
            format!("unknown keyword '{}'", keyword),
            Some("igd does not support async/await.".to_string()),
        ),
        
        "pass" => (
            "unknown keyword 'pass'".to_string(),
            Some("Empty blocks don't need a 'pass' statement.\nJust use empty braces: `{ }`".to_string()),
        ),
        
        "unless" | "until" => (
            format!("unknown keyword '{}'", keyword),
            Some("Use 'if' with a negated condition.\nTry: `if !condition { ... }`".to_string()),
        ),
        
        "switch" | "match" | "case" => (
            format!("unknown keyword '{}'", keyword),
            Some(format!("igd does not have '{}' statements.\nUse if/else if chains instead.", keyword)),
        ),
        
        "foreach" => (
            "unknown keyword 'foreach'".to_string(),
            Some("Use 'for' to iterate over collections.\nTry: `for item in collection { ... }`".to_string()),
        ),
        
        "do" => (
            "unknown keyword 'do'".to_string(),
            Some("igd does not have do-while loops.\nUse 'while' instead: `while condition { ... }`".to_string()),
        ),
        
        "loop" => (
            "unknown keyword 'loop'".to_string(),
            Some("Use 'while true' for infinite loops.\nTry: `while true { ... }`".to_string()),
        ),
        
        "repeat" => (
            "unknown keyword 'repeat'".to_string(),
            Some("Use 'while' or 'for' loops in igd.\nTry: `while condition { ... }`".to_string()),
        ),
        
        "auto" => (
            "unknown keyword 'auto'".to_string(),
            Some("igd infers types automatically.\nJust use: `let x = 5;`".to_string()),
        ),
        
        "any" | "dynamic" => (
            format!("unknown keyword '{}'", keyword),
            Some("igd does not have dynamic typing.\nAll types are determined at compile time.".to_string()),
        ),
        
        "number" => (
            "unknown keyword 'number'".to_string(),
            Some("Use 'int' or 'float' for numeric types.".to_string()),
        ),
        
        "str" => (
            "unknown keyword 'str'".to_string(),
            Some("Use 'string' for text.\nTry: `let text: string = \"hello\";`".to_string()),
        ),
        
        "public" | "pub" | "private" | "protected" => (
            format!("unknown keyword '{}'", keyword),
            Some("igd does not have access modifiers.\nAll items are public.".to_string()),
        ),
        
        "static" => (
            "unknown keyword 'static'".to_string(),
            Some("igd does not use 'static' keyword.\nUse 'const' for constants: `const PI = 3.14;`".to_string()),
        ),
        
        "try" | "catch" | "finally" | "except" => (
            format!("unknown keyword '{}'", keyword),
            Some("igd does not have try/catch error handling.".to_string()),
        ),
        
        "throw" | "raise" => (
            format!("unknown keyword '{}'", keyword),
            Some("igd does not have exception throwing.".to_string()),
        ),
        
        "and" => (
            "unknown keyword 'and'".to_string(),
            Some("Use '&&' for logical AND.\nTry: `if a && b { ... }`".to_string()),
        ),
        
        "or" => (
            "unknown keyword 'or'".to_string(),
            Some("Use '||' for logical OR.\nTry: `if a || b { ... }`".to_string()),
        ),
        
        "not" => (
            "unknown keyword 'not'".to_string(),
            Some("Use '!' for logical NOT.\nTry: `if !condition { ... }`".to_string()),
        ),
        
        "is" => (
            "unknown keyword 'is'".to_string(),
            Some("Use '==' for equality comparison.\nTry: `if x == y { ... }`".to_string()),
        ),
        
        "new" => (
            "unknown keyword 'new'".to_string(),
            Some("igd does not use 'new' for construction.\nJust instantiate the struct: `MyStruct { field: value }`".to_string()),
        ),
        
        "import" | "require" => (
            format!("unknown keyword '{}'", keyword),
            Some("Use 'use' to import modules.\nTry: `use module::item;`".to_string()),
        ),
        
        "export" => (
            "unknown keyword 'export'".to_string(),
            Some("All igd items are automatically public and exported.\nNo explicit 'export' needed.".to_string()),
        ),
        
        "namespace" | "package" => (
            format!("unknown keyword '{}'", keyword),
            Some("Use 'mod' to define modules.\nTry: `mod module_name { ... }`".to_string()),
        ),
        
        "goto" => (
            "unknown keyword 'goto'".to_string(),
            Some("igd does not support 'goto'.".to_string()),
        ),
        
        "super" | "extends" | "implements" => (
            format!("unknown keyword '{}'", keyword),
            Some("igd does not have inheritance.".to_string()),
        ),
        
        _ => (
            format!("unknown keyword '{}'", keyword),
            None,
        ),
    }
}

fn suggest_similar_keyword(input: &str) -> Option<String> {
    const KEYWORDS: &[&str] = &[
        "fn",
        "let",
        "const",
        "if",
        "else",
        "while",
        "for",
        "in",
        "struct",
        "enum",
        "type",
        "dashboard",
        "element",
        "use",
        "mod",
        "return",
        "break",
        "continue",
        "int",
        "float",
        "bool",
        "string",
        "true",
        "false",
    ];

    let input_lower = input.to_lowercase();

    let mut best_match: Option<(&str, usize)> = None;

    for keyword in KEYWORDS {
        let distance = strsim::levenshtein(&input_lower, keyword);

        if distance <= 2 {
            if let Some((_, best_distance)) = best_match {
                if distance < best_distance {
                    best_match = Some((keyword, distance));
                }
            } else {
                best_match = Some((keyword, distance));
            }
        }
    }

    best_match.map(|(keyword, _)| format!("Did you mean '{}'?", keyword))
}

fn check_missing_type_annotation(token: &Token, expected: &[String]) -> Option<(String, String)> {
    let expects_colon = expected.iter().any(|e| {
        let trimmed = e.trim_matches('"');
        trimmed == ":" || trimmed == "Colon"
    });
    let is_param_separator = matches!(token, Token::CloseParen | Token::Comma);

    if expects_colon && is_param_separator {
        return Some((
            "missing type annotation".to_string(),
            "Function parameters require type annotations.\nTry: `fn name(param: type) { ... }`"
                .to_string(),
        ));
    }

    None
}

fn check_missing_semicolon(token: &Token, expected: &[String]) -> Option<(String, String)> {
    let expects_semicolon = expected.iter().any(|e| {
        let trimmed = e.trim_matches('"');
        trimmed == ";" || trimmed == "SemiColon"
    });

    if expects_semicolon {
        return Some((
            format!("unexpected {}", token),
            "Did you forget a semicolon ';' at the end of the statement?".to_string(),
        ));
    }

    None
}

fn check_incomplete_expression(token: &Token, expected: &[String]) -> Option<(String, String)> {
    let is_terminator = token.is_separator();
    let expects_value = expected.iter().any(|e| {
        let trimmed = e.trim_matches('"');
        trimmed.contains("Ident")
            || trimmed.contains("Int")
            || trimmed.contains("Float")
            || trimmed.contains("String")
            || trimmed.contains("Bool")
    });

    if is_terminator && expects_value {
        return Some((
            "incomplete expression".to_string(),
            "This expression is missing a value".to_string(),
        ));
    }

    None
}

fn check_invalid_character(token: &Token) -> Option<(String, String)> {
    if matches!(token, Token::Error) {
        return Some((
            "invalid character".to_string(),
            "This character is not recognized by the language.\nValid characters include: letters, digits, and common symbols"
                .to_string(),
        ));
    }

    None
}

fn check_missing_field_type(token: &Token, expected: &[String]) -> Option<(String, String)> {
    let expects_colon = expected.iter().any(|e| {
        let trimmed = e.trim_matches('"');
        trimmed == ":" || trimmed == "Colon"
    });
    let is_struct_separator = matches!(token, Token::CloseBrace | Token::Comma);

    if expects_colon && is_struct_separator {
        return Some((
            "missing field type".to_string(),
            "Struct fields require type annotations.\nTry: `field_name: type`".to_string(),
        ));
    }

    None
}

fn format_expected(expected: &[String]) -> String {
    let categorized = categorize_expected_tokens(expected);
    format_categories(categorized)
}

struct CategorizedTokens<'a> {
    literals: Vec<&'a str>,
    types: Vec<&'a str>,
    keywords: Vec<&'a str>,
    operators: Vec<&'a str>,
    punctuation: Vec<&'a str>,
}

fn categorize_expected_tokens(expected: &[String]) -> CategorizedTokens<'_> {
    let mut categorized = CategorizedTokens {
        literals: Vec::new(),
        types: Vec::new(),
        keywords: Vec::new(),
        operators: Vec::new(),
        punctuation: Vec::new(),
    };

    for token_str in expected.iter().map(|s| s.trim_matches('"')) {
        match token_str {
            "Ident" => categorized.literals.push("an identifier"),
            "Int" => categorized.literals.push("an integer"),
            "Float" => categorized.literals.push("a float"),
            "String" => categorized.literals.push("a string"),
            "Bool" => categorized.literals.push("a boolean"),

            "IntType" | "int" => categorized.types.push("int"),
            "FloatType" | "float" => categorized.types.push("float"),
            "BoolType" | "bool" => categorized.types.push("bool"),
            "StringType" | "string" => categorized.types.push("string"),

            "let" | "const" | "fn" | "struct" | "enum" | "type" | "if" | "else" | "while"
            | "for" | "return" | "break" | "continue" | "use" | "mod" | "dashboard" | "element" => {
                categorized.keywords.push(token_str)
            }

            "+" | "-" | "*" | "/" | "%" | "**" | "<<" | ">>" | "==" | "!=" | "<" | "<=" | ">"
            | ">=" | "&&" | "||" | "^" | "&" | "|" | "!" | "as" | "+=" | "-=" | "*=" | "/="
            | "%=" | "**=" | "^=" | "&=" | "|=" | "<<=" | ">>=" => {
                categorized.operators.push(token_str)
            }

            "(" | ")" | "[" | "]" | "{" | "}" | ";" | ":" | "::" | "," | "." | "->" | "=>"
            | ".." | "..=" | "?" | "~" | "=" => categorized.punctuation.push(token_str),

            _ => categorized.operators.push(token_str),
        }
    }

    categorized
}

fn format_categories(cat: CategorizedTokens) -> String {
    let mut parts = Vec::new();

    if !cat.literals.is_empty() {
        parts.push(format_list(&cat.literals, "a value"));
    }

    if !cat.types.is_empty() {
        let unique_types = deduplicate(&cat.types);
        parts.push(format_list(&unique_types, "a type"));
    }

    if !cat.keywords.is_empty() {
        parts.push(format_keywords(&cat.keywords));
    }

    if !cat.operators.is_empty() {
        parts.push(format_operators(&cat.operators));
    }

    if !cat.punctuation.is_empty() {
        parts.push(format_punctuation(&cat.punctuation));
    }

    join_parts(parts)
}

fn format_list(items: &[&str], category_name: &str) -> String {
    if items.len() == 1 {
        items[0].to_string()
    } else {
        format!("{} ({})", category_name, items.join(", "))
    }
}

fn format_keywords(keywords: &[&str]) -> String {
    if keywords.len() == 1 {
        format!("'{}'", keywords[0])
    } else if keywords.len() <= 3 {
        keywords
            .iter()
            .map(|k| format!("'{}'", k))
            .collect::<Vec<_>>()
            .join(", ")
    } else {
        "a keyword".to_string()
    }
}

fn format_operators(operators: &[&str]) -> String {
    if operators.len() <= 5 {
        operators
            .iter()
            .map(|o| format!("'{}'", o))
            .collect::<Vec<_>>()
            .join(", ")
    } else {
        "an operator".to_string()
    }
}

fn format_punctuation(punctuation: &[&str]) -> String {
    if punctuation.len() == 1 {
        format!("'{}'", punctuation[0])
    } else if punctuation.len() <= 3 {
        punctuation
            .iter()
            .map(|p| format!("'{}'", p))
            .collect::<Vec<_>>()
            .join(", ")
    } else {
        "punctuation".to_string()
    }
}

fn deduplicate<'a>(items: &[&'a str]) -> Vec<&'a str> {
    let mut unique = items.to_vec();
    unique.sort();
    unique.dedup();
    unique
}

fn join_parts(parts: Vec<String>) -> String {
    match parts.len() {
        0 => "something else".to_string(),
        1 => parts[0].clone(),
        2 => format!("{} or {}", parts[0], parts[1]),
        _ => {
            let mut parts = parts;
            let last = parts.pop().unwrap();
            format!("{}, or {}", parts.join(", "), last)
        }
    }
}
