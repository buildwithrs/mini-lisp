use std::io::{self, Write};

use mini_lisp::env::{Env, EnvRef};
use mini_lisp::value::Value;
use mini_lisp::{builtin, LispInterpreter};

fn main() {
    let root = Env::root();
    register_builtins(&root);
    let interp = LispInterpreter::new(root);

    println!("mini-lisp REPL");
    println!("type (quit) or press Ctrl-D to exit");
    println!();

    loop {
        print!("> ");
        io::stdout().flush().expect("flush failed");

        let mut line = String::new();
        match io::stdin().read_line(&mut line) {
            Ok(0) => {
                // EOF (Ctrl-D on Unix, Ctrl-Z on Windows)
                println!();
                break;
            }
            Ok(_) => {}
            Err(e) => {
                eprintln!("read error: {}", e);
                break;
            }
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed == "(quit)" || trimmed == "(exit)" {
            break;
        }

        match interp.eval(trimmed) {
            Ok(v) => println!("=> {}", v),
            Err(e) => eprintln!("error: {}", e),
        }
    }
}

fn register_builtins(env: &EnvRef) {
    for name in ["+", "-", "*", "/"] {
        Env::define(env, name.into(), Value::Builtin(builtin::arithmetic(name)));
    }
    for name in ["=", "<", "<=", ">", ">="] {
        Env::define(env, name.into(), Value::Builtin(builtin::comparison(name)));
    }
    Env::define(env, "not".into(), Value::Builtin(builtin::boolean("not")));
    for name in ["list", "cons", "car", "cdr", "null?"] {
        Env::define(env, name.into(), Value::Builtin(builtin::list_ops(name)));
    }
    Env::define(env, "print".into(), Value::Builtin(builtin::output("print")));
}
