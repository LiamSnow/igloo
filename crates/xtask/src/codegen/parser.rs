use anyhow::{Result, bail};
use ariadne::{Color, Label, Report, ReportKind, Source};
use logos::{Logos, Span, SpannedIter};
use std::{fs, path::Path};

#[derive(Debug)]
#[allow(dead_code)]
pub enum Component {
    Marker {
        name: String,
        doc: String,
    },
    Single {
        name: String,
        doc: String,
        ty: IglooType,
    },
    Enum {
        name: String,
        doc: String,
        vars: Vec<Variant>,
    },
}

#[derive(Debug)]
pub enum IglooType {
    Int,
    Float,
    Bool,
    String,
    Color,
    Date,
    Time,

    IntList,
    FloatList,
    BoolList,
    StringList,
    ColorList,
    DateList,
    TimeList,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct Variant {
    pub name: String,
    pub doc: String,
}

#[derive(Logos, Clone, Debug, PartialEq)]
#[logos(skip r"[ \t]+")]
enum Token<'a> {
    #[regex(r"\r?\n")]
    NewLine,

    #[regex(r"#[^\n]*", |lex| lex.slice()[1..].trim(), allow_greedy = true)]
    CommentLine(&'a str),

    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,

    #[token("none")]
    None,
    #[token("enum")]
    Enum,

    #[token("float")]
    Float,
    #[token("int")]
    Int,
    #[token("bool")]
    Bool,
    #[token("string")]
    String,
    #[token("color")]
    Color,
    #[token("date")]
    Date,
    #[token("time")]
    Time,

    #[token("list<float>")]
    FloatList,
    #[token("list<int>")]
    IntList,
    #[token("list<bool>")]
    BoolList,
    #[token("list<string>")]
    StringList,
    #[token("list<color>")]
    ColorList,
    #[token("list<date>")]
    DateList,
    #[token("list<time>")]
    TimeList,

    #[regex(r#"[\p{XID_Start}_]\p{XID_Continue}*"#, |lex| lex.slice())]
    Ident(&'a str),
}

type ParseError = (String, Span);

pub fn parse(file_path: &Path) -> Result<Vec<Component>> {
    let src = fs::read_to_string(file_path).expect("Failed to read file");

    let mut iter = Token::lexer(&src).spanned();
    let mut comps = Vec::new();

    loop {
        match parse_component(&mut iter) {
            Ok(Some(comp)) => comps.push(comp),
            Ok(None) => break,
            Err((msg, span)) => {
                Report::build(ReportKind::Error, span.clone())
                    .with_message("Parse Error")
                    .with_label(Label::new(span).with_message(msg).with_color(Color::Red))
                    .finish()
                    .print(Source::from(&src))?;
                bail!("Invalid components file")
            }
        }
    }

    Ok(comps)
}

fn next_significant<'a>(
    iter: &mut SpannedIter<'a, Token<'a>>,
) -> Result<Option<(Token<'a>, Span, String)>, ParseError> {
    let mut docs = Vec::new();
    let mut saw_newline = false;

    for (res, span) in iter.by_ref() {
        let token = res.map_err(|_| ("Invalid token".to_string(), span.clone()))?;

        match token {
            Token::NewLine => {
                if saw_newline {
                    docs.clear();
                }
                saw_newline = true;
            }
            Token::CommentLine(text) => {
                docs.push(text);
                saw_newline = false;
            }
            other => return Ok(Some((other, span, docs.join("\n")))),
        }
    }

    Ok(None)
}

fn parse_component<'a>(
    iter: &mut SpannedIter<'a, Token<'a>>,
) -> Result<Option<Component>, ParseError> {
    let (name_tok, name_span, doc) = match next_significant(iter)? {
        Some(t) => t,
        None => return Ok(None),
    };

    let name = match name_tok {
        Token::Ident(n) => n.to_string(),
        t => return Err((format!("Expected Identifier, got {:?}", t), name_span)),
    };

    let (type_tok, type_span, _) = match next_significant(iter)? {
        Some(t) => t,
        None => return Err(("Expected type specifier, found EOF".to_string(), name_span)),
    };

    let comp = match type_tok {
        Token::None => Component::Marker { name, doc },
        Token::Int => Component::Single {
            name,
            doc,
            ty: IglooType::Int,
        },
        Token::Float => Component::Single {
            name,
            doc,
            ty: IglooType::Float,
        },
        Token::Bool => Component::Single {
            name,
            doc,
            ty: IglooType::Bool,
        },
        Token::String => Component::Single {
            name,
            doc,
            ty: IglooType::String,
        },
        Token::Color => Component::Single {
            name,
            doc,
            ty: IglooType::Color,
        },
        Token::Date => Component::Single {
            name,
            doc,
            ty: IglooType::Date,
        },
        Token::Time => Component::Single {
            name,
            doc,
            ty: IglooType::Time,
        },
        Token::IntList => Component::Single {
            name,
            doc,
            ty: IglooType::IntList,
        },
        Token::FloatList => Component::Single {
            name,
            doc,
            ty: IglooType::FloatList,
        },
        Token::BoolList => Component::Single {
            name,
            doc,
            ty: IglooType::BoolList,
        },
        Token::StringList => Component::Single {
            name,
            doc,
            ty: IglooType::StringList,
        },
        Token::ColorList => Component::Single {
            name,
            doc,
            ty: IglooType::ColorList,
        },
        Token::DateList => Component::Single {
            name,
            doc,
            ty: IglooType::DateList,
        },
        Token::TimeList => Component::Single {
            name,
            doc,
            ty: IglooType::TimeList,
        },
        Token::Enum => parse_enum_body(iter, name, doc)?,
        t => {
            return Err((
                format!("Expected type (int, float, enum...), got {:?}", t),
                type_span,
            ));
        }
    };

    Ok(Some(comp))
}

fn parse_enum_body<'a>(
    iter: &mut SpannedIter<'a, Token<'a>>,
    name: String,
    doc: String,
) -> Result<Component, ParseError> {
    match next_significant(iter)? {
        Some((Token::LBrace, _, _)) => {}
        Some((t, span, _)) => return Err((format!("Expected '{{', got {:?}", t), span)),
        None => return Err(("Expected '{{', found EOF".to_string(), Span::default())),
    };

    let mut vars = Vec::new();

    loop {
        let (tok, span, vdoc) = match next_significant(iter)? {
            Some(t) => t,
            None => return Err(("Unexpected EOF inside enum".to_string(), Span::default())),
        };

        match tok {
            Token::Ident(vname) => vars.push(Variant {
                name: vname.to_string(),
                doc: vdoc,
            }),
            Token::RBrace => break,
            t => return Err((format!("Expected Variant or '}}', got {:?}", t), span)),
        }
    }

    Ok(Component::Enum { name, doc, vars })
}

impl Component {
    pub fn get_doc(&self) -> &str {
        use Component::*;
        match self {
            Marker { doc, .. } | Single { doc, .. } | Enum { doc, .. } => doc,
        }
    }
}
