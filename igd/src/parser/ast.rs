use either::Either;
use litrs::{FloatLit, IntegerLit, StringLit};

#[derive(Clone, Debug, PartialEq)]
pub enum Statement<'a> {
    Use {
        path: NamespacePath<'a>,
        r#as: Option<&'a str>,
    },
    Module {
        name: &'a str,
        body: &'a [Statement<'a>],
    },

    Const {
        name: &'a str,
        r#type: Type<'a>,
        value: Expr<'a>,
    },
    Let {
        name: &'a str,
        r#type: Option<Type<'a>>,
        value: Expr<'a>,
    },
    Assign {
        var: VarAccess<'a>,
        value: Expr<'a>,
    },
    CompoundAssign {
        var: VarAccess<'a>,
        op: Opcode,
        value: Expr<'a>,
    },

    FnDefn {
        name: &'a str,
        params: &'a [ParameterDefn<'a>],
        return_type: Option<Type<'a>>,
        body: &'a [Statement<'a>],
    },
    StructDefn {
        name: &'a str,
        fields: &'a [FieldDefn<'a>],
    },
    EnumDefn {
        name: &'a str,
        variants: &'a [&'a str],
    },
    TypeDefn {
        name: &'a str,
        value: Type<'a>,
    },

    DashboardDefn {
        name: StringLit<&'a str>,
        body: &'a [Statement<'a>],
    },
    ElementDefn {
        name: &'a str,
        params: &'a [ParameterDefn<'a>],
        body: &'a [Statement<'a>],
    },

    /// bare function call, not assigned to anything
    /// ex. `print("Hello World")`
    FnCall {
        name: VarAccess<'a>,
        params: &'a [Expr<'a>],
    },
    /// element construction
    ElementInst {
        name: &'a str,
        params: &'a [NamedParameterInst<'a>],
        body: &'a [Statement<'a>],
    },

    If {
        cond: Expr<'a>,
        body: &'a [Statement<'a>],
        r#else: Option<&'a Else<'a>>,
    },
    For {
        var: &'a str,
        iter: Either<Range<'a>, VarAccess<'a>>,
        body: &'a [Statement<'a>],
    },
    While {
        cond: Expr<'a>,
        body: &'a [Statement<'a>],
    },

    Return(Option<Expr<'a>>),
    Break,
    Continue,

    Error,
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
pub struct ParameterDefn<'a> {
    pub name: &'a str,
    pub r#type: Type<'a>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FieldDefn<'a> {
    pub name: &'a str,
    pub r#type: Type<'a>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NamedParameterInst<'a> {
    pub name: Option<&'a str>,
    pub value: Expr<'a>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FieldInst<'a> {
    pub name: &'a str,
    // None for shorthand
    pub value: Option<Expr<'a>>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Primitive<'a> {
    Int(IntegerLit<&'a str>),
    Float(FloatLit<&'a str>),
    Bool(bool),
    String(StringLit<&'a str>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Compound<'a> {
    List(&'a [Expr<'a>]),
    Tuple(&'a [Expr<'a>]),
    // Enum is processed as a Var
    Struct {
        name: NamespacePath<'a>,
        fields: &'a [FieldInst<'a>],
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum Expr<'a> {
    Primitive(Primitive<'a>),
    Compound(Compound<'a>),

    Var(VarAccess<'a>),
    FnCall {
        name: VarAccess<'a>,
        params: &'a [Expr<'a>],
    },
    FString(&'a [Expr<'a>]),

    Op(&'a Expr<'a>, Opcode, &'a Expr<'a>),
    LogicalNot(&'a Expr<'a>),
    Neg(&'a Expr<'a>),

    Cast {
        value: &'a Expr<'a>,
        to: Type<'a>,
    },

    Ternary {
        cond: &'a Expr<'a>,
        a: &'a Expr<'a>,
        b: &'a Expr<'a>,
    },

    Query {
        query_type: QueryType,
        /// Can bind component name (for component ObserveValue)
        /// or a subset of observe queries:
        ///  1. DeviceName, DeviceAttached
        ///  2. EntityRegistered
        ///  3. ExtensionAttached
        ///  4. GroupName
        target: &'a str,
        params: &'a [QueryParam<'a>],
        r#where: Option<&'a Expr<'a>>,
    },

    Error,
}

#[derive(Clone, Debug, PartialEq)]
pub struct QueryParam<'a> {
    pub name: &'a str,
    pub value: QueryParamValue<'a>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum QueryParamValue<'a> {
    Primitive(Primitive<'a>),
    Ident(&'a str),
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
    Bind,
    Observe,
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
