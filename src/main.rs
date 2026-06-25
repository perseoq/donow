#![allow(dead_code)]

mod ast;
mod error;
mod eval;
mod expand;
mod lexer;
mod module;
mod parser;
mod scope;
mod token;
mod value;

use lexer::Lexer;
use parser::Parser;

fn parse_and_print(label: &str, input: &str) {
    println!("=== {} ===", label);
    let mut lexer = Lexer::new(input);
    let tokens = match lexer.tokenize() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("  Lex error: {}", e);
            return;
        }
    };

    let mut parser = Parser::new(tokens, input);
    match parser.parse() {
        Ok(program) => {
            for block in &program.blocks {
                println!("  block '{}' ({}:{})", block.name, block.span.0, block.span.1);
                print_stmts(&block.body, 4);
            }
        }
        Err(e) => {
            eprintln!("  Parse error: {}", e);
        }
    }
    println!();
}

fn print_stmts(stmts: &[ast::Stmt], indent: usize) {
    let pad = " ".repeat(indent);
    for stmt in stmts {
        match stmt {
            ast::Stmt::Assign { name, value, span } => {
                println!("{}Assign {} = {:?} [{}:{}]", pad, name, value, span.0, span.1);
            }
            ast::Stmt::ColonAssign { target, var_name, span } => {
                println!("{}ColonAssign {:?} : ${} [{}:{}]", pad, target, var_name, span.0, span.1);
            }
            ast::Stmt::If { cond, body, else_body, span } => {
                println!("{}If {:?} [{}:{}]", pad, cond, span.0, span.1);
                print_stmts(body, indent + 4);
                if let Some(eb) = else_body {
                    println!("{}Else:", pad);
                    print_stmts(eb, indent + 4);
                }
            }
            ast::Stmt::While { cond, body, span } => {
                println!("{}While {:?} [{}:{}]", pad, cond, span.0, span.1);
                print_stmts(body, indent + 4);
            }
            ast::Stmt::For { var, iter, body, span } => {
                println!("{}For ${} in {:?} [{}:{}]", pad, var, iter, span.0, span.1);
                print_stmts(body, indent + 4);
            }
            ast::Stmt::Command(text, span) => {
                println!("{}Command {:?} [{}:{}]", pad, text, span.0, span.1);
            }
            ast::Stmt::PriorityBlock(stmts, span) => {
                println!("{}PriorityBlock [{}:{}]", pad, span.0, span.1);
                print_stmts(stmts, indent + 4);
            }
            ast::Stmt::DeferredBlock(stmts, span) => {
                println!("{}DeferredBlock [{}:{}]", pad, span.0, span.1);
                print_stmts(stmts, indent + 4);
            }
            ast::Stmt::BraceBlock(stmts, span) => {
                println!("{}BraceBlock [{}:{}]", pad, span.0, span.1);
                print_stmts(stmts, indent + 4);
            }
        }
    }
}

fn main() {
    parse_and_print(
        "Example DoNow",
        r#"build:
    cargo build

pull:
    {
        $DATE = "date +%F"
        FOLDER = "project/"
        (git add %FOLDER && git commit -m %DATE)
        git push origin main
    }
"#,
    );

    parse_and_print(
        "Control flow",
        r#"test:
    x = 5
    ? if $x == 5:
        echo ok
    e?:
        echo fail
    w! $i < 10:
        echo $i
    f! $i in a[1,2,3]:
        echo $i
    [git push]
    (git pull)
"#,
    );

    parse_and_print(
        "Math and operators",
        r#"calc:
    x + 5 : $result
    a + b * c : $r2
    $r3 == 10
    $value >! 5
"#,
    );
}
