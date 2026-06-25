use std::fs;
use std::path::PathBuf;

use crate::scope::Scope;
use crate::value::Value;

#[derive(Debug, Clone)]
pub struct ExpandError {
    pub message: String,
}

impl ExpandError {
    pub fn new(message: impl Into<String>) -> Self {
        ExpandError {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for ExpandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ExpandError {}

// ----------------------------------------------------------------
//  Expander
// ----------------------------------------------------------------

pub struct Expander {
    donow_dir: PathBuf,
}

impl Expander {
    pub fn new() -> Self {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_else(|_| ".".into());
        Expander {
            donow_dir: PathBuf::from(home).join(".donow"),
        }
    }

    pub fn with_dir(dir: PathBuf) -> Self {
        Expander { donow_dir: dir }
    }

    /// Expand all variable references, templates, functions, and classes
    /// in a command string.
    pub fn expand(&self, text: &str, scope: &Scope) -> Result<String, ExpandError> {
        let mut out = String::new();
        let chars: Vec<char> = text.chars().collect();
        let len = chars.len();
        let mut i = 0;

        while i < len {
            // %var
            if chars[i] == '%' && i + 1 < len && is_id_start(chars[i + 1]) {
                i += 1;
                let start = i;
                while i < len && is_id_cont(chars[i]) {
                    i += 1;
                }
                let name: String = chars[start..i].iter().collect();
                match scope.get_var(&name) {
                    Some(v) => out.push_str(&v.to_string()),
                    None => {
                        return Err(ExpandError::new(format!(
                            "undefined variable %{}",
                            name
                        )));
                    }
                }
                continue;
            }

            // @param
            if chars[i] == '@' && i + 1 < len && is_id_start(chars[i + 1]) {
                i += 1;
                let start = i;
                while i < len && is_id_cont(chars[i]) {
                    i += 1;
                }
                let name: String = chars[start..i].iter().collect();
                match scope.get_param(&name) {
                    Some(v) => out.push_str(&v.to_string()),
                    None => {
                        return Err(ExpandError::new(format!(
                            "undefined CLI param @{}",
                            name
                        )));
                    }
                }
                continue;
            }

            // $var
            if chars[i] == '$' && i + 1 < len && is_id_start(chars[i + 1]) {
                i += 1;
                let start = i;
                while i < len && is_id_cont(chars[i]) {
                    i += 1;
                }
                let name: String = chars[start..i].iter().collect();
                match scope.get_var(&name) {
                    Some(v) => out.push_str(&v.to_string()),
                    None => {
                        return Err(ExpandError::new(format!(
                            "undefined variable ${}",
                            name
                        )));
                    }
                }
                continue;
            }

            // T<name>
            if chars[i] == 'T'
                && i + 1 < len
                && chars[i + 1] == '<'
            {
                i += 2; // skip T<
                let start = i;
                while i < len && chars[i] != '>' {
                    i += 1;
                }
                if i >= len {
                    return Err(ExpandError::new(
                        "unclosed T< template reference",
                    ));
                }
                let name: String = chars[start..i].iter().collect();
                i += 1; // skip >
                let content = self.load_template(&name)?;
                out.push_str(&content);
                continue;
            }

            // C<name>
            if chars[i] == 'C'
                && i + 1 < len
                && chars[i + 1] == '<'
            {
                i += 2; // skip C<
                let start = i;
                while i < len && chars[i] != '>' {
                    i += 1;
                }
                if i >= len {
                    return Err(ExpandError::new(
                        "unclosed C< class reference",
                    ));
                }
                let name: String = chars[start..i].iter().collect();
                i += 1;
                let content = self.load_class(&name)?;
                out.push_str(&content);
                continue;
            }

            // F<name>(...)
            if chars[i] == 'F'
                && i + 1 < len
                && chars[i + 1] == '<'
            {
                i += 2;
                let start = i;
                while i < len && chars[i] != '>' {
                    i += 1;
                }
                if i >= len {
                    return Err(ExpandError::new(
                        "unclosed F< function reference",
                    ));
                }
                let name: String = chars[start..i].iter().collect();
                i += 1;

                // parse optional (args)
                let mut args: Vec<String> = Vec::new();
                if i < len && chars[i] == '(' {
                    i += 1;
                    let mut current = String::new();
                    let mut depth = 1;
                    while i < len && depth > 0 {
                        match chars[i] {
                            '(' => depth += 1,
                            ')' => {
                                depth -= 1;
                                if depth == 0 {
                                    if !current.is_empty() {
                                        args.push(current.trim().to_string());
                                    }
                                    i += 1;
                                    break;
                                }
                            }
                            ',' if depth == 1 => {
                                args.push(current.trim().to_string());
                                current.clear();
                            }
                            c => current.push(c),
                        }
                        i += 1;
                    }
                }

                let result = self.call_func(&name, &args)?;
                out.push_str(&result);
                continue;
            }

            out.push(chars[i]);
            i += 1;
        }

        Ok(out)
    }

    // --------------------------------------------------------
    //  Template / Function / Class loading
    // --------------------------------------------------------

    fn load_file(&self, subdir: &str, name: &str) -> Result<String, ExpandError> {
        let path = self.donow_dir.join(subdir).join(name);
        if !path.exists() {
            return Err(ExpandError::new(format!(
                "{} file not found: {}",
                subdir,
                path.display()
            )));
        }
        fs::read_to_string(&path).map_err(|e| {
            ExpandError::new(format!("failed to read {}: {}", path.display(), e))
        })
    }

    fn load_template(&self, name: &str) -> Result<String, ExpandError> {
        self.load_file("templates", name)
    }

    fn load_class(&self, name: &str) -> Result<String, ExpandError> {
        self.load_file("classes", name)
    }

    fn call_func(&self, name: &str, args: &[String]) -> Result<String, ExpandError> {
        let content = self.load_file("funcs", name)?;

        // Create a scope with positional arguments as %1, %2, ...
        let mut fn_scope = Scope::new();
        for (idx, arg) in args.iter().enumerate() {
            fn_scope.set_var(&(idx + 1).to_string(), Value::String(arg.clone()));
            fn_scope.set_param(&(idx + 1).to_string(), Value::String(arg.clone()));
        }

        // Expand the function content with the argument scope
        let expanded = self.expand(&content, &fn_scope)?;

        Ok(expanded)
    }
}

impl Default for Expander {
    fn default() -> Self {
        Self::new()
    }
}

fn is_id_start(c: char) -> bool {
    c.is_alphabetic() || c == '_'
}

fn is_id_cont(c: char) -> bool {
    c.is_alphanumeric()
}

// ----------------------------------------------------------------
//  Tests
// ----------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_scope() -> Scope {
        let mut s = Scope::new();
        s.set_var("name", Value::String("world".into()));
        s.set_var("x", Value::Int(42));
        s.set_param("env", Value::String("prod".into()));
        s
    }

    fn expand(text: &str) -> String {
        let scope = make_scope();
        let e = Expander::with_dir(PathBuf::from("/tmp/donow_test"));
        e.expand(text, &scope).unwrap()
    }

    fn expand_err(text: &str) -> ExpandError {
        let scope = make_scope();
        let e = Expander::with_dir(PathBuf::from("/tmp/donow_test"));
        e.expand(text, &scope).unwrap_err()
    }

    #[test]
    fn no_vars() {
        assert_eq!(expand("hello world"), "hello world");
    }

    #[test]
    fn percent_var() {
        assert_eq!(expand("echo %name"), "echo world");
    }

    #[test]
    fn dollar_var() {
        assert_eq!(expand("echo $x"), "echo 42");
    }

    #[test]
    fn at_param() {
        assert_eq!(expand("deploy @env"), "deploy prod");
    }

    #[test]
    fn multiple_vars() {
        assert_eq!(
            expand("hello %name, x=$x, env=@env"),
            "hello world, x=42, env=prod"
        );
    }

    #[test]
    fn undefined_var_error() {
        let err = expand_err("echo %nonexistent");
        assert!(err.message.contains("%nonexistent"));
    }

    #[test]
    fn template_not_found() {
        let err = expand_err("T<missing>");
        assert!(err.message.contains("not found"));
    }

    #[test]
    fn func_not_found() {
        let err = expand_err("F<missing>()");
        assert!(err.message.contains("not found"));
    }

    #[test]
    fn class_not_found() {
        let err = expand_err("C<missing>");
        assert!(err.message.contains("not found"));
    }

    #[test]
    fn unclosed_template() {
        let err = expand_err("T<unclosed");
        assert!(err.message.contains("unclosed"));
    }

    #[test]
    fn mixed_text_and_refs() {
        assert_eq!(
            expand("prefix_%name_suffix"),
            "prefix_world_suffix"
        );
    }

    #[test]
    fn func_with_args_raw() {
        // Just test that F<name>(arg1, arg2) gets parsed correctly
        // and tries to load the function file (which will fail)
        let scope = make_scope();
        let e = Expander::with_dir(PathBuf::from("/tmp/donow_test"));
        let result = e.expand("run F<deploy>(prod, us-east)", &scope);
        assert!(result.is_err());
        // It tried to load funcs/deploy
        assert!(result.unwrap_err().message.contains("funcs"));
    }

    #[test]
    fn dollar_not_var() {
        // $ followed by non-alphabetic should be literal
        assert_eq!(expand("$100"), "$100");
    }

    #[test]
    fn percent_not_var() {
        // % followed by non-alphabetic should be literal
        assert_eq!(expand("100%"), "100%");
    }
}
