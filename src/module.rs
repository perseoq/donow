use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ModuleError {
    pub message: String,
}

impl ModuleError {
    pub fn new(message: impl Into<String>) -> Self {
        ModuleError {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for ModuleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ModuleError {}

// ----------------------------------------------------------------
//  ModuleLoader
// ----------------------------------------------------------------

pub struct ModuleLoader {
    donow_dir: PathBuf,
}

impl ModuleLoader {
    pub fn new() -> Self {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_else(|_| ".".into());
        ModuleLoader {
            donow_dir: PathBuf::from(home).join(".donow"),
        }
    }

    pub fn with_dir(dir: PathBuf) -> Self {
        ModuleLoader { donow_dir: dir }
    }

    pub fn donow_dir(&self) -> &PathBuf {
        &self.donow_dir
    }

    /// Load a template file: ~/.donow/templates/<name>
    pub fn load_template(&self, name: &str) -> Result<String, ModuleError> {
        self.load_file("templates", name)
    }

    /// Load a function file: ~/.donow/funcs/<name>
    pub fn load_func(&self, name: &str) -> Result<String, ModuleError> {
        self.load_file("funcs", name)
    }

    /// Load a class file: ~/.donow/classes/<name>
    pub fn load_class(&self, name: &str) -> Result<String, ModuleError> {
        self.load_file("classes", name)
    }

    /// List all available templates
    pub fn list_templates(&self) -> Result<Vec<String>, ModuleError> {
        self.list_dir("templates")
    }

    /// List all available functions
    pub fn list_funcs(&self) -> Result<Vec<String>, ModuleError> {
        self.list_dir("funcs")
    }

    /// List all available classes
    pub fn list_classes(&self) -> Result<Vec<String>, ModuleError> {
        self.list_dir("classes")
    }

    /// Check if a template exists
    pub fn template_exists(&self, name: &str) -> bool {
        self.donow_dir.join("templates").join(name).exists()
    }

    /// Check if a function exists
    pub fn func_exists(&self, name: &str) -> bool {
        self.donow_dir.join("funcs").join(name).exists()
    }

    /// Check if a class exists
    pub fn class_exists(&self, name: &str) -> bool {
        self.donow_dir.join("classes").join(name).exists()
    }

    /// Ensure all module directories exist
    pub fn ensure_dirs(&self) -> Result<(), ModuleError> {
        for sub in &["templates", "funcs", "classes"] {
            let dir = self.donow_dir.join(sub);
            fs::create_dir_all(&dir).map_err(|e| {
                ModuleError::new(format!("failed to create {}: {}", dir.display(), e))
            })?;
        }
        Ok(())
    }

    // --------------------------------------------------------
    //  Internal helpers
    // --------------------------------------------------------

    fn load_file(&self, subdir: &str, name: &str) -> Result<String, ModuleError> {
        let path = self.donow_dir.join(subdir).join(name);
        if !path.exists() {
            return Err(ModuleError::new(format!(
                "{} '{}' not found at {}",
                subdir.trim_end_matches('s'),
                name,
                path.display()
            )));
        }
        fs::read_to_string(&path).map_err(|e| {
            ModuleError::new(format!("failed to read {}: {}", path.display(), e))
        })
    }

    fn list_dir(&self, subdir: &str) -> Result<Vec<String>, ModuleError> {
        let dir = self.donow_dir.join(subdir);
        if !dir.exists() {
            return Ok(vec![]);
        }
        let mut entries = Vec::new();
        let rd = fs::read_dir(&dir).map_err(|e| {
            ModuleError::new(format!("failed to list {}: {}", dir.display(), e))
        })?;
        for entry in rd {
            let entry = entry.map_err(|e| {
                ModuleError::new(format!("failed to read entry: {}", e))
            })?;
            let ft = entry.file_type().map_err(|e| {
                ModuleError::new(format!("failed to get file type: {}", e))
            })?;
            if ft.is_file() {
                if let Some(name) = entry.file_name().to_str() {
                    entries.push(name.to_string());
                }
            }
        }
        entries.sort();
        Ok(entries)
    }
}

impl Default for ModuleLoader {
    fn default() -> Self {
        Self::new()
    }
}

// ----------------------------------------------------------------
//  Tests
// ----------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn setup_test_dir() -> PathBuf {
        let n = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!("donow_test_{}", n));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write(dir: &Path, sub: &str, name: &str, content: &str) {
        let subdir = dir.join(sub);
        fs::create_dir_all(&subdir).unwrap();
        fs::write(subdir.join(name), content).unwrap();
    }

    #[test]
    fn load_template() {
        let dir = setup_test_dir();
        write(&dir, "templates", "greeting", "Hello, %name!");

        let loader = ModuleLoader::with_dir(dir.clone());
        let content = loader.load_template("greeting").unwrap();
        assert_eq!(content, "Hello, %name!");
    }

    #[test]
    fn load_func() {
        let dir = setup_test_dir();
        write(&dir, "funcs", "encrypt", "echo %1");

        let loader = ModuleLoader::with_dir(dir);
        let content = loader.load_func("encrypt").unwrap();
        assert_eq!(content, "echo %1");
    }

    #[test]
    fn load_class() {
        let dir = setup_test_dir();
        write(&dir, "classes", "Server", "deploy:\n    echo deploying");

        let loader = ModuleLoader::with_dir(dir);
        let content = loader.load_class("Server").unwrap();
        assert!(content.contains("deploy"));
    }

    #[test]
    fn load_missing() {
        let dir = setup_test_dir();
        let loader = ModuleLoader::with_dir(dir);
        let err = loader.load_template("nonexistent").unwrap_err();
        assert!(err.message.contains("not found"));
    }

    #[test]
    fn list_templates() {
        let dir = setup_test_dir();
        write(&dir, "templates", "a", "");
        write(&dir, "templates", "b", "");

        let loader = ModuleLoader::with_dir(dir);
        let list = loader.list_templates().unwrap();
        assert_eq!(list, vec!["a", "b"]);
    }

    #[test]
    fn list_empty_dir() {
        let dir = setup_test_dir();
        let loader = ModuleLoader::with_dir(dir);
        assert!(loader.list_templates().unwrap().is_empty());
    }

    #[test]
    fn exists() {
        let dir = setup_test_dir();
        write(&dir, "templates", "exists", "");

        let loader = ModuleLoader::with_dir(dir);
        assert!(loader.template_exists("exists"));
        assert!(!loader.template_exists("missing"));
    }

    #[test]
    fn ensure_dirs_creates_them() {
        let dir = setup_test_dir();
        let loader = ModuleLoader::with_dir(dir.clone());
        loader.ensure_dirs().unwrap();
        assert!(dir.join("templates").exists());
        assert!(dir.join("funcs").exists());
        assert!(dir.join("classes").exists());
    }

    #[test]
    fn load_empty_file() {
        let dir = setup_test_dir();
        write(&dir, "templates", "empty", "");

        let loader = ModuleLoader::with_dir(dir);
        assert_eq!(loader.load_template("empty").unwrap(), "");
    }

    #[test]
    fn donow_dir_path() {
        let dir = setup_test_dir();
        let loader = ModuleLoader::with_dir(dir.clone());
        assert_eq!(*loader.donow_dir(), dir);
    }
}
