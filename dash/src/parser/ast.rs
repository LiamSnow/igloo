use std::borrow::Cow;

use either::Either;
use litrs::{FloatLit, IntegerLit};

#[derive(Clone, Debug, PartialEq)]
pub enum GlobalStatement<'source> {
    Include(Cow<'source, str>),
    ElementDefn(
        &'source str,
        Vec<(&'source str, Type<'source>)>,
        Vec<Statement<'source>>,
        Element<'source>,
    ),
    FnDef(
        &'source str,
        Vec<(&'source str, Type<'source>)>,
        Option<Type<'source>>,
        Vec<Statement<'source>>,
    ),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Statement<'source> {
    Bind(&'source str, Option<Type<'source>>, Expr<'source>),
    Const(&'source str, Option<Type<'source>>, Expr<'source>),

    DeclVar(&'source str, Option<Type<'source>>),
    DeclAsgnVar(&'source str, Option<Type<'source>>, Expr<'source>),
    AsgnVar(VarAccess<'source>, Expr<'source>),
    ModVar(VarAccess<'source>, Opcode, Expr<'source>),

    Element(Element<'source>),
    Inject(Expr<'source>),

    Call(VarAccess<'source>, Vec<(&'source str, Expr<'source>)>),

    If(
        Box<Expr<'source>>,
        Vec<Statement<'source>>,
        Option<Box<Else<'source>>>,
    ),
    For(
        &'source str,
        Either<Range<'source>, VarAccess<'source>>,
        Vec<Statement<'source>>,
    ),
    While(Expr<'source>, Vec<Statement<'source>>),
    Loop(Vec<Statement<'source>>),

    Return(Option<Expr<'source>>),
    Break,
    Continue,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Element<'source> {
    pub name: &'source str,
    pub params: Vec<(&'source str, Expr<'source>)>,
    pub body: Vec<Statement<'source>>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Else<'source> {
    ElseIf(
        Expr<'source>,
        Vec<Statement<'source>>,
        Option<Box<Else<'source>>>,
    ),
    Else(Vec<Statement<'source>>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Primitive<'source> {
    Int(IntegerLit<&'source str>),
    Float(FloatLit<&'source str>),
    Bool(bool),
    Char(char),
    Byte(u8),
    String(Cow<'source, str>),
    ByteString(Cow<'source, [u8]>),
    FString(Vec<Expr<'source>>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Expr<'source> {
    Primitive(Primitive<'source>),
    List(Vec<Expr<'source>>),
    Element(Element<'source>),

    Var(VarAccess<'source>),
    Call(VarAccess<'source>, Vec<(&'source str, Expr<'source>)>),

    Op(Box<Expr<'source>>, Opcode, Box<Expr<'source>>),
    LogicalNot(Box<Expr<'source>>),
    Neg(Box<Expr<'source>>),

    Default(Box<Expr<'source>>, Box<Expr<'source>>),
    Unwrap(Box<Expr<'source>>),

    Cast(Box<Expr<'source>>, Type<'source>),

    If(
        Box<Expr<'source>>,
        Vec<Statement<'source>>,
        Option<Box<Else<'source>>>,
    ),

    Lambda(Vec<&'source str>, Vec<Statement<'source>>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Range<'source> {
    pub start: Expr<'source>,
    pub inclusive: bool,
    pub end: Expr<'source>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Type<'source> {
    I8,
    I16,
    I32,
    I64,
    I128,
    ISize,
    U8,
    U16,
    U32,
    U64,
    U128,
    USize,
    F32,
    F64,
    Bool,
    Char,
    Byte,
    String,
    ByteString,
    List(Box<Type<'source>>),
    User(&'source str),
    Fn(
        Vec<(&'source str, Type<'source>)>,
        Option<Box<Type<'source>>>,
    ),
    Option(Box<Type<'source>>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Opcode {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Shl,
    Shr,
    Pow,

    EqEq,
    Neq,
    Lt,
    Le,
    Gt,
    Ge,

    AndAnd,
    OrOr,
    Xor,
    And,
    Or,
}

/// var         -> "var"
/// cfg.field   -> "cfg"  [Field("cfg")]
/// list[var]   -> "list" [Bracket(var)]
/// list[0]     -> "list" [Bracket(0)]
/// cfg.list[0] -> "cfg"  [Field("list"), Bracket(0)]
#[derive(Clone, Debug, PartialEq)]
pub struct VarAccess<'source> {
    pub var: &'source str,
    pub tail: Vec<CompoundPart<'source>>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum CompoundPart<'source> {
    // list[0]
    Bracket(Expr<'source>),
    // struct.field | cfg.field
    Field(&'source str),
}
