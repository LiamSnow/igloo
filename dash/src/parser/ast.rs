use either::Either;
use litrs::{FloatLit, IntegerLit, StringLit};
use rustc_hash::FxHashMap;

#[derive(Clone, Debug, PartialEq)]
pub enum Statement<'a> {
    Use {
        path: NamespacePath<'a>,
        r#as: Option<&'a str>,
    },
    Module(&'a str, &'a [Statement<'a>]),

    Const(&'a str, Type<'a>, Expr<'a>),
    LetVar(&'a str, Option<Type<'a>>, Expr<'a>),
    SetVar(VarAccess<'a>, Expr<'a>),
    ModVar(VarAccess<'a>, Opcode, Expr<'a>),

    FnDefn {
        name: &'a str,
        params: &'a [(&'a str, Type<'a>)],
        result: Option<Type<'a>>,
        body: &'a [Statement<'a>],
    },
    StructDefn {
        name: &'a str,
        /// TODO bumpalo type?
        params: FxHashMap<&'a str, Type<'a>>,
    },
    EnumDefn {
        name: &'a str,
        variants: &'a [&'a str],
    },
    TypeDefn(&'a str, Type<'a>),

    DashboardDefn {
        name: StringLit<&'a str>,
        body: &'a [Statement<'a>],
    },
    ElementDefn {
        name: &'a str,
        params: &'a [(&'a str, Type<'a>)],
        body: &'a [Statement<'a>],
    },

    /// bare function call, not assigned to anything
    /// ex. `print("Hello World")`
    FnCall {
        name: VarAccess<'a>,
        params: &'a [Expr<'a>],
    },
    /// element construction
    Element {
        name: &'a str,
        params: &'a [(Option<&'a str>, Type<'a>)],
        body: &'a [Statement<'a>],
    },

    If {
        cond: Expr<'a>,
        body: &'a [Statement<'a>],
        r#else: Option<&'a Else<'a>>,
    },
    For {
        var: &'a str,
        of: Either<Range<'a>, VarAccess<'a>>,
        body: &'a [Statement<'a>],
    },
    While {
        cond: Expr<'a>,
        body: &'a [Statement<'a>],
    },

    Return(Option<Expr<'a>>),
    Break,
    Continue,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Else<'a> {
    If {
        cond: Expr<'a>,
        body: &'a [Statement<'a>],
        r#else: Option<&'a Else<'a>>,
    },
    Else(&'a [Statement<'a>]),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Primitive<'a> {
    Int(IntegerLit<&'a str>),
    Float(FloatLit<&'a str>),
    Bool(bool),
    String(StringLit<&'a str>),
    FString(&'a [Expr<'a>]),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Compound<'a> {
    List(&'a [Expr<'a>]),
    Tuple(&'a [Expr<'a>]),
    // Enum is processed as a Var
    Struct(NamespacePath<'a>, &'a [(&'a str, Option<Expr<'a>>)]),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Expr<'a> {
    Primitive(Primitive<'a>),
    Compound(Compound<'a>),

    Var(VarAccess<'a>),
    FnCall(VarAccess<'a>, &'a [Expr<'a>]),

    Op(&'a Expr<'a>, Opcode, &'a Expr<'a>),
    LogicalNot(&'a Expr<'a>),
    Neg(&'a Expr<'a>),

    Cast(&'a Expr<'a>, Type<'a>),

    Ternary {
        cond: &'a Expr<'a>,
        a: &'a Expr<'a>,
        b: &'a Expr<'a>,
    },

    Query {
        qtype: QueryType,
        comp_type: Option<String>,
        /// TODO bumpalo type?
        params: FxHashMap<String, Expr<'a>>,
        r#where: &'a [Expr<'a>],
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Range<'a> {
    pub start: Expr<'a>,
    pub inclusive: bool,
    pub end: Expr<'a>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Type<'a> {
    Int,
    Float,
    Bool,
    String,
    List(&'a Type<'a>),
    Query(QueryType),
    Tuple(&'a [Type<'a>]),
    /// struct or enum
    User(&'a str),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum QueryType {
    Observe,
    Bind,
    FilterSet,
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

/// example::fn() -> ["example"], "fn"
#[derive(Clone, Debug, PartialEq)]
pub struct NamespacePath<'a> {
    pub ns: &'a [&'a str],
    pub tail: &'a str,
}

/// var         -> "var"
/// cfg.field   -> "cfg"  [Field("cfg")]
/// map["c"]    -> "map"  [Bracket("c")]
/// list[var]   -> "list" [Bracket(var)]
/// list[0]     -> "list" [Bracket(0)]
/// tup.0       -> "tup"  [Tuple(0)]
/// cfg.list[0] -> "cfg"  [Field("list"), Bracket(0)]
#[derive(Clone, Debug, PartialEq)]
pub struct VarAccess<'a> {
    pub var: NamespacePath<'a>,
    pub tail: &'a [CompoundPart<'a>],
}

#[derive(Clone, Debug, PartialEq)]
pub enum CompoundPart<'a> {
    // map["key"] or list[0]
    Bracket(Expr<'a>),
    // tup.2
    Tuple(IntegerLit<&'a str>),
    // struct.field | cfg.field
    Field(&'a str),
}
