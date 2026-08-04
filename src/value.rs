use std::{fmt, rc::Rc};

use crate::{
    env::{Env, EnvRef},
    errors::EvalError,
    expr::Expr,
};

#[derive(Clone)]
pub enum Value {
    Integer(i64),
    Boolean(bool),
    Str(String),
    Symbol(String), // Produced by quote.
    Nil,
    List(Vec<Value>), // Invariant: contains at least one item.
    Builtin(Builtin),
    Lambda(Rc<Lambda>),
}

#[derive(Clone, Copy)]
pub struct Builtin {
    pub name: &'static str,
    pub function: BuiltinFn,
}

pub type BuiltinFn = fn(&[Value]) -> Result<Value, EvalError>;

#[derive(Clone)]
pub struct Lambda {
    pub params: Vec<String>,
    pub body: Vec<Expr>,
    pub env: crate::env::EnvRef,
}

impl Value {
    pub fn list(items: Vec<Value>) -> Self {
        if items.is_empty() {
            Self::Nil
        } else {
            Self::List(items)
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Integer(n) => write!(f, "{}", n),
            Value::Boolean(b) => write!(f, "{}", if *b { "#t" } else { "#f" }),
            Value::Str(s) => write!(f, "\"{}\"", s),
            Value::Symbol(s) => write!(f, "{}", s),
            Value::Nil => write!(f, "nil"),
            Value::List(items) => {
                write!(f, "(")?;
                for (i, v) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", v)?;
                }
                write!(f, ")")
            }
            Value::Builtin(b) => write!(f, "<builtin:{}>", b.name),
            Value::Lambda(_) => write!(f, "<lambda>"),
        }
    }
}

pub fn eval(expr: &Expr, env: EnvRef) -> Result<Value, EvalError> {
    match expr {
        Expr::Integer(v) => Ok(Value::Integer(*v)),
        Expr::Boolean(v) => Ok(Value::Boolean(*v)),
        Expr::Str(v) => Ok(Value::Str(v.clone())),
        Expr::Nil => Ok(Value::Nil),
        Expr::Symbol(sym) => Ok(Env::lookup(&env, &sym).ok_or_else(|| EvalError::UndefinedSymbol)?),
        Expr::List(list_expr) => Ok(eval_list(list_expr, env.clone())?),
        _ => Err(EvalError::BadSpecialForm("unknown expr")),
    }
}

fn eval_list(list: &Vec<Expr>, env: EnvRef) -> Result<Value, EvalError> {
    let (head, args) = list
        .split_first()
        .ok_or_else(|| EvalError::EmptyApplication)?;

    if let Expr::Symbol(sym) = head {
        match sym.as_str() {
            "quote" => return eval_quote(args),
            "if" => return eval_if(args, env),
            "define" => return eval_define(args, env),
            "lambda" => return eval_lambda(args, env),
            "begin" => return eval_begin(args, env),
            _ => {}
        }
    }

    let caller = eval(head, env.clone())?;
    let mut values = Vec::new();
    for arg in args {
        values.push(eval(arg, env.clone())?);
    }

    match caller {
        Value::Builtin(b) => (b.function)(&values),
        Value::Lambda(c) => apply_lambda(&c, &values),
        other => Err(EvalError::NotCallable(other.to_string())),
    }
}

fn apply_lambda(lambda: &Lambda, args: &[Value]) -> Result<Value, EvalError> {
    if lambda.params.len() != args.len() {
        return Err(EvalError::ArgNotMatch);
    }

    let call_env = Env::child(lambda.env.clone());
    for (p, arg) in lambda.params.iter().zip(args) {
        Env::define(&call_env, p.clone(), arg.clone());
    }

    eval_begin(&lambda.body, call_env)
}

fn eval_quote(args: &[Expr]) -> Result<Value, EvalError> {
    let first = args.first().ok_or(EvalError::BadSpecialForm("quote"))?;
    Ok(quote_to_value(first))
}

fn quote_to_value(expr: &Expr) -> Value {
    match expr {
        Expr::Integer(v) => Value::Integer(*v),
        Expr::Boolean(v) => Value::Boolean(*v),
        Expr::Str(v) => Value::Str(v.clone()),
        Expr::Nil => Value::Nil,
        Expr::Symbol(v) => Value::Symbol(v.clone()),
        Expr::List(items) => Value::List(items.iter().map(quote_to_value).collect()),
    }
}

fn eval_if(args: &[Expr], env: EnvRef) -> Result<Value, EvalError> {
    if args.len() != 3 {
        return Err(EvalError::BadSpecialForm("if"));
    }

    let cond = eval(&args[0], env.clone())?;
    if !matches!(cond, Value::Boolean(false) | Value::Nil) {
        eval(&args[1], env.clone())
    } else {
        eval(&args[2], env.clone())
    }
}

fn eval_define(args: &[Expr], env: EnvRef) -> Result<Value, EvalError> {
    // define needs 2 parameters
    if args.len() != 2 {
        return Err(EvalError::BadSpecialForm("define"));
    }

    let name = match args[0].clone() {
        Expr::Symbol(name) => name.clone(),
        _ => return Err(EvalError::BadSpecialForm("define need name")),
    };

    Env::define(&env, name, eval(&args[1], env.clone())?);
    Ok(Value::Nil)
}

fn eval_lambda(args: &[Expr], env: EnvRef) -> Result<Value, EvalError> {
    let pe = args.first().ok_or(EvalError::BadSpecialForm("lambda"))?;
    let body = &args[1..];
    if body.is_empty() {
        return Err(EvalError::BadSpecialForm("lambda"));
    }

    let params = match pe {
        Expr::Nil => vec![],
        Expr::List(items) => {
            let mut ps = Vec::new();
            for item in items {
                if let Expr::Symbol(name) = item {
                    ps.push(name.clone());
                } else {
                    return Err(EvalError::BadSpecialForm("lambda"));
                }
            }
            ps.dedup();
            ps
        }
        _ => return Err(EvalError::BadSpecialForm("lambda")),
    };

    Ok(Value::Lambda(Rc::new(Lambda {
        params,
        body: body.to_vec(),
        env,
    })))
}

fn eval_begin(exprs: &[Expr], env: EnvRef) -> Result<Value, EvalError> {
    let mut last = Value::Nil;
    for e in exprs {
        last = eval(e, env.clone())?;
    }

    Ok(last)
}


pub fn expect_arity(args: &[Value], n: usize) -> Result<(), EvalError> {
    if args.len() != n {
        Err(EvalError::ArgNotMatch)
    } else {
        Ok(())
    }
}

pub fn expect_integer(v: &Value) -> Result<i64, EvalError> {
    if let Value::Integer(n) = v { Ok(*n) } else { Err(EvalError::BadArg("integer")) }
}

pub fn expect_list(v: &Value) -> Result<&[Value], EvalError> {
    match v {
        Value::Nil => Ok(&[]),
        Value::List(items) => Ok(items),
        _ => Err(EvalError::BadArg("list")),
    }
}