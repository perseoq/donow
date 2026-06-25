use crate::error::DonowError;
use crate::token::{Spanned, Token};

pub struct Lexer {
    chars: Vec<char>,
    pos: usize,
    line: usize,
    col: usize,
    indent_stack: Vec<usize>,
    brace_depth: usize,
    at_line_start: bool,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Lexer {
            chars: input.chars().collect(),
            pos: 0,
            line: 1,
            col: 1,
            indent_stack: vec![0],
            brace_depth: 0,
            at_line_start: true,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Spanned<Token>>, DonowError> {
        let mut tokens = Vec::new();

        loop {
            if self.pos >= self.chars.len() {
                break;
            }

            if self.at_line_start && self.brace_depth == 0 {
                match self.process_line_start()? {
                    LineAction::Skip => continue,
                    LineAction::Emit(ts) => {
                        tokens.extend(ts);
                        self.at_line_start = false;
                    }
                    LineAction::Done => {
                        self.at_line_start = false;
                    }
                }
                if self.pos >= self.chars.len() {
                    break;
                }
            }

            self.skip_inline_whitespace();
            if self.pos >= self.chars.len() {
                break;
            }

            let line = self.line;
            let col = self.col;
            let c = self.chars[self.pos];

            match c {
                '\r' => {
                    self.pos += 1;
                    self.col += 1;
                }
                '\n' => {
                    self.pos += 1;
                    self.line += 1;
                    self.col = 1;
                    self.at_line_start = true;
                    tokens.push(Spanned::new(Token::Newline, line, col));
                }
                '#' => {
                    self.skip_to_end_of_line();
                }
                '\'' => {
                    if self.pos + 1 < self.chars.len() && self.chars[self.pos + 1] == '\'' {
                        self.pos += 2;
                        self.col += 2;
                        self.skip_multiline_comment()?;
                    } else {
                        return Err(DonowError::new(
                            line,
                            col,
                            "unexpected single quote, use '' for multiline comments",
                        ));
                    }
                }
                '"' => {
                    self.pos += 1;
                    self.col += 1;
                    tokens.push(self.read_string(line, col)?);
                }
                '0'..='9' => {
                    self.pos += 1;
                    self.col += 1;
                    tokens.push(self.read_number(c, line, col)?);
                }
                'a'..='z' | 'A'..='Z' | '_' => {
                    self.pos += 1;
                    self.col += 1;
                    tokens.push(self.read_identifier_or_keyword(c, line, col)?);
                }
                '+' => {
                    self.pos += 1;
                    self.col += 1;
                    tokens.push(Spanned::new(Token::Plus, line, col));
                }
                '-' => {
                    self.pos += 1;
                    self.col += 1;
                    tokens.push(Spanned::new(Token::Minus, line, col));
                }
                '*' => {
                    self.pos += 1;
                    self.col += 1;
                    tokens.push(Spanned::new(Token::Star, line, col));
                }
                '/' => {
                    self.pos += 1;
                    self.col += 1;
                    tokens.push(Spanned::new(Token::Slash, line, col));
                }
                '=' => {
                    self.pos += 1;
                    self.col += 1;
                    if self.pos < self.chars.len() && self.chars[self.pos] == '=' {
                        self.pos += 1;
                        self.col += 1;
                        tokens.push(Spanned::new(Token::Eq, line, col));
                    } else {
                        tokens.push(Spanned::new(Token::Assign, line, col));
                    }
                }
                '!' => {
                    self.pos += 1;
                    self.col += 1;
                    tokens.push(Spanned::new(Token::Not, line, col));
                }
                ':' => {
                    self.pos += 1;
                    self.col += 1;
                    tokens.push(Spanned::new(Token::Colon, line, col));
                }
                '?' => {
                    self.pos += 1;
                    self.col += 1;
                    self.handle_question(line, col, &mut tokens)?;
                }
                '$' => {
                    self.pos += 1;
                    self.col += 1;
                    tokens.push(Spanned::new(Token::Dollar, line, col));
                }
                '%' => {
                    self.pos += 1;
                    self.col += 1;
                    tokens.push(Spanned::new(Token::Percent, line, col));
                }
                '@' => {
                    self.pos += 1;
                    self.col += 1;
                    tokens.push(Spanned::new(Token::At, line, col));
                }
                '(' => {
                    self.pos += 1;
                    self.col += 1;
                    tokens.push(Spanned::new(Token::LParen, line, col));
                }
                ')' => {
                    self.pos += 1;
                    self.col += 1;
                    tokens.push(Spanned::new(Token::RParen, line, col));
                }
                '[' => {
                    self.pos += 1;
                    self.col += 1;
                    tokens.push(Spanned::new(Token::LBracket, line, col));
                }
                ']' => {
                    self.pos += 1;
                    self.col += 1;
                    tokens.push(Spanned::new(Token::RBracket, line, col));
                }
                '{' => {
                    self.pos += 1;
                    self.col += 1;
                    self.brace_depth += 1;
                    tokens.push(Spanned::new(Token::LBrace, line, col));
                }
                '}' => {
                    self.pos += 1;
                    self.col += 1;
                    self.brace_depth -= 1;
                    tokens.push(Spanned::new(Token::RBrace, line, col));
                }
                '<' => {
                    self.pos += 1;
                    self.col += 1;
                    if self.pos < self.chars.len() && self.chars[self.pos] == '=' {
                        self.pos += 1;
                        self.col += 1;
                        tokens.push(Spanned::new(Token::Lte, line, col));
                    } else if self.pos < self.chars.len() && self.chars[self.pos] == '!' {
                        self.pos += 1;
                        self.col += 1;
                        tokens.push(Spanned::new(Token::NeqLt, line, col));
                    } else {
                        tokens.push(Spanned::new(Token::Lt, line, col));
                    }
                }
                '>' => {
                    self.pos += 1;
                    self.col += 1;
                    if self.pos < self.chars.len() && self.chars[self.pos] == '=' {
                        self.pos += 1;
                        self.col += 1;
                        tokens.push(Spanned::new(Token::Gte, line, col));
                    } else if self.pos < self.chars.len() && self.chars[self.pos] == '!' {
                        self.pos += 1;
                        self.col += 1;
                        tokens.push(Spanned::new(Token::NeqGt, line, col));
                    } else {
                        tokens.push(Spanned::new(Token::Gt, line, col));
                    }
                }
                '&' => {
                    self.pos += 1;
                    self.col += 1;
                    tokens.push(Spanned::new(Token::Ampersand, line, col));
                }
                '|' => {
                    self.pos += 1;
                    self.col += 1;
                    tokens.push(Spanned::new(Token::Pipe, line, col));
                }
                ',' => {
                    self.pos += 1;
                    self.col += 1;
                    tokens.push(Spanned::new(Token::Comma, line, col));
                }
                '.' => {
                    self.pos += 1;
                    self.col += 1;
                    tokens.push(Spanned::new(Token::Dot, line, col));
                }
                _ => {
                    return Err(DonowError::new(
                        line,
                        col,
                        format!("unexpected character '{}'", c),
                    ));
                }
            }
        }

        while self.indent_stack.len() > 1 {
            self.indent_stack.pop().unwrap();
            tokens.push(Spanned::new(
                Token::Dedent,
                self.line,
                self.col,
            ));
        }

        tokens.push(Spanned::new(Token::Eof, self.line, self.col));
        Ok(tokens)
    }

    fn process_line_start(&mut self) -> Result<LineAction, DonowError> {
        let mut spaces = 0;
        while self.pos < self.chars.len() {
            let c = self.chars[self.pos];
            if c == ' ' {
                spaces += 1;
                self.pos += 1;
            } else if c == '\t' {
                spaces += 4;
                self.pos += 1;
            } else {
                break;
            }
        }
        self.col = 1 + spaces;

        if self.pos >= self.chars.len() {
            return Ok(LineAction::Done);
        }

        let c = self.chars[self.pos];

        if c == '\n' {
            self.pos += 1;
            self.line += 1;
            self.col = 1;
            return Ok(LineAction::Skip);
        }

        if c == '#' {
            self.skip_to_end_of_line();
            return Ok(LineAction::Skip);
        }

        if c == '\'' && self.pos + 1 < self.chars.len() && self.chars[self.pos + 1] == '\'' {
            self.pos += 2;
            self.col += 2;
            self.skip_multiline_comment()?;
            return Ok(LineAction::Skip);
        }

        let current_indent = *self.indent_stack.last().unwrap();
        let mut ts = Vec::new();

        if spaces > current_indent {
            self.indent_stack.push(spaces);
            ts.push(Spanned::new(Token::Indent, self.line, self.col));
        } else if spaces < current_indent {
            while self.indent_stack.len() > 1
                && *self.indent_stack.last().unwrap() > spaces
            {
                self.indent_stack.pop();
                ts.push(Spanned::new(Token::Dedent, self.line, self.col));
            }
            if *self.indent_stack.last().unwrap() != spaces {
                return Err(DonowError::new(
                    self.line,
                    self.col,
                    format!(
                        "inconsistent indentation (expected {} spaces)",
                        self.indent_stack.last().unwrap()
                    ),
                ));
            }
        }

        if ts.is_empty() {
            Ok(LineAction::Done)
        } else {
            Ok(LineAction::Emit(ts))
        }
    }

    fn handle_question(
        &mut self,
        line: usize,
        col: usize,
        tokens: &mut Vec<Spanned<Token>>,
    ) -> Result<(), DonowError> {
        let saved = (self.pos, self.col);

        while self.pos < self.chars.len() && self.chars[self.pos] == ' ' {
            self.pos += 1;
            self.col += 1;
        }

        let mut word = String::new();
        while self.pos < self.chars.len() && self.chars[self.pos].is_alphabetic() {
            word.push(self.chars[self.pos]);
            self.pos += 1;
            self.col += 1;
        }

        if word == "if" {
            tokens.push(Spanned::new(Token::If, line, col));
            Ok(())
        } else {
            self.pos = saved.0;
            self.col = saved.1;
            tokens.push(Spanned::new(Token::Question, line, col));
            Ok(())
        }
    }

    fn read_string(&mut self, line: usize, col: usize) -> Result<Spanned<Token>, DonowError> {
        let mut s = String::new();
        loop {
            if self.pos >= self.chars.len() {
                return Err(DonowError::new(line, col, "unterminated string literal"));
            }
            let c = self.chars[self.pos];
            self.pos += 1;
            self.col += 1;
            match c {
                '"' => return Ok(Spanned::new(Token::Str(s), line, col)),
                '\n' => {
                    self.line += 1;
                    self.col = 1;
                    s.push(c);
                }
                _ => {
                    s.push(c);
                }
            }
        }
    }

    fn read_number(
        &mut self,
        first: char,
        line: usize,
        col: usize,
    ) -> Result<Spanned<Token>, DonowError> {
        let mut s = String::new();
        s.push(first);
        while self.pos < self.chars.len() && self.chars[self.pos].is_ascii_digit() {
            s.push(self.chars[self.pos]);
            self.pos += 1;
            self.col += 1;
        }
        let n: i64 = s.parse().unwrap();
        Ok(Spanned::new(Token::Number(n), line, col))
    }

    fn read_identifier_or_keyword(
        &mut self,
        first: char,
        line: usize,
        col: usize,
    ) -> Result<Spanned<Token>, DonowError> {
        if first == 'e' && self.chars.get(self.pos) == Some(&'?') {
            self.pos += 1;
            self.col += 1;
            return Ok(Spanned::new(Token::Else, line, col));
        }
        if first == 'w' && self.chars.get(self.pos) == Some(&'!') {
            self.pos += 1;
            self.col += 1;
            return Ok(Spanned::new(Token::While, line, col));
        }
        if first == 'f' && self.chars.get(self.pos) == Some(&'!') {
            self.pos += 1;
            self.col += 1;
            return Ok(Spanned::new(Token::For, line, col));
        }

        let mut s = String::new();
        s.push(first);
        while self.pos < self.chars.len() {
            let c = self.chars[self.pos];
            if c.is_alphanumeric() || c == '_' || c == '-' {
                s.push(c);
                self.pos += 1;
                self.col += 1;
            } else {
                break;
            }
        }

        match s.as_str() {
            "if" => Ok(Spanned::new(Token::If, line, col)),
            "in" => Ok(Spanned::new(Token::In, line, col)),
            "or" => Ok(Spanned::new(Token::Or, line, col)),
            "and" => Ok(Spanned::new(Token::And, line, col)),
            _ => Ok(Spanned::new(Token::Ident(s), line, col)),
        }
    }

    fn skip_inline_whitespace(&mut self) {
        while self.pos < self.chars.len()
            && (self.chars[self.pos] == ' ' || self.chars[self.pos] == '\t')
        {
            self.pos += 1;
            self.col += 1;
        }
    }

    fn skip_to_end_of_line(&mut self) {
        while self.pos < self.chars.len() && self.chars[self.pos] != '\n' {
            self.pos += 1;
            self.col += 1;
        }
    }

    fn skip_multiline_comment(&mut self) -> Result<(), DonowError> {
        loop {
            if self.pos >= self.chars.len() {
                return Err(DonowError::new(self.line, self.col, "unterminated multiline comment"));
            }
            let c = self.chars[self.pos];
            self.pos += 1;
            match c {
                '\'' => {
                    if self.pos < self.chars.len() && self.chars[self.pos] == '\'' {
                        self.pos += 1;
                        self.col += 2;
                        return Ok(());
                    }
                }
                '\n' => {
                    self.line += 1;
                    self.col = 1;
                }
                _ => {
                    self.col += 1;
                }
            }
        }
    }
}

enum LineAction {
    Skip,
    Emit(Vec<Spanned<Token>>),
    Done,
}

fn collect_tokens(input: &str) -> Result<Vec<Token>, DonowError> {
    let mut lexer = Lexer::new(input);
    lexer.tokenize().map(|v| v.into_iter().map(|s| s.token).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tok(input: &str) -> Vec<Token> {
        collect_tokens(input).unwrap()
    }

    #[test]
    fn empty_input() {
        assert_eq!(tok(""), vec![Token::Eof]);
    }

    #[test]
    fn only_comments() {
        assert_eq!(tok("# comment\n\n  # another\n"), vec![Token::Eof]);
    }

    #[test]
    fn only_multiline_comment() {
        assert_eq!(tok("'' comment ''"), vec![Token::Eof]);
    }

    #[test]
    fn block_header() {
        let t = tok("build:\n    cargo");
        assert_eq!(t[0], Token::Ident("build".into()));
        assert_eq!(t[1], Token::Colon);
        assert_eq!(t[2], Token::Newline);
        assert_eq!(t[3], Token::Indent);
        assert_eq!(t[4], Token::Ident("cargo".into()));
    }

    #[test]
    fn assignment() {
        let t = tok("    x = 5");
        assert!(t.contains(&Token::Assign));
        assert!(t.contains(&Token::Number(5)));
    }

    #[test]
    fn colon_assign() {
        let t = tok("    x + 5 : $var");
        assert!(t.contains(&Token::Plus));
        assert!(t.contains(&Token::Number(5)));
        assert!(t.contains(&Token::Colon));
        assert!(t.contains(&Token::Dollar));
        assert!(t.contains(&Token::Ident("var".into())));
    }

    #[test]
    fn if_statement() {
        let t = tok("    ? if $x == 5:");
        assert!(t.contains(&Token::If));
        assert!(t.contains(&Token::Dollar));
        assert!(t.contains(&Token::Ident("x".into())));
        assert!(t.contains(&Token::Eq));
        assert!(t.contains(&Token::Number(5)));
        assert!(t.contains(&Token::Colon));
    }

    #[test]
    fn control_flow() {
        let t = tok("    e?:\n    w! $i < 10:\n    f! $i in a[1,2,3]:");
        assert!(t.contains(&Token::Else));
        assert!(t.contains(&Token::While));
        assert!(t.contains(&Token::For));
        assert!(t.contains(&Token::In));
        assert!(t.contains(&Token::Lt));
    }

    #[test]
    fn string_literal() {
        let t = tok(r#"    x = "hello world""#);
        assert!(t.contains(&Token::Str("hello world".into())));
    }

    #[test]
    fn comparison_operators() {
        let t = tok("    == < > <= >= >! <!");
        assert!(t.contains(&Token::Eq));
        assert!(t.contains(&Token::Lt));
        assert!(t.contains(&Token::Gt));
        assert!(t.contains(&Token::Lte));
        assert!(t.contains(&Token::Gte));
        assert!(t.contains(&Token::NeqGt));
        assert!(t.contains(&Token::NeqLt));
    }

    #[test]
    fn math_operators() {
        let t = tok("    + - * /");
        assert!(t.contains(&Token::Plus));
        assert!(t.contains(&Token::Minus));
        assert!(t.contains(&Token::Star));
        assert!(t.contains(&Token::Slash));
    }

    #[test]
    fn variable_refs() {
        let t = tok("    $a %b @c");
        assert!(t.contains(&Token::Dollar));
        assert!(t.contains(&Token::Percent));
        assert!(t.contains(&Token::At));
        assert!(t.contains(&Token::Ident("a".into())));
        assert!(t.contains(&Token::Ident("b".into())));
        assert!(t.contains(&Token::Ident("c".into())));
    }

    #[test]
    fn array_literal() {
        let t = tok("    a[1, 2, 3]");
        assert!(t.contains(&Token::Ident("a".into())));
        assert!(t.contains(&Token::LBracket));
        assert!(t.contains(&Token::Number(1)));
        assert!(t.contains(&Token::Comma));
        assert!(t.contains(&Token::Number(2)));
        assert!(t.contains(&Token::Number(3)));
        assert!(t.contains(&Token::RBracket));
    }

    #[test]
    fn dict_access() {
        let t = tok("    $dict.clave");
        assert!(t.contains(&Token::Dollar));
        assert!(t.contains(&Token::Ident("dict".into())));
        assert!(t.contains(&Token::Dot));
        assert!(t.contains(&Token::Ident("clave".into())));
    }

    #[test]
    fn braces_toggle_indent() {
        let t = tok("task:\n    {\n    x = 1\n    }");
        assert!(t.contains(&Token::LBrace));
        assert!(t.contains(&Token::RBrace));
        // Inside braces, no Indent/Dedent for inner content
        let indent_count = t.iter().filter(|t| **t == Token::Indent).count();
        assert_eq!(indent_count, 1); // only for the block body
    }

    #[test]
    fn comment_line_skipped() {
        let t = tok("a:\n    # comment\n    x");
        // No comment tokens, just structure
        assert!(t.contains(&Token::Ident("a".into())));
        assert!(t.contains(&Token::Ident("x".into())));
    }

    #[test]
    fn multiline_comment_skipped() {
        let t = tok("a:\n    '' comment ''\n    x");
        assert!(t.contains(&Token::Ident("a".into())));
        assert!(t.contains(&Token::Ident("x".into())));
    }

    #[test]
    fn logical_operators() {
        let t = tok("    or and");
        assert!(t.contains(&Token::Or));
        assert!(t.contains(&Token::And));
    }

    #[test]
    fn ampersand_and_pipe() {
        let t = tok("    & |");
        assert!(t.contains(&Token::Ampersand));
        assert!(t.contains(&Token::Pipe));
    }

    #[test]
    fn inconsistent_indentation_error() {
        let result = collect_tokens("a:\n    x\n  y");
        assert!(result.is_err());
    }

    #[test]
    fn unterminated_string_error() {
        let result = collect_tokens("    x = \"hello");
        assert!(result.is_err());
    }

    #[test]
    fn unterminated_multiline_comment_error() {
        let result = collect_tokens("    '' no close");
        assert!(result.is_err());
    }

    #[test]
    fn standalone_question_mark() {
        let t = tok("    ? maybe");
        assert!(t.contains(&Token::Question));
        assert!(t.contains(&Token::Ident("maybe".into())));
    }

    #[test]
    fn dedent_between_blocks() {
        let t = tok("a:\n    x\nb:\n    y\n");
        // a: body has Indent... Dedent... then b:
        let dedents: Vec<&Token> = t.iter().filter(|t| **t == Token::Dedent).collect();
        assert_eq!(dedents.len(), 2); // end of a body, end of b body
    }

    #[test]
    fn full_example() {
        let input = r#"build:
    cargo build

pull:
    {
        $DATE = "date +%F"
        FOLDER = "project/"
        (git add %FOLDER && git commit -m %DATE)
        git push origin main
    }
"#;
        let result = collect_tokens(input);
        assert!(result.is_ok());
        let t = result.unwrap();
        assert!(t.contains(&Token::Ident("build".into())));
        assert!(t.contains(&Token::Ident("pull".into())));
        assert!(t.contains(&Token::LBrace));
        assert!(t.contains(&Token::RBrace));
        assert!(t.contains(&Token::Dollar));
        assert!(t.contains(&Token::Percent));
        assert!(t.contains(&Token::LParen));
        assert!(t.contains(&Token::RParen));
        assert!(t.contains(&Token::Ampersand));
        assert!(t.contains(&Token::Minus));
    }
}

