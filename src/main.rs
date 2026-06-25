use std::env;
use std::fs;
use std::path::PathBuf;
use std::process;

mod ast;
mod builtins;
mod error;
mod eval;
mod expand;
mod lexer;
mod module;
mod parser;
mod scope;
mod token;
mod value;

use eval::Eval;
use lexer::Lexer;
use module::ModuleLoader;
use parser::Parser;
use scope::Scope;
use value::Value;

const DONOW_FILE: &str = "DoNow";

fn main() {
    let args: Vec<String> = env::args().collect();
    let donow_dir = get_donow_dir();
    let donow_path = donow_dir.join(DONOW_FILE);

    // Handle flags without block
    if args.len() < 2 || args[1] == "--help" || args[1] == "-h" {
        print_usage();
        return;
    }

    if args[1] == "--init" {
        init_donow(&donow_dir, &donow_path);
        return;
    }

    // First arg is the block name
    let block_name = &args[1];

    // Special: --list can come after block name (list all blocks)
    if block_name == "--list" || args.iter().any(|a| a == "--list" || a == "-l") {
        list_blocks(&donow_path);
        return;
    }

    // Parse remaining args into params (skip first arg which is the block name)
    let params = parse_params(&args[2..]);

    // Read and execute
    let result = execute_block(&donow_path, block_name, &params);

    match result {
        Ok(()) => {}
        Err(msg) => {
            eprintln!("error: {}", msg);
            process::exit(1);
        }
    }
}

// ----------------------------------------------------------------
//  Core execution
// ----------------------------------------------------------------

fn execute_block(path: &PathBuf, block_name: &str, params: &[(String, String)]) -> Result<(), String> {
    // Read file
    let source = fs::read_to_string(path)
        .map_err(|e| format!("cannot read {}: {}", path.display(), e))?;

    // Tokenize
    let mut lexer = Lexer::new(&source);
    let tokens = lexer.tokenize().map_err(|e| format!("lex error: {}", e))?;

    // Parse
    let mut parser = Parser::new(tokens, &source);
    let program = parser.parse().map_err(|e| format!("parse error: {}", e))?;

    // Find block
    let block = program
        .blocks
        .iter()
        .find(|b| b.name == block_name)
        .ok_or_else(|| {
            let available: Vec<&str> = program.blocks.iter().map(|b| b.name.as_str()).collect();
            if available.is_empty() {
                format!("block '{}' not found — no blocks defined in DoNow", block_name)
            } else {
                format!("block '{}' not found. Available blocks: {}", block_name, available.join(", "))
            }
        })?;

    // Set up scope with params
    let mut scope = Scope::new();
    for (key, val) in params {
        scope.set_param(key, Value::String(val.clone()));
    }

    // Execute
    let mut eval = Eval::new(&mut scope);
    eval.eval_body(&block.body).map_err(|e| format!("runtime error: {}", e))?;

    Ok(())
}

// ----------------------------------------------------------------
//  Argument parsing
// ----------------------------------------------------------------

fn parse_params(args: &[String]) -> Vec<(String, String)> {
    let mut params = Vec::new();

    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];

        if arg == "--list" || arg == "-l" {
            i += 1;
            continue;
        }

        if arg.starts_with("--") {
            let inner = arg.trim_start_matches("--");
            if let Some(idx) = inner.find('=') {
                let key = &inner[..idx];
                let val = &inner[idx + 1..];
                params.push((key.to_string(), val.to_string()));
            } else {
                params.push((inner.to_string(), "true".to_string()));
            }
        } else if arg.starts_with('@') {
            let inner = arg.trim_start_matches('@');
            if let Some(idx) = inner.find('=') {
                let key = &inner[..idx];
                let val = &inner[idx + 1..];
                params.push((key.to_string(), val.to_string()));
            } else {
                params.push((inner.to_string(), "true".to_string()));
            }
        } else {
            // Positional argument
            params.push((i.to_string(), arg.clone()));
        }

        i += 1;
    }

    params
}

// ----------------------------------------------------------------
//  File / directory helpers
// ----------------------------------------------------------------

fn get_donow_dir() -> PathBuf {
    let home = env::var("HOME")
        .or_else(|_| env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join(".donow")
}

fn init_donow(dir: &PathBuf, file: &PathBuf) {
    // Create directories
    let loader = ModuleLoader::with_dir(dir.clone());
    if let Err(e) = loader.ensure_dirs() {
        eprintln!("error: {}", e);
        process::exit(1);
    }

    // Create example DoNow if not exists
    if !file.exists() {
        let example = r#"# DoNow — command & script runner
#
# Usage:
#   donow build        -> runs the build block
#   donow build --opt  -> @opt = "true"
#
build:
    echo "building..."

test:
    echo "running tests..."

deploy:
    {
        echo "deploying to @env..."
        git push origin main
    }
"#;
        fs::write(file, example).unwrap_or_else(|e| {
            eprintln!("error: cannot create {}: {}", file.display(), e);
            process::exit(1);
        });
        println!("created {}", file.display());
    }

    println!("donow initialized at {}", dir.display());
}

// ----------------------------------------------------------------
//  Listing
// ----------------------------------------------------------------

fn list_blocks(path: &PathBuf) {
    let source = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: cannot read {}: {}", path.display(), e);
            process::exit(1);
        }
    };

    let mut lexer = Lexer::new(&source);
    let tokens = match lexer.tokenize() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("lex error: {}", e);
            process::exit(1);
        }
    };

    let mut parser = Parser::new(tokens, &source);
    let program = match parser.parse() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("parse error: {}", e);
            process::exit(1);
        }
    };

    if program.blocks.is_empty() {
        println!("no blocks defined in {}", path.display());
        return;
    }

    println!("Available blocks:");
    for block in &program.blocks {
        println!("  {}", block.name);
    }
}

fn print_usage() {
    let name = env::args().next().unwrap_or_else(|| "donow".into());
    eprintln!("Usage: {} <block> [@param=value ...] [--flag ...]", name);
    eprintln!();
    eprintln!("Arguments:");
    eprintln!("  <block>              Name of the block to execute");
    eprintln!("  @param=value         Set a CLI parameter (accessible via @param)");
    eprintln!("  --flag               Shorthand for @flag=true");
    eprintln!("  --key=value          Shorthand for @key=value");
    eprintln!();
    eprintln!("Commands:");
    eprintln!("  {} --list           List available blocks", name);
    eprintln!("  {} --init           Create ~/.donow/DoNow with example", name);
    eprintln!("  {} --help           Show this help", name);
    eprintln!();
    eprintln!("Examples:");
    eprintln!("  {} build", name);
    eprintln!("  {} deploy @env=prod --verbose", name);
    eprintln!("  {} test", name);
}
