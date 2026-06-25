use std::io::{self, BufRead, Write};
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::value::Value;

#[derive(Debug, Clone)]
pub struct BuiltinError {
    pub message: String,
}

impl BuiltinError {
    pub fn new(message: impl Into<String>) -> Self {
        BuiltinError {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for BuiltinError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for BuiltinError {}

/// Check if a name is a registered builtin function.
pub fn is_builtin(name: &str) -> bool {
    matches!(name, "echo" | "date" | "sum" | "avg" | "len" | "read" | "exit")
}

/// Call a builtin function by name with the given arguments.
pub fn call_builtin(name: &str, args: &[Value]) -> Result<Value, BuiltinError> {
    match name {
        "echo" => builtin_echo(args),
        "date" => builtin_date(args),
        "sum" => builtin_sum(args),
        "avg" => builtin_avg(args),
        "len" => builtin_len(args),
        "read" => builtin_read(args),
        "exit" => builtin_exit(args),
        _ => Err(BuiltinError::new(format!("unknown builtin: {}", name))),
    }
}

// ----------------------------------------------------------------
//  Builtin implementations
// ----------------------------------------------------------------

fn builtin_echo(args: &[Value]) -> Result<Value, BuiltinError> {
    let line: String = args
        .iter()
        .map(|v| v.to_string())
        .collect::<Vec<_>>()
        .join(" ");
    println!("{}", line);
    Ok(Value::Null)
}

fn builtin_date(args: &[Value]) -> Result<Value, BuiltinError> {
    let fmt = args.first().map(|v| v.to_string()).unwrap_or_default();

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| BuiltinError::new(format!("time error: {}", e)))?;

    let secs = now.as_secs();

    let result = if fmt.is_empty() {
        // Default: ISO-8601 like date +%F
        let days = secs / 86400;
        let y = 1970 + (days as f64 / 365.25) as u64;
        // rough calculation; real impl would use chrono
        format!("{}", y)
    } else {
        // Simple format substitutions
        let mut out = fmt;
        out = out.replace("%Y", &format!("{}", 1970 + (secs / 31536000)));
        out = out.replace("%s", &secs.to_string());
        out
    };

    Ok(Value::String(result))
}

fn builtin_sum(args: &[Value]) -> Result<Value, BuiltinError> {
    let arr = expect_array(args, "sum")?;
    let total: i64 = arr
        .iter()
        .map(|v| v.as_int().unwrap_or(0))
        .sum();
    Ok(Value::Int(total))
}

fn builtin_avg(args: &[Value]) -> Result<Value, BuiltinError> {
    let arr = expect_array(args, "avg")?;
    if arr.is_empty() {
        return Ok(Value::Int(0));
    }
    let total: i64 = arr
        .iter()
        .map(|v| v.as_int().unwrap_or(0))
        .sum();
    Ok(Value::Int(total / arr.len() as i64))
}

fn builtin_len(args: &[Value]) -> Result<Value, BuiltinError> {
    let val = args
        .first()
        .ok_or_else(|| BuiltinError::new("len requires 1 argument"))?;
    let n = match val {
        Value::String(s) => s.len() as i64,
        Value::Array(a) => a.len() as i64,
        Value::List(l) => l.len() as i64,
        Value::Dict(d) => d.len() as i64,
        _ => {
            return Err(BuiltinError::new(format!(
                "len not supported for {}",
                val.type_name()
            )))
        }
    };
    Ok(Value::Int(n))
}

fn builtin_read(args: &[Value]) -> Result<Value, BuiltinError> {
    let prompt = args
        .first()
        .map(|v| v.to_string())
        .unwrap_or_default();

    if !prompt.is_empty() {
        print!("{}", prompt);
        io::stdout().flush().map_err(|e| BuiltinError::new(e.to_string()))?;
    }

    let mut line = String::new();
    io::stdin()
        .lock()
        .read_line(&mut line)
        .map_err(|e| BuiltinError::new(format!("read error: {}", e)))?;

    Ok(Value::String(line.trim_end().to_string()))
}

fn builtin_exit(args: &[Value]) -> Result<Value, BuiltinError> {
    let code = args
        .first()
        .and_then(|v| v.as_int())
        .unwrap_or(0) as i32;
    process::exit(code);
}

// ----------------------------------------------------------------
//  Helper
// ----------------------------------------------------------------

fn expect_array<'a>(args: &'a [Value], name: &str) -> Result<&'a [Value], BuiltinError> {
    let val = args
        .first()
        .ok_or_else(|| BuiltinError::new(format!("{} requires 1 argument (array)", name)))?;
    match val {
        Value::Array(arr) => Ok(arr.as_slice()),
        _ => Err(BuiltinError::new(format!(
            "{} expected array, got {}",
            name,
            val.type_name()
        ))),
    }
}

// ----------------------------------------------------------------
//  Tests
// ----------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_builtin_recognizes_all() {
        assert!(is_builtin("echo"));
        assert!(is_builtin("date"));
        assert!(is_builtin("sum"));
        assert!(is_builtin("avg"));
        assert!(is_builtin("len"));
        assert!(is_builtin("read"));
        assert!(is_builtin("exit"));
        assert!(!is_builtin("nonexistent"));
    }

    #[test]
    fn echo_returns_null() {
        let r = call_builtin("echo", &[Value::String("hi".into())]).unwrap();
        assert_eq!(r, Value::Null);
    }

    #[test]
    fn sum_of_array() {
        let arr = Value::Array(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        let r = call_builtin("sum", &[arr]).unwrap();
        assert_eq!(r, Value::Int(6));
    }

    #[test]
    fn sum_of_empty() {
        let arr = Value::Array(vec![]);
        let r = call_builtin("sum", &[arr]).unwrap();
        assert_eq!(r, Value::Int(0));
    }

    #[test]
    fn avg_of_array() {
        let arr = Value::Array(vec![Value::Int(2), Value::Int(4), Value::Int(6)]);
        let r = call_builtin("avg", &[arr]).unwrap();
        assert_eq!(r, Value::Int(4));
    }

    #[test]
    fn avg_of_empty() {
        let arr = Value::Array(vec![]);
        let r = call_builtin("avg", &[arr]).unwrap();
        assert_eq!(r, Value::Int(0));
    }

    #[test]
    fn len_string() {
        let r = call_builtin("len", &[Value::String("hello".into())]).unwrap();
        assert_eq!(r, Value::Int(5));
    }

    #[test]
    fn len_array() {
        let r = call_builtin("len", &[Value::Array(vec![Value::Int(1), Value::Int(2)])]).unwrap();
        assert_eq!(r, Value::Int(2));
    }

    #[test]
    fn len_list() {
        let r = call_builtin("len", &[Value::List(vec![Value::Int(1)])]).unwrap();
        assert_eq!(r, Value::Int(1));
    }

    #[test]
    fn len_dict() {
        let d = Value::Dict(vec![("a".into(), Value::Int(1))]);
        let r = call_builtin("len", &[d]).unwrap();
        assert_eq!(r, Value::Int(1));
    }

    #[test]
    fn len_bad_type() {
        let r = call_builtin("len", &[Value::Int(42)]);
        assert!(r.is_err());
    }

    #[test]
    fn sum_wrong_type() {
        let r = call_builtin("sum", &[Value::String("x".into())]);
        assert!(r.is_err());
    }

    #[test]
    fn missing_args() {
        assert!(call_builtin("sum", &[]).is_err());
        assert!(call_builtin("len", &[]).is_err());
    }

    #[test]
    fn unknown_builtin() {
        let r = call_builtin("foobar", &[]);
        assert!(r.is_err());
    }

    #[test]
    fn date_returns_string() {
        let r = call_builtin("date", &[]).unwrap();
        assert!(matches!(r, Value::String(_)));
    }
}
