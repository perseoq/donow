use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    If,
    Else,
    While,
    For,
    In,
    Or,
    And,

    Ident(String),
    Number(i64),
    Str(String),

    Plus,
    Minus,
    Star,
    Slash,

    Eq,
    Not,
    Gt,
    Lt,
    Gte,
    Lte,
    NeqGt,
    NeqLt,

    Ampersand,
    Pipe,

    Assign,
    Colon,
    Question,
    Comma,
    Dot,

    Dollar,
    Percent,
    At,

    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,

    Indent,
    Dedent,
    Newline,

    Eof,
}

#[derive(Debug, Clone)]
pub struct Spanned<T> {
    pub token: T,
    pub line: usize,
    pub col: usize,
}

impl<T> Spanned<T> {
    pub fn new(token: T, line: usize, col: usize) -> Self {
        Spanned { token, line, col }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::If => write!(f, "If"),
            Token::Else => write!(f, "Else"),
            Token::While => write!(f, "While"),
            Token::For => write!(f, "For"),
            Token::In => write!(f, "In"),
            Token::Or => write!(f, "Or"),
            Token::And => write!(f, "And"),
            Token::Ident(s) => write!(f, "Ident({})", s),
            Token::Number(n) => write!(f, "Number({})", n),
            Token::Str(s) => write!(f, "Str(\"{}\")", s),
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::Star => write!(f, "*"),
            Token::Slash => write!(f, "/"),
            Token::Eq => write!(f, "=="),
            Token::Not => write!(f, "!"),
            Token::Gt => write!(f, ">"),
            Token::Lt => write!(f, "<"),
            Token::Gte => write!(f, ">="),
            Token::Lte => write!(f, "<="),
            Token::NeqGt => write!(f, ">!"),
            Token::NeqLt => write!(f, "<!"),
            Token::Ampersand => write!(f, "&"),
            Token::Pipe => write!(f, "|"),
            Token::Assign => write!(f, "="),
            Token::Colon => write!(f, ":"),
            Token::Question => write!(f, "?"),
            Token::Comma => write!(f, ","),
            Token::Dot => write!(f, "."),
            Token::Dollar => write!(f, "$"),
            Token::Percent => write!(f, "%"),
            Token::At => write!(f, "@"),
            Token::LParen => write!(f, "("),
            Token::RParen => write!(f, ")"),
            Token::LBracket => write!(f, "["),
            Token::RBracket => write!(f, "]"),
            Token::LBrace => write!(f, "{{"),
            Token::RBrace => write!(f, "}}"),
            Token::Indent => write!(f, "Indent"),
            Token::Dedent => write!(f, "Dedent"),
            Token::Newline => write!(f, "Newline"),
            Token::Eof => write!(f, "Eof"),
        }
    }
}
