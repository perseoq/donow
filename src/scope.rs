use std::collections::HashMap;

use crate::value::Value;

#[derive(Debug, Clone)]
pub struct Scope {
    variables: HashMap<String, Value>,
    pub params: HashMap<String, Value>,
    parent: Option<Box<Scope>>,
}

impl Scope {
    pub fn new() -> Self {
        Scope {
            variables: HashMap::new(),
            params: HashMap::new(),
            parent: None,
        }
    }

    pub fn new_child(parent: &Scope) -> Self {
        Scope {
            variables: HashMap::new(),
            params: HashMap::new(),
            parent: Some(Box::new(parent.clone())),
        }
    }

    pub fn set_var(&mut self, name: &str, value: Value) {
        self.variables.insert(name.to_string(), value);
    }

    pub fn get_var(&self, name: &str) -> Option<&Value> {
        match self.variables.get(name) {
            Some(v) => Some(v),
            None => match &self.parent {
                Some(parent) => parent.get_var(name),
                None => None,
            },
        }
    }

    pub fn set_param(&mut self, name: &str, value: Value) {
        self.params.insert(name.to_string(), value);
    }

    pub fn get_param(&self, name: &str) -> Option<&Value> {
        self.params.get(name)
    }

    pub fn has_var(&self, name: &str) -> bool {
        match self.variables.contains_key(name) {
            true => true,
            false => match &self.parent {
                Some(parent) => parent.has_var(name),
                None => false,
            },
        }
    }

    pub fn iter_vars(&self) -> impl Iterator<Item = (&String, &Value)> {
        self.variables.iter()
    }
}

impl Default for Scope {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_and_get_var() {
        let mut scope = Scope::new();
        scope.set_var("x", Value::Int(42));
        assert_eq!(scope.get_var("x"), Some(&Value::Int(42)));
    }

    #[test]
    fn get_missing_var() {
        let scope = Scope::new();
        assert_eq!(scope.get_var("nonexistent"), None);
    }

    #[test]
    fn child_scope_inherits() {
        let mut parent = Scope::new();
        parent.set_var("x", Value::Int(10));
        let child = Scope::new_child(&parent);
        assert_eq!(child.get_var("x"), Some(&Value::Int(10)));
    }

    #[test]
    fn child_shadows_parent() {
        let mut parent = Scope::new();
        parent.set_var("x", Value::Int(10));
        let mut child = Scope::new_child(&parent);
        child.set_var("x", Value::Int(20));
        assert_eq!(child.get_var("x"), Some(&Value::Int(20)));
        assert_eq!(parent.get_var("x"), Some(&Value::Int(10))); // parent unchanged
    }

    #[test]
    fn params() {
        let mut scope = Scope::new();
        scope.set_param("env", Value::String("prod".into()));
        assert_eq!(scope.get_param("env"), Some(&Value::String("prod".into())));
        assert_eq!(scope.get_param("missing"), None);
    }

    #[test]
    fn has_var() {
        let mut scope = Scope::new();
        scope.set_var("a", Value::Int(1));
        assert!(scope.has_var("a"));
        assert!(!scope.has_var("b"));
    }
}

