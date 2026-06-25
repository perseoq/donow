use crate::ast::*;
use crate::error::DonowError;
use crate::token::{Spanned, Token};

pub struct Parser {
    tokens: Vec<Spanned<Token>>,
    pos: usize,
    source: Vec<u8>,
    line_starts: Vec<usize>,
}

impl Parser {
    pub fn new(tokens: Vec<Spanned<Token>>, source: &str) -> Self {
        let line_starts = Self::compute_line_starts(source);
        Parser {
            tokens,
            pos: 0,
            source: source.as_bytes().to_vec(),
            line_starts,
        }
    }

    fn compute_line_starts(source: &str) -> Vec<usize> {
        let mut starts = vec![0];
        for (i, c) in source.char_indices() {
            if c == '\n' {
                starts.push(i + 1);
            }
        }
        starts
    }

    fn byte_offset(&self, line: usize, col: usize) -> usize {
        let line_idx = line.checked_sub(1).unwrap_or(0);
        self.line_starts
            .get(line_idx)
            .copied()
            .unwrap_or(0)
            .checked_add(col.checked_sub(1).unwrap_or(0))
            .unwrap_or(0)
    }

    fn slice(&self, start: usize, end: usize) -> &str {
        let s = if end > self.source.len() {
            &self.source[start..]
        } else if start < end {
            &self.source[start..end]
        } else {
            return "";
        };
        std::str::from_utf8(s).unwrap_or("")
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos).map(|s| &s.token)
    }

    fn peek_spanned(&self) -> Option<&Spanned<Token>> {
        self.tokens.get(self.pos)
    }

    fn peek_at(&self, offset: usize) -> Option<&Token> {
        self.tokens.get(self.pos + offset).map(|s| &s.token)
    }

    fn advance(&mut self) -> Option<&Spanned<Token>> {
        let t = self.tokens.get(self.pos);
        if t.is_some() {
            self.pos += 1;
        }
        t
    }

    fn expect(&mut self, expected: &Token) -> Result<(), DonowError> {
        match self.peek() {
            Some(t) if t == expected => {
                self.advance();
                Ok(())
            }
            Some(t) => {
                let s = self.peek_spanned().unwrap();
                Err(DonowError::new(
                    s.line,
                    s.col,
                    format!("expected {}, found {}", expected, t),
                ))
            }
            None => Err(DonowError::new(0, 0, "unexpected end of input")),
        }
    }

    fn expect_ident(&mut self) -> Result<String, DonowError> {
        match self.peek() {
            Some(Token::Ident(s)) => {
                let s = s.clone();
                self.advance();
                Ok(s)
            }
            Some(t) => {
                let s = self.peek_spanned().unwrap();
                Err(DonowError::new(
                    s.line,
                    s.col,
                    format!("expected identifier, found {}", t),
                ))
            }
            None => Err(DonowError::new(0, 0, "unexpected end of input")),
        }
    }

    fn consume_newline(&mut self) {
        if self.peek() == Some(&Token::Newline) {
            self.advance();
        }
    }

    // ----------------------------------------------------------------
    //  MAIN ENTRY
    // ----------------------------------------------------------------

    pub fn parse(&mut self) -> Result<Program, DonowError> {
        let mut blocks = Vec::new();
        loop {
            match self.peek() {
                Some(Token::Eof) => break,
                Some(Token::Newline) => {
                    self.advance();
                }
                _ => {
                    let block = self.parse_block()?;
                    blocks.push(block);
                }
            }
        }
        Ok(Program { blocks })
    }

    // ----------------------------------------------------------------
    //  BLOCKS
    // ----------------------------------------------------------------

    fn parse_block(&mut self) -> Result<Block, DonowError> {
        let name = self.expect_ident()?;
        let span_line = self
            .peek_spanned()
            .map(|s| s.line)
            .unwrap_or(0);
        let span_col = self
            .peek_spanned()
            .map(|s| s.col)
            .unwrap_or(0);
        self.expect(&Token::Colon)?;
        if self.peek() == Some(&Token::Newline) {
            self.advance();
        }
        let body = self.parse_body()?;
        Ok(Block {
            name,
            body,
            span: (span_line, span_col),
        })
    }

    fn parse_body(&mut self) -> Result<Vec<Stmt>, DonowError> {
        match self.peek() {
            Some(Token::Indent) => {
                self.advance();
                let mut stmts = Vec::new();
                loop {
                    match self.peek() {
                        Some(Token::Dedent) => {
                            self.advance();
                            break;
                        }
                        Some(Token::Eof) => break,
                        Some(Token::Newline) => {
                            self.advance();
                        }
                        _ => {
                            let stmt = self.parse_stmt()?;
                            stmts.push(stmt);
                        }
                    }
                }
                Ok(stmts)
            }
            Some(Token::LBrace) => {
                self.advance();
                let mut stmts = Vec::new();
                loop {
                    match self.peek() {
                        Some(Token::RBrace) => {
                            self.advance();
                            break;
                        }
                        Some(Token::Eof) => {
                            return Err(DonowError::new(0, 0, "unclosed brace block"));
                        }
                        Some(Token::Newline) => {
                            self.advance();
                        }
                        _ => {
                            let stmt = self.parse_stmt_in_braces()?;
                            stmts.push(stmt);
                        }
                    }
                }
                Ok(stmts)
            }
            _ => Ok(vec![]),
        }
    }

    fn parse_stmt_in_braces(&mut self) -> Result<Stmt, DonowError> {
        match self.peek() {
            Some(Token::If) => self.parse_if(),
            Some(Token::Else) => self.parse_else(),
            Some(Token::While) => self.parse_while(),
            Some(Token::For) => self.parse_for(),
            Some(Token::LBrace) => self.parse_brace_block(),
            Some(Token::LParen) => self.parse_priority_block(),
            Some(Token::LBracket) => self.parse_deferred_block(),
            Some(Token::Newline) => {
                self.advance();
                self.parse_stmt_in_braces()
            }
            Some(Token::Dedent) | Some(Token::RBrace) | Some(Token::Eof) => {
                Err(DonowError::new(0, 0, "unexpected end of block"))
            }
            _ => {
                if self.is_expression_statement() {
                    self.parse_expression_statement(true)
                } else {
                    self.parse_command()
                }
            }
        }
    }

    // ----------------------------------------------------------------
    //  STATEMENTS
    // ----------------------------------------------------------------

    fn parse_stmt(&mut self) -> Result<Stmt, DonowError> {
        match self.peek() {
            Some(Token::If) => self.parse_if(),
            Some(Token::Else) => self.parse_else(),
            Some(Token::While) => self.parse_while(),
            Some(Token::For) => self.parse_for(),
            Some(Token::LBrace) => self.parse_brace_block(),
            Some(Token::LParen) => self.parse_priority_block(),
            Some(Token::LBracket) => self.parse_deferred_block(),
            Some(Token::Newline) => {
                self.advance();
                self.parse_stmt()
            }
            Some(Token::Dedent) | Some(Token::RBrace) | Some(Token::Eof) => {
                Err(DonowError::new(
                    self.tokens[self.pos].line,
                    self.tokens[self.pos].col,
                    "unexpected end of block",
                ))
            }
            _ => {
                if self.is_expression_statement() {
                    self.parse_expression_statement(false)
                } else {
                    self.parse_command()
                }
            }
        }
    }

    fn is_expression_statement(&self) -> bool {
        match self.peek() {
            Some(Token::Dollar) | Some(Token::Percent) | Some(Token::At) => {
                for i in 0..6 {
                    match self.peek_at(i) {
                        Some(Token::Assign)
                        | Some(Token::Colon)
                        | Some(Token::Eq)
                        | Some(Token::Lt)
                        | Some(Token::Gt)
                        | Some(Token::Lte)
                        | Some(Token::Gte)
                        | Some(Token::NeqGt)
                        | Some(Token::NeqLt)
                        | Some(Token::Not)
                        | Some(Token::Or)
                        | Some(Token::And)
                        | Some(Token::Plus)
                        | Some(Token::Star)
                        | Some(Token::Slash) => return true,
                        Some(Token::Newline) | Some(Token::Dedent) | None => return false,
                        _ => {}
                    }
                }
                false
            }
            Some(Token::Ident(_)) => {
                for i in 0..6 {
                    match self.peek_at(i) {
                        Some(Token::Assign) | Some(Token::Colon) => return true,
                        Some(Token::Newline) | Some(Token::Dedent) | None => return false,
                        _ => {}
                    }
                }
                false
            }
            Some(Token::Number(_))
            | Some(Token::Str(_))
            | Some(Token::LParen)
            | Some(Token::Not)
            | Some(Token::Plus) => {
                for i in 0..6 {
                    match self.peek_at(i) {
                        Some(Token::Colon)
                        | Some(Token::Eq)
                        | Some(Token::Lt)
                        | Some(Token::Gt)
                        | Some(Token::Lte)
                        | Some(Token::Gte)
                        | Some(Token::NeqGt)
                        | Some(Token::NeqLt)
                        | Some(Token::Plus)
                        | Some(Token::Star)
                        | Some(Token::Slash)
                        | Some(Token::Or)
                        | Some(Token::And) => return true,
                        Some(Token::Assign) => return false,
                        Some(Token::Newline) | Some(Token::Dedent) | None => return false,
                        _ => {}
                    }
                }
                false
            }
            _ => false,
        }
    }

    fn parse_expression_statement(&mut self, _inside_braces: bool) -> Result<Stmt, DonowError> {
        let start = self.peek_spanned().cloned().unwrap();
        let start_byte = self.byte_offset(start.line, start.col);
        let expr = self.parse_expr()?;
        let span = (start.line, start.col);

        match self.peek() {
            Some(Token::Assign) => {
                self.advance();
                let value = self.parse_expr()?;
                let name = match &expr {
                    Expr::Ident(n, _) => n.clone(),
                    Expr::VarRef(n, _) => n.clone(),
                    _ => {
                        return Err(DonowError::new(
                            start.line,
                            start.col,
                            "invalid assignment target",
                        ))
                    }
                };
                self.consume_newline();
                Ok(Stmt::Assign {
                    name,
                    value: Box::new(value),
                    span,
                })
            }
            Some(Token::Colon) => {
                self.advance();
                let var_expr = self.parse_expr()?;
                let var_name = match &var_expr {
                    Expr::VarRef(n, _) => n.clone(),
                    _ => {
                        return Err(DonowError::new(
                            start.line,
                            start.col,
                            "colon-assign target must be $var",
                        ))
                    }
                };
                self.consume_newline();
                Ok(Stmt::ColonAssign {
                    target: Box::new(expr),
                    var_name,
                    span,
                })
            }
            _ => {
                let end_byte = self
                    .peek_spanned()
                    .map(|s| self.byte_offset(s.line, s.col))
                    .unwrap_or(self.source.len());
                self.consume_newline();
                let text = self.slice(start_byte, end_byte).trim().to_string();
                Ok(Stmt::Command(text, span))
            }
        }
    }

    // ----------------------------------------------------------------
    //  CONTROL FLOW
    // ----------------------------------------------------------------

    fn parse_if(&mut self) -> Result<Stmt, DonowError> {
        let start = self.peek_spanned().cloned().unwrap();
        self.advance();
        let cond = self.parse_expr()?;
        if self.peek() == Some(&Token::Colon) {
            self.advance();
        }
        if self.peek() == Some(&Token::Newline) {
            self.advance();
        }
        let body = self.parse_body()?;

        let else_body = if self.peek() == Some(&Token::Else) {
            self.advance();
            if self.peek() == Some(&Token::Colon) {
                self.advance();
            }
            if self.peek() == Some(&Token::Newline) {
                self.advance();
            }
            Some(self.parse_body()?)
        } else {
            None
        };

        Ok(Stmt::If {
            cond: Box::new(cond),
            body,
            else_body,
            span: (start.line, start.col),
        })
    }

    fn parse_else(&mut self) -> Result<Stmt, DonowError> {
        let start = self.peek_spanned().cloned().unwrap();
        self.advance();
        if self.peek() == Some(&Token::Colon) {
            self.advance();
        }
        if self.peek() == Some(&Token::Newline) {
            self.advance();
        }
        let body = self.parse_body()?;
        Ok(Stmt::If {
            cond: Box::new(Expr::Bool(true, (start.line, start.col))),
            body,
            else_body: None,
            span: (start.line, start.col),
        })
    }

    fn parse_while(&mut self) -> Result<Stmt, DonowError> {
        let start = self.peek_spanned().cloned().unwrap();
        self.advance();
        let cond = self.parse_expr()?;
        if self.peek() == Some(&Token::Colon) {
            self.advance();
        }
        if self.peek() == Some(&Token::Newline) {
            self.advance();
        }
        let body = self.parse_body()?;
        Ok(Stmt::While {
            cond: Box::new(cond),
            body,
            span: (start.line, start.col),
        })
    }

    fn parse_for(&mut self) -> Result<Stmt, DonowError> {
        let start = self.peek_spanned().cloned().unwrap();
        self.advance();
        let var = match self.peek() {
            Some(Token::Dollar) => {
                self.advance();
                self.expect_ident()?
            }
            _ => self.expect_ident()?,
        };
        if self.peek() == Some(&Token::In) {
            self.advance();
        }
        let iter = self.parse_expr()?;
        if self.peek() == Some(&Token::Colon) {
            self.advance();
        }
        if self.peek() == Some(&Token::Newline) {
            self.advance();
        }
        let body = self.parse_body()?;
        Ok(Stmt::For {
            var,
            iter: Box::new(iter),
            body,
            span: (start.line, start.col),
        })
    }

    // ----------------------------------------------------------------
    //  BLOCKS: brace / priority / deferred
    // ----------------------------------------------------------------

    fn parse_brace_block(&mut self) -> Result<Stmt, DonowError> {
        let start = self.peek_spanned().cloned().unwrap();
        self.advance();
        let mut stmts = Vec::new();
        loop {
            match self.peek() {
                Some(Token::RBrace) => {
                    self.advance();
                    break;
                }
                Some(Token::Eof) => {
                    return Err(DonowError::new(start.line, start.col, "unclosed brace block"));
                }
                Some(Token::Newline) => {
                    self.advance();
                }
                _ => {
                    let stmt = self.parse_stmt_in_braces()?;
                    stmts.push(stmt);
                }
            }
        }
        self.consume_newline();
        Ok(Stmt::BraceBlock(stmts, (start.line, start.col)))
    }

    fn parse_priority_block(&mut self) -> Result<Stmt, DonowError> {
        let start = self.peek_spanned().cloned().unwrap();
        self.advance();
        let text = self.read_until_balanced(Token::LParen, Token::RParen)?;
        Ok(Stmt::PriorityBlock(
            vec![Stmt::Command(text, (start.line, start.col))],
            (start.line, start.col),
        ))
    }

    fn parse_deferred_block(&mut self) -> Result<Stmt, DonowError> {
        let start = self.peek_spanned().cloned().unwrap();
        self.advance();
        let text = self.read_until_balanced(Token::LBracket, Token::RBracket)?;
        Ok(Stmt::DeferredBlock(
            vec![Stmt::Command(text, (start.line, start.col))],
            (start.line, start.col),
        ))
    }

    fn read_until_balanced(&mut self, open: Token, close: Token) -> Result<String, DonowError> {
        let start = self.tokens[self.pos - 1].clone();
        let open_byte = self.byte_offset(start.line, start.col) + 1;
        let mut depth = 1;

        while self.pos < self.tokens.len() {
            let t = &self.tokens[self.pos];
            let t_byte = self.byte_offset(t.line, t.col);
            if t.token == open {
                depth += 1;
            } else if t.token == close {
                depth -= 1;
                if depth == 0 {
                    let end_byte = t_byte;
                    self.advance();
                    let text = self.slice(open_byte, end_byte).trim().to_string();
                    self.consume_newline();
                    return Ok(text);
                }
            }
            self.pos += 1;
        }

        Err(DonowError::new(
            start.line,
            start.col,
            format!("unclosed block"),
        ))
    }

    // ----------------------------------------------------------------
    //  COMMANDS  (fallback: any non-expression line)
    // ----------------------------------------------------------------

    fn parse_command(&mut self) -> Result<Stmt, DonowError> {
        let first = self.peek_spanned().cloned().unwrap();
        let start_byte = self.byte_offset(first.line, first.col);

        loop {
            match self.peek() {
                Some(Token::Newline) => {
                    let end_byte = self.byte_offset(
                        self.tokens[self.pos].line,
                        self.tokens[self.pos].col,
                    );
                    self.advance();
                    let text = self.slice(start_byte, end_byte).trim().to_string();
                    return Ok(Stmt::Command(text, (first.line, first.col)));
                }
                Some(Token::Dedent) | Some(Token::RBrace) | Some(Token::Eof) => {
                    let end_byte = if self.pos > 0 {
                        let p = &self.tokens[self.pos - 1];
                        self.byte_offset(p.line, p.col) + token_source_len(&p.token)
                    } else {
                        start_byte + 1
                    };
                    let text = self.slice(start_byte, end_byte).trim().to_string();
                    return Ok(Stmt::Command(text, (first.line, first.col)));
                }
                _ => {
                    self.advance();
                }
            }
        }
    }

    // ----------------------------------------------------------------
    //  EXPRESSION PARSER  (Pratt / precedence climbing)
    // ----------------------------------------------------------------

    fn parse_expr(&mut self) -> Result<Expr, DonowError> {
        self.parse_binary(0)
    }

    fn parse_binary(&mut self, min_prec: u8) -> Result<Expr, DonowError> {
        let mut lhs = self.parse_prefix()?;

        loop {
            // Postfix operators (highest precedence)
            if self.peek() == Some(&Token::LBracket) {
                // Index access: expr[expr]
                let span = {
                    let s = self.peek_spanned().unwrap();
                    (s.line, s.col)
                };
                self.advance();
                let index = self.parse_expr()?;
                self.expect(&Token::RBracket)?;
                let lhs_span = lhs.span();
                lhs = Expr::Index {
                    arr: Box::new(lhs),
                    index: Box::new(index),
                    span: (span.0.min(lhs_span.0), span.1),
                };
                continue;
            }

            if self.peek() == Some(&Token::Dot) {
                let span = {
                    let s = self.peek_spanned().unwrap();
                    (s.line, s.col)
                };
                self.advance();
                let field = self.expect_ident()?;
                lhs = Expr::DotAccess {
                    obj: Box::new(lhs),
                    field,
                    span,
                };
                continue;
            }

            // Infix operators
            match self.peek() {
                Some(tok) if tok.is_binary_op() => {
                    let (prec, assoc) = tok.precedence();
                    if prec < min_prec {
                        break;
                    }
                    let op_token = tok.clone();
                    let span = {
                        let s = self.peek_spanned().unwrap();
                        (s.line, s.col)
                    };
                    self.advance();
                    let next_min = if assoc == Assoc::Left {
                        prec + 1
                    } else {
                        prec
                    };
                    let rhs = self.parse_binary(next_min)?;
                    let op = token_to_binop(&op_token, span)?;
                    let lhs_span = lhs.span();
                    lhs = Expr::BinOp {
                        left: Box::new(lhs),
                        op,
                        right: Box::new(rhs),
                        span: (lhs_span.0, span.1),
                    };
                }
                _ => break,
            }
        }

        Ok(lhs)
    }

    fn parse_prefix(&mut self) -> Result<Expr, DonowError> {
        match self.peek() {
            Some(Token::Not) => {
                let not_span = {
                    let s = self.peek_spanned().unwrap();
                    (s.line, s.col)
                };
                self.advance();
                let expr = self.parse_binary(10)?;
                Ok(Expr::UnaryOp {
                    op: UnaryOp::Not,
                    expr: Box::new(expr),
                    span: not_span,
                })
            }
            Some(Token::LParen) => {
                self.advance();
                let expr = self.parse_expr()?;
                self.expect(&Token::RParen)?;
                Ok(expr)
            }
            Some(Token::Number(n)) => {
                let start = self.peek_spanned().cloned().unwrap();
                let n = *n;
                self.advance();
                Ok(Expr::Number(n, (start.line, start.col)))
            }
            Some(Token::Str(s)) => {
                let start = self.peek_spanned().cloned().unwrap();
                let s = s.clone();
                self.advance();
                Ok(Expr::String(s, (start.line, start.col)))
            }
            Some(Token::Dollar) => {
                let start = self.peek_spanned().cloned().unwrap();
                self.advance();
                let name = self.expect_ident()?;
                Ok(Expr::VarRef(name, (start.line, start.col)))
            }
            Some(Token::Percent) => {
                let start = self.peek_spanned().cloned().unwrap();
                self.advance();
                let name = self.expect_ident()?;
                Ok(Expr::ParamRef(name, (start.line, start.col)))
            }
            Some(Token::At) => {
                let start = self.peek_spanned().cloned().unwrap();
                self.advance();
                let name = self.expect_ident()?;
                Ok(Expr::CliParam(name, (start.line, start.col)))
            }
            Some(Token::Ident(s)) => {
                let start = self.peek_spanned().cloned().unwrap();
                let name = s.clone();
                self.advance();

                match name.as_str() {
                    "true" => Ok(Expr::Bool(true, (start.line, start.col))),
                    "false" => Ok(Expr::Bool(false, (start.line, start.col))),
                    "T" if self.peek() == Some(&Token::Lt) => self.parse_template(),
                    "F" if self.peek() == Some(&Token::Lt) => self.parse_func_call(),
                    "C" if self.peek() == Some(&Token::Lt) => self.parse_class_ref(),
                    "a" if self.peek() == Some(&Token::LBracket) => self.parse_array(),
                    "l" if self.peek() == Some(&Token::LBracket) => self.parse_list(),
                    "d" if self.peek() == Some(&Token::LBracket) => self.parse_dict(),
                    _ => Ok(Expr::Ident(name, (start.line, start.col))),
                }
            }
            Some(tok) => {
                let s = self.peek_spanned().unwrap();
                Err(DonowError::new(
                    s.line,
                    s.col,
                    format!("unexpected token '{}' in expression", tok),
                ))
            }
            None => Err(DonowError::new(0, 0, "unexpected end of input in expression")),
        }
    }

    // --- array / list / dict literals ---

    fn parse_array(&mut self) -> Result<Expr, DonowError> {
        let start = self.peek_spanned().cloned().unwrap();
        self.advance(); // consume '[' (a was already consumed by parse_prefix)
        let mut elems = Vec::new();
        loop {
            match self.peek() {
                Some(Token::RBracket) => {
                    self.advance();
                    break;
                }
                Some(Token::Newline) => {
                    self.advance();
                }
                _ => {
                    if !elems.is_empty() && self.peek() == Some(&Token::Comma) {
                        self.advance();
                    }
                    let e = self.parse_expr()?;
                    elems.push(e);
                }
            }
        }
        Ok(Expr::Array(elems, (start.line, start.col)))
    }

    fn parse_list(&mut self) -> Result<Expr, DonowError> {
        let start = self.peek_spanned().cloned().unwrap();
        self.advance(); // consume '['
        let mut elems = Vec::new();
        loop {
            match self.peek() {
                Some(Token::RBracket) => {
                    self.advance();
                    break;
                }
                Some(Token::Newline) => {
                    self.advance();
                }
                _ => {
                    if !elems.is_empty() && self.peek() == Some(&Token::Comma) {
                        self.advance();
                    }
                    let e = self.parse_expr()?;
                    elems.push(e);
                }
            }
        }
        Ok(Expr::List(elems, (start.line, start.col)))
    }

    fn parse_dict(&mut self) -> Result<Expr, DonowError> {
        let start = self.peek_spanned().cloned().unwrap();
        self.advance(); // consume '['
        let mut entries = Vec::new();
        loop {
            match self.peek() {
                Some(Token::RBracket) => {
                    self.advance();
                    break;
                }
                Some(Token::Newline) => {
                    self.advance();
                }
                _ => {
                    if !entries.is_empty() && self.peek() == Some(&Token::Comma) {
                        self.advance();
                    }
                    let key = self.parse_expr()?;
                    self.expect(&Token::Colon)?;
                    let value = self.parse_expr()?;
                    entries.push((key, value));
                }
            }
        }
        Ok(Expr::Dict(entries, (start.line, start.col)))
    }

    // --- template / func / class ---

    fn parse_template(&mut self) -> Result<Expr, DonowError> {
        let start = self.peek_spanned().cloned().unwrap();
        self.advance(); // consume '<'
        let name = self.expect_ident()?;
        self.expect(&Token::Gt)?;
        Ok(Expr::Template {
            name,
            span: (start.line, start.col),
        })
    }

    fn parse_func_call(&mut self) -> Result<Expr, DonowError> {
        let start = self.peek_spanned().cloned().unwrap();
        self.advance(); // consume '<'
        let name = self.expect_ident()?;
        self.expect(&Token::Gt)?;

        let args = if self.peek() == Some(&Token::LParen) {
            self.advance();
            let args = self.parse_args()?;
            self.expect(&Token::RParen)?;
            args
        } else {
            vec![]
        };

        Ok(Expr::FuncCall {
            name,
            args,
            span: (start.line, start.col),
        })
    }

    fn parse_class_ref(&mut self) -> Result<Expr, DonowError> {
        let start = self.peek_spanned().cloned().unwrap();
        self.advance(); // consume '<'
        let name = self.expect_ident()?;
        self.expect(&Token::Gt)?;
        Ok(Expr::ClassRef {
            name,
            span: (start.line, start.col),
        })
    }

    fn parse_args(&mut self) -> Result<Vec<Expr>, DonowError> {
        let mut args = Vec::new();
        loop {
            match self.peek() {
                Some(Token::RParen) | None => break,
                Some(Token::Newline) => {
                    self.advance();
                }
                _ => {
                    if !args.is_empty() && self.peek() == Some(&Token::Comma) {
                        self.advance();
                    }
                    let e = self.parse_expr()?;
                    args.push(e);
                }
            }
        }
        Ok(args)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Assoc {
    Left,
    Right,
}

trait BinOpInfo {
    fn is_binary_op(&self) -> bool;
    fn precedence(&self) -> (u8, Assoc);
}

impl BinOpInfo for Token {
    fn is_binary_op(&self) -> bool {
        matches!(
            self,
            Token::Plus
                | Token::Minus
                | Token::Star
                | Token::Slash
                | Token::Eq
                | Token::Lt
                | Token::Gt
                | Token::Lte
                | Token::Gte
                | Token::NeqGt
                | Token::NeqLt
                | Token::Not
                | Token::Or
                | Token::And
        )
    }

    fn precedence(&self) -> (u8, Assoc) {
        match self {
            Token::Or => (1, Assoc::Left),
            Token::And => (2, Assoc::Left),
            Token::Eq => (3, Assoc::Left),
            Token::Lt | Token::Gt | Token::Lte | Token::Gte | Token::NeqGt | Token::NeqLt => {
                (4, Assoc::Left)
            }
            Token::Plus | Token::Minus => (5, Assoc::Left),
            Token::Star | Token::Slash => (6, Assoc::Left),
            Token::Not => (7, Assoc::Right),
            _ => (0, Assoc::Left),
        }
    }
}

fn token_to_binop(tok: &Token, span: (usize, usize)) -> Result<BinOp, DonowError> {
    match tok {
        Token::Plus => Ok(BinOp::Add),
        Token::Minus => Ok(BinOp::Sub),
        Token::Star => Ok(BinOp::Mul),
        Token::Slash => Ok(BinOp::Div),
        Token::Eq => Ok(BinOp::Eq),
        Token::Lt => Ok(BinOp::Lt),
        Token::Gt => Ok(BinOp::Gt),
        Token::Lte => Ok(BinOp::Lte),
        Token::Gte => Ok(BinOp::Gte),
        Token::NeqGt => Ok(BinOp::NeqGt),
        Token::NeqLt => Ok(BinOp::NeqLt),
        Token::Or => Ok(BinOp::Or),
        Token::And => Ok(BinOp::And),
        Token::Not => Ok(BinOp::Neq), // used as !=
        _ => Err(DonowError::new(
            span.0,
            span.1,
            format!("not a binary operator: {}", tok),
        )),
    }
}

fn token_source_len(tok: &Token) -> usize {
    match tok {
        Token::Ident(s) => s.len(),
        Token::Number(n) => n.to_string().len(),
        Token::Str(s) => s.len() + 2,
        Token::Plus
        | Token::Minus
        | Token::Star
        | Token::Slash
        | Token::Not
        | Token::Question
        | Token::Comma
        | Token::Dot
        | Token::Dollar
        | Token::Percent
        | Token::At
        | Token::Assign
        | Token::Colon
        | Token::LParen
        | Token::RParen
        | Token::LBracket
        | Token::RBracket
        | Token::LBrace
        | Token::RBrace => 1,
        Token::Eq | Token::Lt | Token::Gt => 1,
        Token::Lte | Token::Gte | Token::NeqGt | Token::NeqLt => 2,
        Token::If | Token::In | Token::Or | Token::And => 2,
        Token::Else => 2,
        Token::While | Token::For => 2,
        Token::Indent | Token::Dedent | Token::Newline | Token::Eof => 0,
        Token::Ampersand | Token::Pipe => 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    fn parse(input: &str) -> Result<Program, DonowError> {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize()?;
        let mut parser = Parser::new(tokens, input);
        parser.parse()
    }

    fn assert_block_names(program: &Program, expected: &[&str]) {
        let names: Vec<&str> = program.blocks.iter().map(|b| b.name.as_str()).collect();
        assert_eq!(names, expected);
    }

    fn count_stmts(program: &Program, block_idx: usize) -> usize {
        program.blocks[block_idx].body.len()
    }

    #[test]
    fn single_command_block() {
        let p = parse("build:\n    cargo build\n").unwrap();
        assert_block_names(&p, &["build"]);
        assert_eq!(count_stmts(&p, 0), 1);
        match &p.blocks[0].body[0] {
            Stmt::Command(text, _) => assert_eq!(text, "cargo build"),
            _ => panic!("expected Command"),
        }
    }

    #[test]
    fn multiple_blocks() {
        let p = parse("a:\n    x\ny:\n    z\n").unwrap();
        assert_block_names(&p, &["a", "y"]);
    }

    #[test]
    fn assignment_outside_braces() {
        let p = parse("calc:\n    x = 5\n").unwrap();
        match &p.blocks[0].body[0] {
            Stmt::Assign { name, value, .. } => {
                assert_eq!(name, "x");
                match value.as_ref() {
                    Expr::Number(5, _) => {}
                    _ => panic!("expected Number(5)"),
                }
            }
            _ => panic!("expected Assign"),
        }
    }

    #[test]
    fn assignment_inside_braces() {
        let p = parse("calc:\n    {\n    x = 5\n    }\n").unwrap();
        match &p.blocks[0].body[0] {
            Stmt::BraceBlock(stmts, _) => {
                match &stmts[0] {
                    Stmt::Assign { name, value, .. } => {
                        assert_eq!(name, "x");
                        match value.as_ref() {
                            Expr::Number(5, _) => {}
                            _ => panic!("expected Number(5)"),
                        }
                    }
                    _ => panic!("expected Assign"),
                }
            }
            _ => panic!("expected BraceBlock"),
        }
    }

    #[test]
    fn colon_assign() {
        let p = parse("calc:\n    x + 5 : $var\n").unwrap();
        match &p.blocks[0].body[0] {
            Stmt::ColonAssign { target, var_name, .. } => {
                assert_eq!(var_name, "var");
                match target.as_ref() {
                    Expr::BinOp { op: BinOp::Add, .. } => {}
                    _ => panic!("expected BinOp(Add)"),
                }
            }
            _ => panic!("expected ColonAssign"),
        }
    }

    #[test]
    fn if_else() {
        let p = parse("test:\n    ? if $x == 5:\n        echo ok\n    e?:\n        echo fail\n").unwrap();
        match &p.blocks[0].body[0] {
            Stmt::If { cond, body, else_body, .. } => {
                assert!(matches!(cond.as_ref(), Expr::BinOp { op: BinOp::Eq, .. }));
                assert_eq!(body.len(), 1);
                assert!(else_body.is_some());
                assert_eq!(else_body.as_ref().unwrap().len(), 1);
            }
            _ => panic!("expected If"),
        }
    }

    #[test]
    fn while_loop() {
        let p = parse("test:\n    w! $i < 10:\n        echo $i\n").unwrap();
        match &p.blocks[0].body[0] {
            Stmt::While { cond, body, .. } => {
                assert!(matches!(cond.as_ref(), Expr::BinOp { op: BinOp::Lt, .. }));
                assert_eq!(body.len(), 1);
            }
            _ => panic!("expected While"),
        }
    }

    #[test]
    fn for_loop_with_array() {
        let p = parse("test:\n    f! $i in a[1,2,3]:\n        echo $i\n").unwrap();
        match &p.blocks[0].body[0] {
            Stmt::For { var, iter, body, .. } => {
                assert_eq!(var, "i");
                assert!(matches!(iter.as_ref(), Expr::Array(..)));
                assert_eq!(body.len(), 1);
            }
            _ => panic!("expected For"),
        }
    }

    #[test]
    fn priority_block() {
        let p = parse("test:\n    (git push)\n").unwrap();
        match &p.blocks[0].body[0] {
            Stmt::PriorityBlock(stmts, _) => {
                match &stmts[0] {
                    Stmt::Command(text, _) => assert_eq!(text, "git push"),
                    _ => panic!("expected Command inside Priority"),
                }
            }
            _ => panic!("expected PriorityBlock"),
        }
    }

    #[test]
    fn deferred_block() {
        let p = parse("test:\n    [git push]\n").unwrap();
        match &p.blocks[0].body[0] {
            Stmt::DeferredBlock(stmts, _) => {
                match &stmts[0] {
                    Stmt::Command(text, _) => assert_eq!(text, "git push"),
                    _ => panic!("expected Command inside Deferred"),
                }
            }
            _ => panic!("expected DeferredBlock"),
        }
    }

    #[test]
    fn var_ref() {
        let p = parse("test:\n    echo $name\n").unwrap();
        match &p.blocks[0].body[0] {
            Stmt::Command(text, _) => assert_eq!(text, "echo $name"),
            _ => panic!("expected Command"),
        }
    }

    #[test]
    fn binop_precedence() {
        let p = parse("calc:\n    a + b * c : $r\n").unwrap();
        match &p.blocks[0].body[0] {
            Stmt::ColonAssign { target, .. } => {
                match target.as_ref() {
                    Expr::BinOp { op: BinOp::Add, left, right, .. } => {
                        assert!(matches!(left.as_ref(), Expr::Ident(..)));
                        match right.as_ref() {
                            Expr::BinOp { op: BinOp::Mul, .. } => {} // a + (b * c) ✓
                            _ => panic!("expected Mul inside Add"),
                        }
                    }
                    _ => panic!("expected BinOp(Add)"),
                }
            }
            _ => panic!("expected ColonAssign"),
        }
    }

    #[test]
    fn template_syntax() {
        let p = parse("test:\n    T<header>\n").unwrap();
        match &p.blocks[0].body[0] {
            Stmt::Command(text, _) => assert_eq!(text, "T<header>"),
            _ => panic!("expected Command"),
        }
    }

    #[test]
    fn array_literal_in_command() {
        let p = parse("test:\n    a[1, 2, 3]\n").unwrap();
        match &p.blocks[0].body[0] {
            Stmt::Command(text, _) => assert_eq!(text, "a[1, 2, 3]"),
            _ => panic!("expected Command"),
        }
    }

    #[test]
    fn empty_block() {
        let p = parse("empty:\n").unwrap();
        assert_eq!(p.blocks[0].body.len(), 0);
    }

    #[test]
    fn parse_error_unknown_block() {
        let result = parse("");
        assert!(result.is_ok()); // empty program
        assert_eq!(result.unwrap().blocks.len(), 0);
    }
}

