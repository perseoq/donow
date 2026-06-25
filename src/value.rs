use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    String(String),
    Int(i64),
    Bool(bool),
    Array(Vec<Value>),
    List(Vec<Value>),
    Dict(Vec<(String, Value)>),
    Null,
}

impl Value {
    pub fn as_int(&self) -> Option<i64> {
        match self {
            Value::Int(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s.as_str()),
            _ => None,
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Int(n) => *n != 0,
            Value::String(s) => !s.is_empty(),
            Value::Array(a) => !a.is_empty(),
            Value::List(l) => !l.is_empty(),
            Value::Dict(d) => !d.is_empty(),
            Value::Null => false,
        }
    }

    pub fn add(&self, other: &Value) -> Result<Value, ValueError> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
            (Value::String(a), Value::String(b)) => {
                let mut s = a.clone();
                s.push_str(b);
                Ok(Value::String(s))
            }
            (Value::Array(a), Value::Array(b)) => {
                let mut v = a.clone();
                v.extend(b.clone());
                Ok(Value::Array(v))
            }
            (Value::List(a), Value::List(b)) => {
                let mut v = a.clone();
                v.extend(b.clone());
                Ok(Value::List(v))
            }
            _ => Err(ValueError::new(format!(
                "cannot add {} and {}",
                self.type_name(),
                other.type_name()
            ))),
        }
    }

    pub fn sub(&self, other: &Value) -> Result<Value, ValueError> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a - b)),
            _ => Err(ValueError::new(format!(
                "cannot subtract {} and {}",
                self.type_name(),
                other.type_name()
            ))),
        }
    }

    pub fn mul(&self, other: &Value) -> Result<Value, ValueError> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a * b)),
            (Value::String(s), Value::Int(n)) => Ok(Value::String(s.repeat(*n as usize))),
            (Value::Int(n), Value::String(s)) => Ok(Value::String(s.repeat(*n as usize))),
            (Value::Array(a), Value::Int(n)) => {
                let mut v = Vec::new();
                for _ in 0..*n {
                    v.extend(a.clone());
                }
                Ok(Value::Array(v))
            }
            _ => Err(ValueError::new(format!(
                "cannot multiply {} and {}",
                self.type_name(),
                other.type_name()
            ))),
        }
    }

    pub fn div(&self, other: &Value) -> Result<Value, ValueError> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => {
                if *b == 0 {
                    Err(ValueError::new("division by zero".into()))
                } else {
                    Ok(Value::Int(a / b))
                }
            }
            _ => Err(ValueError::new(format!(
                "cannot divide {} and {}",
                self.type_name(),
                other.type_name()
            ))),
        }
    }

    pub fn eq(&self, other: &Value) -> Result<Value, ValueError> {
        Ok(Value::Bool(self == other))
    }

    pub fn lt(&self, other: &Value) -> Result<Value, ValueError> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a < b)),
            (Value::String(a), Value::String(b)) => Ok(Value::Bool(a < b)),
            _ => Err(ValueError::new(format!(
                "cannot compare {} and {} with <",
                self.type_name(),
                other.type_name()
            ))),
        }
    }

    pub fn gt(&self, other: &Value) -> Result<Value, ValueError> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a > b)),
            (Value::String(a), Value::String(b)) => Ok(Value::Bool(a > b)),
            _ => Err(ValueError::new(format!(
                "cannot compare {} and {} with >",
                self.type_name(),
                other.type_name()
            ))),
        }
    }

    pub fn lte(&self, other: &Value) -> Result<Value, ValueError> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a <= b)),
            (Value::String(a), Value::String(b)) => Ok(Value::Bool(a <= b)),
            _ => Err(ValueError::new(format!(
                "cannot compare {} and {} with <=",
                self.type_name(),
                other.type_name()
            ))),
        }
    }

    pub fn gte(&self, other: &Value) -> Result<Value, ValueError> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a >= b)),
            (Value::String(a), Value::String(b)) => Ok(Value::Bool(a >= b)),
            _ => Err(ValueError::new(format!(
                "cannot compare {} and {} with >=",
                self.type_name(),
                other.type_name()
            ))),
        }
    }

    pub fn neq_gt(&self, other: &Value) -> Result<Value, ValueError> {
        let eq = self.eq(other)?;
        let gt = self.gt(other)?;
        Ok(Value::Bool(eq.as_bool().unwrap() || gt.as_bool().unwrap()))
    }

    pub fn neq_lt(&self, other: &Value) -> Result<Value, ValueError> {
        let eq = self.eq(other)?;
        let lt = self.lt(other)?;
        Ok(Value::Bool(eq.as_bool().unwrap() || lt.as_bool().unwrap()))
    }

    pub fn and(&self, other: &Value) -> Result<Value, ValueError> {
        Ok(Value::Bool(self.is_truthy() && other.is_truthy()))
    }

    pub fn or(&self, other: &Value) -> Result<Value, ValueError> {
        Ok(Value::Bool(self.is_truthy() || other.is_truthy()))
    }

    pub fn not(&self) -> Result<Value, ValueError> {
        Ok(Value::Bool(!self.is_truthy()))
    }

    pub fn index(&self, index: &Value) -> Result<Value, ValueError> {
        match (self, index) {
            (Value::Array(arr), Value::Int(i)) => {
                let idx = if *i < 0 {
                    arr.len().checked_sub(i.unsigned_abs() as usize)
                } else {
                    Some(*i as usize)
                };
                match idx {
                    Some(i) if i < arr.len() => Ok(arr[i].clone()),
                    _ => Ok(Value::Null),
                }
            }
            (Value::List(lst), Value::Int(i)) => {
                let idx = if *i < 0 {
                    lst.len().checked_sub(i.unsigned_abs() as usize)
                } else {
                    Some(*i as usize)
                };
                match idx {
                    Some(i) if i < lst.len() => Ok(lst[i].clone()),
                    _ => Ok(Value::Null),
                }
            }
            _ => Err(ValueError::new(format!(
                "cannot index {} with {}",
                self.type_name(),
                index.type_name()
            ))),
        }
    }

    pub fn dot(&self, field: &str) -> Result<Value, ValueError> {
        match self {
            Value::Dict(entries) => {
                for (k, v) in entries {
                    if k == field {
                        return Ok(v.clone());
                    }
                }
                Ok(Value::Null)
            }
            _ => Err(ValueError::new(format!(
                "cannot access field '{}' on {}",
                field,
                self.type_name()
            ))),
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            Value::String(_) => "string",
            Value::Int(_) => "int",
            Value::Bool(_) => "bool",
            Value::Array(_) => "array",
            Value::List(_) => "list",
            Value::Dict(_) => "dict",
            Value::Null => "null",
        }
    }

    pub fn into_string(self) -> String {
        match self {
            Value::String(s) => s,
            Value::Int(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Null => "null".into(),
            Value::Array(arr) => {
                let items: Vec<String> = arr.into_iter().map(|v| v.into_string()).collect();
                format!("[{}]", items.join(", "))
            }
            Value::List(lst) => {
                let items: Vec<String> = lst.into_iter().map(|v| v.into_string()).collect();
                format!("[{}]", items.join(", "))
            }
            Value::Dict(entries) => {
                let items: Vec<String> = entries
                    .into_iter()
                    .map(|(k, v)| format!("{}: {}", k, v.into_string()))
                    .collect();
                format!("{{{}}}", items.join(", "))
            }
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::String(s) => write!(f, "{}", s),
            Value::Int(n) => write!(f, "{}", n),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Null => write!(f, "null"),
            Value::Array(arr) => {
                let items: Vec<String> = arr.iter().map(|v| v.to_string()).collect();
                write!(f, "[{}]", items.join(", "))
            }
            Value::List(lst) => {
                let items: Vec<String> = lst.iter().map(|v| v.to_string()).collect();
                write!(f, "[{}]", items.join(", "))
            }
            Value::Dict(entries) => {
                let items: Vec<String> =
                    entries.iter().map(|(k, v)| format!("{}: {}", k, v)).collect();
                write!(f, "{{{}}}", items.join(", "))
            }
        }
    }
}

impl From<i64> for Value {
    fn from(n: i64) -> Self {
        Value::Int(n)
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::String(s.to_string())
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::String(s)
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Value::Bool(b)
    }
}

impl From<Vec<Value>> for Value {
    fn from(v: Vec<Value>) -> Self {
        Value::Array(v)
    }
}

// ----------------------------------------------------------------
//  ValueError
// ----------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct ValueError {
    pub message: String,
}

impl ValueError {
    pub fn new(message: String) -> Self {
        ValueError { message }
    }
}

impl fmt::Display for ValueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ValueError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn int_arithmetic() {
        let a = Value::Int(10);
        let b = Value::Int(3);
        assert_eq!(a.add(&b).unwrap(), Value::Int(13));
        assert_eq!(a.sub(&b).unwrap(), Value::Int(7));
        assert_eq!(a.mul(&b).unwrap(), Value::Int(30));
        assert_eq!(a.div(&b).unwrap(), Value::Int(3));
    }

    #[test]
    fn string_concat() {
        let a = Value::String("hello ".into());
        let b = Value::String("world".into());
        assert_eq!(a.add(&b).unwrap(), Value::String("hello world".into()));
    }

    #[test]
    fn string_repeat() {
        let s = Value::String("ab".into());
        assert_eq!(s.mul(&Value::Int(3)).unwrap(), Value::String("ababab".into()));
    }

    #[test]
    fn type_errors() {
        let a = Value::Int(5);
        let b = Value::String("x".into());
        assert!(a.add(&b).is_err());
        assert!(a.sub(&b).is_err());
        assert!(a.mul(&b).is_ok()); // Int * String is repeat
        assert!(a.div(&b).is_err());
    }

    #[test]
    fn division_by_zero() {
        assert!(Value::Int(5).div(&Value::Int(0)).is_err());
    }

    #[test]
    fn comparisons() {
        assert_eq!(Value::Int(3).lt(&Value::Int(5)).unwrap(), Value::Bool(true));
        assert_eq!(Value::Int(5).lt(&Value::Int(3)).unwrap(), Value::Bool(false));
        assert_eq!(Value::Int(3).gt(&Value::Int(5)).unwrap(), Value::Bool(false));
        assert_eq!(Value::Int(5).gt(&Value::Int(3)).unwrap(), Value::Bool(true));
        assert_eq!(Value::Int(5).eq(&Value::Int(5)).unwrap(), Value::Bool(true));
        assert_eq!(Value::Int(5).eq(&Value::Int(3)).unwrap(), Value::Bool(false));
        assert_eq!(Value::Int(3).lte(&Value::Int(5)).unwrap(), Value::Bool(true));
        assert_eq!(Value::Int(5).lte(&Value::Int(3)).unwrap(), Value::Bool(false));
        assert_eq!(Value::Int(5).gte(&Value::Int(3)).unwrap(), Value::Bool(true));
    }

    #[test]
    fn string_comparisons() {
        assert_eq!(
            Value::String("a".into()).lt(&Value::String("b".into())).unwrap(),
            Value::Bool(true)
        );
    }

    #[test]
    fn neq_operators() {
        let a = Value::Int(5);
        let b = Value::Int(3);
        assert_eq!(a.neq_gt(&b).unwrap(), Value::Bool(true)); // 5 >= 3 → true
        assert_eq!(b.neq_gt(&a).unwrap(), Value::Bool(false)); // 3 >= 5 → false
        assert_eq!(a.neq_gt(&Value::Int(5)).unwrap(), Value::Bool(true)); // 5 >= 5 → true
        assert_eq!(b.neq_lt(&a).unwrap(), Value::Bool(true)); // 3 <= 5 → true
    }

    #[test]
    fn logical_ops() {
        assert_eq!(Value::Bool(true).and(&Value::Bool(true)).unwrap(), Value::Bool(true));
        assert_eq!(Value::Bool(true).and(&Value::Bool(false)).unwrap(), Value::Bool(false));
        assert_eq!(Value::Bool(true).or(&Value::Bool(false)).unwrap(), Value::Bool(true));
        assert_eq!(Value::Bool(false).or(&Value::Bool(false)).unwrap(), Value::Bool(false));
        assert_eq!(Value::Bool(true).not().unwrap(), Value::Bool(false));
        assert_eq!(Value::Bool(false).not().unwrap(), Value::Bool(true));
    }

    #[test]
    fn truthiness() {
        assert!(!Value::Null.is_truthy());
        assert!(!Value::Bool(false).is_truthy());
        assert!(!Value::Int(0).is_truthy());
        assert!(!Value::String("".into()).is_truthy());
        assert!(!Value::Array(vec![]).is_truthy());
        assert!(Value::Bool(true).is_truthy());
        assert!(Value::Int(1).is_truthy());
        assert!(Value::String("x".into()).is_truthy());
        assert!(Value::Array(vec![Value::Int(1)]).is_truthy());
    }

    #[test]
    fn array_index() {
        let arr = Value::Array(vec![Value::Int(10), Value::Int(20), Value::Int(30)]);
        assert_eq!(arr.index(&Value::Int(0)).unwrap(), Value::Int(10));
        assert_eq!(arr.index(&Value::Int(1)).unwrap(), Value::Int(20));
        assert_eq!(arr.index(&Value::Int(2)).unwrap(), Value::Int(30));
        assert_eq!(arr.index(&Value::Int(3)).unwrap(), Value::Null);
        assert_eq!(arr.index(&Value::Int(-1)).unwrap(), Value::Int(30));
    }

    #[test]
    fn dict_dot_access() {
        let dict = Value::Dict(vec![
            ("name".into(), Value::String("test".into())),
            ("count".into(), Value::Int(42)),
        ]);
        assert_eq!(dict.dot("name").unwrap(), Value::String("test".into()));
        assert_eq!(dict.dot("count").unwrap(), Value::Int(42));
        assert_eq!(dict.dot("missing").unwrap(), Value::Null);
    }

    #[test]
    fn array_add() {
        let a = Value::Array(vec![Value::Int(1), Value::Int(2)]);
        let b = Value::Array(vec![Value::Int(3)]);
        let r = a.add(&b).unwrap();
        assert_eq!(r, Value::Array(vec![Value::Int(1), Value::Int(2), Value::Int(3)]));
    }

    #[test]
    fn into_string_conversion() {
        assert_eq!(Value::Int(42).into_string(), "42");
        assert_eq!(Value::String("hi".into()).into_string(), "hi");
        assert_eq!(Value::Bool(true).into_string(), "true");
        assert_eq!(Value::Null.into_string(), "null");
    }

    #[test]
    fn eq_with_different_types() {
        assert_eq!(Value::Int(5).eq(&Value::String("5".into())).unwrap(), Value::Bool(false));
    }
}

