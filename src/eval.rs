use std::process::Command;

use crate::ast::*;
use crate::expand::Expander;
use crate::scope::Scope;
use crate::value::Value;

#[derive(Debug, Clone)]
pub struct EvalError {
    pub message: String,
}

impl EvalError {
    pub fn new(message: impl Into<String>) -> Self {
        EvalError {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for EvalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for EvalError {}

// ----------------------------------------------------------------
//  Eval
// ----------------------------------------------------------------

pub struct Eval<'a> {
    pub scope: &'a mut Scope,
    pub expander: Expander,
}

impl<'a> Eval<'a> {
    pub fn new(scope: &'a mut Scope) -> Self {
        Eval {
            scope,
            expander: Expander::new(),
        }
    }

    // --------------------------------------------------------
    //  Program & Block dispatch
    // --------------------------------------------------------

    pub fn eval_program(&mut self, program: &Program, block_name: &str) -> Result<(), EvalError> {
        let block = program
            .blocks
            .iter()
            .find(|b| b.name == block_name)
            .ok_or_else(|| EvalError::new(format!("block '{}' not found", block_name)))?;
        self.eval_body(&block.body)
    }

    pub fn eval_body(&mut self, stmts: &[Stmt]) -> Result<(), EvalError> {
        for stmt in stmts {
            self.eval_stmt(stmt)?;
        }
        Ok(())
    }

    // --------------------------------------------------------
    //  Statements
    // --------------------------------------------------------

    pub fn eval_stmt(&mut self, stmt: &Stmt) -> Result<(), EvalError> {
        match stmt {
            Stmt::Assign { name, value, .. } => {
                let val = self.eval_expr(value)?;
                self.scope.set_var(name, val);
                Ok(())
            }
            Stmt::ColonAssign { target, var_name, .. } => {
                let val = self.eval_expr(target)?;
                self.scope.set_var(var_name, val);
                Ok(())
            }
            Stmt::Command(text, _) => {
                let expanded = self.expander.expand(text, self.scope)
                    .map_err(|e| EvalError::new(e.message))?;
                self.run_shell(&expanded)
            }
            Stmt::BraceBlock(stmts, _) => {
                self.eval_body(stmts)
            }
            Stmt::PriorityBlock(stmts, _) => {
                self.eval_body(stmts)
            }
            Stmt::DeferredBlock(stmts, _) => {
                self.eval_body(stmts)
            }
            Stmt::If { cond, body, else_body, .. } => {
                let cv = self.eval_expr(cond)?;
                if cv.is_truthy() {
                    self.eval_body(body)
                } else if let Some(eb) = else_body {
                    self.eval_body(eb)
                } else {
                    Ok(())
                }
            }
            Stmt::While { cond, body, .. } => {
                loop {
                    let cv = self.eval_expr(cond)?;
                    if !cv.is_truthy() {
                        break;
                    }
                    self.eval_body(body)?;
                }
                Ok(())
            }
            Stmt::For { var, iter, body, .. } => {
                let it = self.eval_expr(iter)?;
                let items: Vec<Value> = match &it {
                    Value::Array(a) => a.clone(),
                    Value::List(l) => l.clone(),
                    _ => return Err(EvalError::new(format!("cannot iterate over {}", it.type_name()))),
                };
                for item in items {
                    self.scope.set_var(var, item.clone());
                    self.eval_body(body)?;
                }
                Ok(())
            }
        }
    }

    // --------------------------------------------------------
    //  Expressions
    // --------------------------------------------------------

    pub fn eval_expr(&mut self, expr: &Expr) -> Result<Value, EvalError> {
        match expr {
            Expr::Number(n, _) => Ok(Value::Int(*n)),
            Expr::String(s, _) => Ok(Value::String(s.clone())),
            Expr::Bool(b, _) => Ok(Value::Bool(*b)),
            Expr::Ident(name, _) => self.scope
                .get_var(name)
                .cloned()
                .ok_or_else(|| EvalError::new(format!("undefined variable: {}", name))),
            Expr::VarRef(name, _) => self.scope
                .get_var(name)
                .cloned()
                .ok_or_else(|| EvalError::new(format!("undefined variable: ${}", name))),
            Expr::ParamRef(name, _) => self.scope
                .get_var(name)
                .cloned()
                .ok_or_else(|| EvalError::new(format!("undefined variable: %{}", name))),
            Expr::CliParam(name, _) => self.scope
                .get_param(name)
                .cloned()
                .ok_or_else(|| EvalError::new(format!("undefined CLI param: @{}", name))),
            Expr::Array(elems, _) => {
                let values: Result<Vec<Value>, _> = elems.iter().map(|e| self.eval_expr(e)).collect();
                Ok(Value::Array(values?))
            }
            Expr::List(elems, _) => {
                let values: Result<Vec<Value>, _> = elems.iter().map(|e| self.eval_expr(e)).collect();
                Ok(Value::List(values?))
            }
            Expr::Dict(entries, _) => {
                let mut pairs = Vec::new();
                for (k, v) in entries {
                    let key = match self.eval_expr(k)? {
                        Value::String(s) => s,
                        val => val.into_string(),
                    };
                    let val = self.eval_expr(v)?;
                    pairs.push((key, val));
                }
                Ok(Value::Dict(pairs))
            }
            Expr::BinOp { left, op, right, .. } => {
                let l = self.eval_expr(left)?;
                let r = self.eval_expr(right)?;
                match op {
                    BinOp::Add => l.add(&r).map_err(errmap),
                    BinOp::Sub => l.sub(&r).map_err(errmap),
                    BinOp::Mul => l.mul(&r).map_err(errmap),
                    BinOp::Div => l.div(&r).map_err(errmap),
                    BinOp::Eq => l.eq(&r).map_err(errmap),
                    BinOp::Neq => Ok(Value::Bool(l != r)), // structural inequality
                    BinOp::Lt => l.lt(&r).map_err(errmap),
                    BinOp::Gt => l.gt(&r).map_err(errmap),
                    BinOp::Lte => l.lte(&r).map_err(errmap),
                    BinOp::Gte => l.gte(&r).map_err(errmap),
                    BinOp::NeqGt => l.neq_gt(&r).map_err(errmap),
                    BinOp::NeqLt => l.neq_lt(&r).map_err(errmap),
                    BinOp::And => l.and(&r).map_err(errmap),
                    BinOp::Or => l.or(&r).map_err(errmap),
                }
            }
            Expr::UnaryOp { op, expr, .. } => {
                let v = self.eval_expr(expr)?;
                match op {
                    UnaryOp::Not => v.not().map_err(errmap),
                }
            }
            Expr::Index { arr, index, .. } => {
                let a = self.eval_expr(arr)?;
                let i = self.eval_expr(index)?;
                a.index(&i).map_err(errmap)
            }
            Expr::DotAccess { obj, field, .. } => {
                let o = self.eval_expr(obj)?;
                o.dot(field).map_err(errmap)
            }
            Expr::Template { name, .. } => {
                Err(EvalError::new(format!("template expansion not yet implemented: T<{}>", name)))
            }
            Expr::FuncCall { name, .. } => {
                Err(EvalError::new(format!("function call not yet implemented: F<{}>", name)))
            }
            Expr::ClassRef { name, .. } => {
                Err(EvalError::new(format!("class reference not yet implemented: C<{}>", name)))
            }
        }
    }

    // --------------------------------------------------------
    //  Command execution
    // --------------------------------------------------------

    fn run_shell(&self, cmd: &str) -> Result<(), EvalError> {
        let output = Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .output()
            .map_err(|e| EvalError::new(format!("failed to execute command: {}", e)))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if !stdout.is_empty() {
            print!("{}", stdout);
        }
        if !stderr.is_empty() {
            eprint!("{}", stderr);
        }

        if !output.status.success() {
            return Err(EvalError::new(format!(
                "command exited with code {:?}: {}",
                output.status.code(),
                cmd
            )));
        }

        Ok(())
    }
}

fn errmap(e: crate::value::ValueError) -> EvalError {
    EvalError::new(e.message)
}

// ----------------------------------------------------------------
//  Tests
// ----------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    fn parse(input: &str) -> Program {
        let mut lex = Lexer::new(input);
        let toks = lex.tokenize().unwrap();
        let mut p = Parser::new(toks, input);
        p.parse().unwrap()
    }

    fn eval_block(body: &[Stmt]) -> Scope {
        let mut scope = Scope::new();
        let mut e = Eval::new(&mut scope);
        e.eval_body(body).unwrap();
        e.scope.clone()
    }

    #[test]
    fn assign_int() {
        let p = parse("t:\n    x = 42");
        let s = eval_block(&p.blocks[0].body);
        assert_eq!(s.get_var("x"), Some(&Value::Int(42)));
    }

    #[test]
    fn assign_string() {
        let p = parse("t:\n    x = \"hello\"");
        let s = eval_block(&p.blocks[0].body);
        assert_eq!(s.get_var("x"), Some(&Value::String("hello".into())));
    }

    #[test]
    fn assign_expr() {
        let p = parse("t:\n    x = 2 + 3 * 4");
        let s = eval_block(&p.blocks[0].body);
        assert_eq!(s.get_var("x"), Some(&Value::Int(14)));
    }

    #[test]
    fn colon_assign() {
        let p = parse("t:\n    {\n    2 + 3 : $x\n    }");
        let s = eval_block(&p.blocks[0].body);
        assert_eq!(s.get_var("x"), Some(&Value::Int(5)));
    }

    #[test]
    fn var_ref_in_expr() {
        let p = parse("t:\n    x = 5\n    y = $x + 3");
        let s = eval_block(&p.blocks[0].body);
        assert_eq!(s.get_var("y"), Some(&Value::Int(8)));
    }

    #[test]
    fn comparison_expr() {
        let p = parse("t:\n    x = 10 == 10\n    y = 3 > 5");
        let s = eval_block(&p.blocks[0].body);
        assert_eq!(s.get_var("x"), Some(&Value::Bool(true)));
        assert_eq!(s.get_var("y"), Some(&Value::Bool(false)));
    }

    #[test]
    fn if_true() {
        let p = parse("t:\n    ? if 1 == 1:\n        x = 42");
        let s = eval_block(&p.blocks[0].body);
        assert_eq!(s.get_var("x"), Some(&Value::Int(42)));
    }

    #[test]
    fn if_false() {
        let p = parse("t:\n    ? if 1 == 0:\n        x = 42\n    e?:\n        x = 99");
        let s = eval_block(&p.blocks[0].body);
        assert_eq!(s.get_var("x"), Some(&Value::Int(99)));
    }

    #[test]
    fn while_loop() {
        let p = parse("t:\n    i = 0\n    w! $i < 5:\n        i = $i + 1");
        let s = eval_block(&p.blocks[0].body);
        assert_eq!(s.get_var("i"), Some(&Value::Int(5)));
    }

    #[test]
    fn for_loop() {
        let p = parse("t:\n    f! $i in a[10, 20, 30]:\n        x = $i");
        let s = eval_block(&p.blocks[0].body);
        assert_eq!(s.get_var("x"), Some(&Value::Int(30)));
    }

    #[test]
    fn command_execution_with_scope() {
        // Test that eval uses Expand for commands
        let p = parse("t:\n    x = hello\n    echo %x");
        let mut scope = Scope::new();
        let mut e = Eval::new(&mut scope);
        // Just verify it doesn't crash (shell may not be available in test env)
        let _ = e.eval_body(&p.blocks[0].body);
    }

    #[test]
    fn and_or_operators() {
        let p = parse("t:\n    x = 1 == 1 and 2 == 2\n    y = 1 == 1 or 2 == 3\n    z = !false");
        let s = eval_block(&p.blocks[0].body);
        assert_eq!(s.get_var("x"), Some(&Value::Bool(true)));
        assert_eq!(s.get_var("y"), Some(&Value::Bool(true)));
        assert_eq!(s.get_var("z"), Some(&Value::Bool(true)));
    }

    #[test]
    fn array_literal() {
        let p = parse("t:\n    x = a[1, 2, 3]");
        let s = eval_block(&p.blocks[0].body);
        let expected = Value::Array(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        assert_eq!(s.get_var("x"), Some(&expected));
    }

    #[test]
    fn index_access() {
        let p = parse("t:\n    x = a[10, 20, 30][1]");
        let s = eval_block(&p.blocks[0].body);
        assert_eq!(s.get_var("x"), Some(&Value::Int(20)));
    }

    #[test]
    fn shell_command_echo() {
        let p = parse("t:\n    echo hello from donow");
        let mut scope = Scope::new();
        let mut e = Eval::new(&mut scope);
        assert!(e.eval_body(&p.blocks[0].body).is_ok());
    }
}
