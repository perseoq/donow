use std::fmt;

pub type Span = (usize, usize);

#[derive(Debug, Clone)]
pub struct Program {
    pub blocks: Vec<Block>,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub name: String,
    pub body: Vec<Stmt>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Assign {
        name: String,
        value: Box<Expr>,
        span: Span,
    },
    ColonAssign {
        target: Box<Expr>,
        var_name: String,
        span: Span,
    },
    If {
        cond: Box<Expr>,
        body: Vec<Stmt>,
        else_body: Option<Vec<Stmt>>,
        span: Span,
    },
    While {
        cond: Box<Expr>,
        body: Vec<Stmt>,
        span: Span,
    },
    For {
        var: String,
        iter: Box<Expr>,
        body: Vec<Stmt>,
        span: Span,
    },
    PriorityBlock(Vec<Stmt>, Span),
    DeferredBlock(Vec<Stmt>, Span),
    BraceBlock(Vec<Stmt>, Span),
    Command(String, Span),
}

#[derive(Debug, Clone)]
pub enum Expr {
    Number(i64, Span),
    String(String, Span),
    Bool(bool, Span),
    Ident(String, Span),
    VarRef(String, Span),
    ParamRef(String, Span),
    CliParam(String, Span),
    Array(Vec<Expr>, Span),
    List(Vec<Expr>, Span),
    Dict(Vec<(Expr, Expr)>, Span),
    BinOp {
        left: Box<Expr>,
        op: BinOp,
        right: Box<Expr>,
        span: Span,
    },
    UnaryOp {
        op: UnaryOp,
        expr: Box<Expr>,
        span: Span,
    },
    Index {
        arr: Box<Expr>,
        index: Box<Expr>,
        span: Span,
    },
    DotAccess {
        obj: Box<Expr>,
        field: String,
        span: Span,
    },
    Template {
        name: String,
        span: Span,
    },
    FuncCall {
        name: String,
        args: Vec<Expr>,
        span: Span,
    },
    ClassRef {
        name: String,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinOp {
    Add, Sub, Mul, Div,
    Eq, Neq, Lt, Gt, Lte, Gte, NeqGt, NeqLt,
    And, Or,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Not,
}

impl Stmt {
    pub fn span(&self) -> Span {
        match self {
            Stmt::Assign { span, .. }
            | Stmt::ColonAssign { span, .. }
            | Stmt::If { span, .. }
            | Stmt::While { span, .. }
            | Stmt::For { span, .. }
            | Stmt::PriorityBlock(_, span)
            | Stmt::DeferredBlock(_, span)
            | Stmt::BraceBlock(_, span)
            | Stmt::Command(_, span) => *span,
        }
    }
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::Number(_, s)
            | Expr::String(_, s)
            | Expr::Bool(_, s)
            | Expr::Ident(_, s)
            | Expr::VarRef(_, s)
            | Expr::ParamRef(_, s)
            | Expr::CliParam(_, s)
            | Expr::Array(_, s)
            | Expr::List(_, s)
            | Expr::Dict(_, s)
            | Expr::BinOp { span: s, .. }
            | Expr::UnaryOp { span: s, .. }
            | Expr::Index { span: s, .. }
            | Expr::DotAccess { span: s, .. }
            | Expr::Template { span: s, .. }
            | Expr::FuncCall { span: s, .. }
            | Expr::ClassRef { span: s, .. } => *s,
        }
    }
}

impl fmt::Display for BinOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinOp::Add => write!(f, "+"),
            BinOp::Sub => write!(f, "-"),
            BinOp::Mul => write!(f, "*"),
            BinOp::Div => write!(f, "/"),
            BinOp::Eq => write!(f, "=="),
            BinOp::Neq => write!(f, "!="),
            BinOp::Lt => write!(f, "<"),
            BinOp::Gt => write!(f, ">"),
            BinOp::Lte => write!(f, "<="),
            BinOp::Gte => write!(f, ">="),
            BinOp::NeqGt => write!(f, ">!"),
            BinOp::NeqLt => write!(f, "<!"),
            BinOp::And => write!(f, "and"),
            BinOp::Or => write!(f, "or"),
        }
    }
}

impl fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UnaryOp::Not => write!(f, "!"),
        }
    }
}
