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

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Integer(n) => write!(f, "Integer({})", n),
            Value::Boolean(b) => write!(f, "Boolean({})", b),
            Value::Str(s) => write!(f, "Str({:?})", s),
            Value::Symbol(s) => write!(f, "Symbol({:?})", s),
            Value::Nil => write!(f, "Nil"),
            Value::List(items) => f.debug_list().entries(items).finish(),
            Value::Builtin(b) => write!(f, "Builtin({:?})", b.name),
            Value::Lambda(_) => write!(f, "Lambda(<fn>)"),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Integer(a), Value::Integer(b)) => a == b,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Str(a), Value::Str(b)) => a == b,
            (Value::Symbol(a), Value::Symbol(b)) => a == b,
            (Value::Nil, Value::Nil) => true,
            (Value::List(a), Value::List(b)) => a == b,
            _ => false,  // Builtin and Lambda are never equal to anything
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
    if args.len() != 1 {
        return Err(EvalError::BadSpecialForm("quote"));
    }
    Ok(quote_to_value(&args[0]))
}

fn quote_to_value(expr: &Expr) -> Value {
    match expr {
        Expr::Integer(v) => Value::Integer(*v),
        Expr::Boolean(v) => Value::Boolean(*v),
        Expr::Str(v) => Value::Str(v.clone()),
        Expr::Nil => Value::Nil,
        Expr::Symbol(v) => Value::Symbol(v.clone()),
        Expr::List(items) => Value::list(items.iter().map(quote_to_value).collect()),
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

    let val = eval(&args[1], env.clone())?;
    Env::define(&env, name, val.clone());
    Ok(val)
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

            let mut sorted = ps.clone();
            sorted.sort();
            sorted.dedup();
            if sorted.len() != ps.len() {
                return Err(EvalError::BadSpecialForm("duplicate parameter"));
            }
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
    if let Value::Integer(n) = v {
        Ok(*n)
    } else {
        Err(EvalError::BadArg("integer"))
    }
}

pub fn expect_list(v: &Value) -> Result<&[Value], EvalError> {
    match v {
        Value::Nil => Ok(&[]),
        Value::List(items) => Ok(items),
        _ => Err(EvalError::BadArg("list")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fresh() -> EnvRef {
        Env::root()
    }

    fn run(expr: &Expr) -> Result<Value, EvalError> {
        eval(expr, fresh())
    }

    fn run_in(expr: &Expr, env: EnvRef) -> Result<Value, EvalError> {
        eval(expr, env)
    }

    fn expect_int(v: Value) -> i64 {
        match v {
            Value::Integer(n) => n,
            _ => panic!("expected Integer"),
        }
    }

    fn expect_nil(v: Value) {
        match v {
            Value::Nil => {}
            _ => panic!("expected Nil"),
        }
    }

    fn expect_lambda(v: Value) {
        match v {
            Value::Lambda(_) => {}
            _ => panic!("expected Lambda"),
        }
    }

    fn mk_builtin(name: &'static str, f: BuiltinFn) -> Value {
        Value::Builtin(Builtin { name, function: f })
    }

    fn add_2(args: &[Value]) -> Result<Value, EvalError> {
        expect_arity(args, 2)?;
        Ok(Value::Integer(
            expect_integer(&args[0])? + expect_integer(&args[1])?,
        ))
    }

    // ===== Display =====

    #[test]
    fn display_integer() {
        assert_eq!(format!("{}", Value::Integer(42)), "42");
    }

    #[test]
    fn display_boolean() {
        assert_eq!(format!("{}", Value::Boolean(true)), "#t");
        assert_eq!(format!("{}", Value::Boolean(false)), "#f");
    }

    #[test]
    fn display_nil() {
        assert_eq!(format!("{}", Value::Nil), "nil");
    }

    #[test]
    fn display_string() {
        assert_eq!(format!("{}", Value::Str("hi".into())), "\"hi\"");
    }

    #[test]
    fn display_symbol() {
        assert_eq!(format!("{}", Value::Symbol("foo".into())), "foo");
    }

    #[test]
    fn display_list() {
        let v = Value::list(vec![Value::Integer(1), Value::Integer(2)]);
        assert_eq!(format!("{}", v), "(1 2)");
    }

    #[test]
    fn display_empty_list_is_nil() {
        let v = Value::list(vec![]);
        assert_eq!(format!("{}", v), "nil");
    }

    // ===== Value::list invariant =====

    #[test]
    fn list_helper_makes_nil_for_empty() {
        assert!(matches!(Value::list(vec![]), Value::Nil));
    }

    #[test]
    fn list_helper_makes_list_for_non_empty() {
        match Value::list(vec![Value::Integer(1)]) {
            Value::List(items) => assert_eq!(items.len(), 1),
            _ => panic!("expected List"),
        }
    }

    // ===== self-evaluating =====

    #[test]
    fn integer_self_evaluates() {
        assert_eq!(expect_int(run(&Expr::Integer(42)).unwrap()), 42);
    }

    #[test]
    fn boolean_self_evaluates() {
        assert!(matches!(
            run(&Expr::Boolean(true)).unwrap(),
            Value::Boolean(true)
        ));
        assert!(matches!(
            run(&Expr::Boolean(false)).unwrap(),
            Value::Boolean(false)
        ));
    }

    #[test]
    fn nil_self_evaluates() {
        expect_nil(run(&Expr::Nil).unwrap());
    }

    #[test]
    fn string_self_evaluates() {
        assert!(matches!(
            run(&Expr::Str("hi".into())).unwrap(),
            Value::Str(s) if s == "hi"
        ));
    }

    // ===== symbol lookup =====

    #[test]
    fn symbol_looks_up_in_env() {
        let env = fresh();
        Env::define(&env, "answer".into(), Value::Integer(42));
        assert_eq!(
            expect_int(run_in(&Expr::Symbol("answer".into()), env).unwrap()),
            42
        );
    }

    #[test]
    fn undefined_symbol_is_error() {
        assert!(matches!(
            run(&Expr::Symbol("missing".into())),
            Err(EvalError::UndefinedSymbol)
        ));
    }

    #[test]
    fn symbol_walks_parent_chain() {
        let parent = fresh();
        Env::define(&parent, "x".into(), Value::Integer(1));
        let child = Env::child(parent.clone());
        Env::define(&child, "y".into(), Value::Integer(2));
        assert_eq!(
            expect_int(run_in(&Expr::Symbol("x".into()), child).unwrap()),
            1
        );
    }

    // ===== quote =====

    #[test]
    fn quote_returns_data_unchanged() {
        let expr = Expr::List(vec![
            Expr::Symbol("quote".into()),
            Expr::List(vec![Expr::Integer(1), Expr::Integer(2)]),
        ]);
        let result = run(&expr).unwrap();
        match result {
            Value::List(items) => {
                assert_eq!(items.len(), 2);
                assert_eq!(expect_int(items[0].clone()), 1);
                assert_eq!(expect_int(items[1].clone()), 2);
            }
            _ => panic!("expected List"),
        }
    }

    #[test]
    fn quote_does_not_evaluate_symbol() {
        let expr = Expr::List(vec![
            Expr::Symbol("quote".into()),
            Expr::Symbol("missing".into()),
        ]);
        let result = run(&expr).unwrap();
        assert!(matches!(result, Value::Symbol(s) if s == "missing"));
    }

    #[test]
    fn quote_nested_lists() {
        // '(1 (2 3))  =>  (1 (2 3))
        let expr = Expr::List(vec![
            Expr::Symbol("quote".into()),
            Expr::List(vec![
                Expr::Integer(1),
                Expr::List(vec![Expr::Integer(2), Expr::Integer(3)]),
            ]),
        ]);
        let result = run(&expr).unwrap();
        match result {
            Value::List(outer) => {
                assert_eq!(outer.len(), 2);
                assert_eq!(expect_int(outer[0].clone()), 1);
                match &outer[1] {
                    Value::List(inner) => {
                        assert_eq!(inner.len(), 2);
                        assert_eq!(expect_int(inner[0].clone()), 2);
                        assert_eq!(expect_int(inner[1].clone()), 3);
                    }
                    _ => panic!("expected nested List"),
                }
            }
            _ => panic!("expected List"),
        }
    }

    // ===== if =====

    #[test]
    fn if_takes_then_branch_when_truthy() {
        let expr = Expr::List(vec![
            Expr::Symbol("if".into()),
            Expr::Boolean(true),
            Expr::Integer(1),
            Expr::Integer(2),
        ]);
        assert_eq!(expect_int(run(&expr).unwrap()), 1);
    }

    #[test]
    fn if_takes_else_branch_when_false() {
        let expr = Expr::List(vec![
            Expr::Symbol("if".into()),
            Expr::Boolean(false),
            Expr::Integer(1),
            Expr::Integer(2),
        ]);
        assert_eq!(expect_int(run(&expr).unwrap()), 2);
    }

    #[test]
    fn if_takes_else_branch_when_nil() {
        let expr = Expr::List(vec![
            Expr::Symbol("if".into()),
            Expr::Nil,
            Expr::Integer(1),
            Expr::Integer(2),
        ]);
        assert_eq!(expect_int(run(&expr).unwrap()), 2);
    }

    #[test]
    fn zero_is_truthy() {
        let expr = Expr::List(vec![
            Expr::Symbol("if".into()),
            Expr::Integer(0),
            Expr::Integer(1),
            Expr::Integer(2),
        ]);
        assert_eq!(expect_int(run(&expr).unwrap()), 1);
    }

    #[test]
    fn empty_string_is_truthy() {
        let expr = Expr::List(vec![
            Expr::Symbol("if".into()),
            Expr::Str(String::new()),
            Expr::Integer(1),
            Expr::Integer(2),
        ]);
        assert_eq!(expect_int(run(&expr).unwrap()), 1);
    }

    #[test]
    fn if_skips_else_branch_with_undefined_symbol() {
        let expr = Expr::List(vec![
            Expr::Symbol("if".into()),
            Expr::Boolean(true),
            Expr::Integer(1),
            Expr::Symbol("missing-else".into()),
        ]);
        assert_eq!(expect_int(run(&expr).unwrap()), 1);
    }

    #[test]
    fn if_skips_then_branch_with_undefined_symbol() {
        let expr = Expr::List(vec![
            Expr::Symbol("if".into()),
            Expr::Boolean(false),
            Expr::Symbol("missing-then".into()),
            Expr::Integer(99),
        ]);
        assert_eq!(expect_int(run(&expr).unwrap()), 99);
    }

    #[test]
    fn if_wrong_arity_is_error() {
        let expr = Expr::List(vec![
            Expr::Symbol("if".into()),
            Expr::Boolean(true),
            Expr::Integer(1),
        ]);
        assert!(matches!(run(&expr), Err(EvalError::BadSpecialForm("if"))));
    }

    // ===== define =====

    #[test]
    fn define_binds_value_in_env() {
        let env = fresh();
        let expr = Expr::List(vec![
            Expr::Symbol("define".into()),
            Expr::Symbol("x".into()),
            Expr::Integer(42),
        ]);
        run_in(&expr, env.clone()).unwrap();
        assert_eq!(
            expect_int(run_in(&Expr::Symbol("x".into()), env).unwrap()),
            42
        );
    }

    #[test]
    fn define_wrong_arity_is_error() {
        let expr = Expr::List(vec![
            Expr::Symbol("define".into()),
            Expr::Symbol("x".into()),
        ]);
        assert!(matches!(
            run(&expr),
            Err(EvalError::BadSpecialForm("define"))
        ));
    }

    #[test]
    fn define_non_symbol_target_is_error() {
        let expr = Expr::List(vec![
            Expr::Symbol("define".into()),
            Expr::Integer(1),
            Expr::Integer(2),
        ]);
        assert!(matches!(run(&expr), Err(EvalError::BadSpecialForm(_))));
    }

    // ===== begin =====

    #[test]
    fn begin_returns_last_value() {
        let env = fresh();
        Env::define(&env, "x".into(), Value::Integer(10));
        let expr = Expr::List(vec![
            Expr::Symbol("begin".into()),
            Expr::Integer(1),
            Expr::Integer(2),
            Expr::Symbol("x".into()),
        ]);
        assert_eq!(expect_int(run_in(&expr, env).unwrap()), 10);
    }

    #[test]
    fn empty_begin_returns_nil() {
        let expr = Expr::List(vec![Expr::Symbol("begin".into())]);
        expect_nil(run(&expr).unwrap());
    }

    // ===== lambda =====

    #[test]
    fn lambda_creates_lambda_without_evaluating_body() {
        let expr = Expr::List(vec![
            Expr::Symbol("lambda".into()),
            Expr::List(vec![Expr::Symbol("x".into())]),
            Expr::Symbol("missing".into()),
        ]);
        expect_lambda(run(&expr).unwrap());
    }

    #[test]
    fn lambda_zero_params() {
        let expr = Expr::List(vec![
            Expr::Symbol("lambda".into()),
            Expr::Nil,
            Expr::Integer(42),
        ]);
        expect_lambda(run(&expr).unwrap());
    }

    #[test]
    fn lambda_no_body_is_error() {
        let expr = Expr::List(vec![
            Expr::Symbol("lambda".into()),
            Expr::List(vec![Expr::Symbol("x".into())]),
        ]);
        assert!(matches!(
            run(&expr),
            Err(EvalError::BadSpecialForm("lambda"))
        ));
    }

    // ===== function calls =====

    #[test]
    fn call_builtin_function() {
        let env = fresh();
        Env::define(&env, "+".into(), mk_builtin("+", add_2));
        let expr = Expr::List(vec![
            Expr::Symbol("+".into()),
            Expr::Integer(1),
            Expr::Integer(2),
        ]);
        assert_eq!(expect_int(run_in(&expr, env).unwrap()), 3);
    }

    #[test]
    fn call_closure_with_args() {
        let env = fresh();
        Env::define(&env, "+".into(), mk_builtin("+", add_2));
        let lambda = Expr::List(vec![
            Expr::Symbol("lambda".into()),
            Expr::List(vec![Expr::Symbol("x".into()), Expr::Symbol("y".into())]),
            Expr::List(vec![
                Expr::Symbol("+".into()),
                Expr::Symbol("x".into()),
                Expr::Symbol("y".into()),
            ]),
        ]);
        Env::define(&env, "add".into(), run_in(&lambda, env.clone()).unwrap());
        let call = Expr::List(vec![
            Expr::Symbol("add".into()),
            Expr::Integer(3),
            Expr::Integer(4),
        ]);
        assert_eq!(expect_int(run_in(&call, env).unwrap()), 7);
    }

    #[test]
    fn closure_wrong_arity_is_error() {
        let env = fresh();
        let lambda = Expr::List(vec![
            Expr::Symbol("lambda".into()),
            Expr::List(vec![Expr::Symbol("x".into())]),
            Expr::Symbol("x".into()),
        ]);
        Env::define(&env, "f".into(), run_in(&lambda, env.clone()).unwrap());
        let call = Expr::List(vec![
            Expr::Symbol("f".into()),
            Expr::Integer(1),
            Expr::Integer(2),
        ]);
        assert!(matches!(run_in(&call, env), Err(EvalError::ArgNotMatch)));
    }

    #[test]
    fn calling_non_callable_is_error() {
        let expr = Expr::List(vec![Expr::Integer(42), Expr::Integer(1)]);
        assert!(matches!(run(&expr), Err(EvalError::NotCallable(_))));
    }

    #[test]
    fn empty_application_is_error() {
        let expr = Expr::List(vec![]);
        assert!(matches!(run(&expr), Err(EvalError::EmptyApplication)));
    }

    // ===== lexical scope =====

    #[test]
    fn closure_captures_defining_environment() {
        // (begin
        //   (define make-adder (lambda (x) (lambda (y) (+ x y))))
        //   (define add-two (make-adder 2))
        //   (add-two 5))   ; => 7
        let env = fresh();
        Env::define(&env, "+".into(), mk_builtin("+", add_2));

        // (define make-adder (lambda (x) (lambda (y) (+ x y))))
        let inner_lambda = Expr::List(vec![
            Expr::Symbol("lambda".into()),
            Expr::List(vec![Expr::Symbol("y".into())]),
            Expr::List(vec![
                Expr::Symbol("+".into()),
                Expr::Symbol("x".into()),
                Expr::Symbol("y".into()),
            ]),
        ]);
        let make_adder_lambda = Expr::List(vec![
            Expr::Symbol("lambda".into()),
            Expr::List(vec![Expr::Symbol("x".into())]),
            inner_lambda,
        ]);
        let define_maker = Expr::List(vec![
            Expr::Symbol("define".into()),
            Expr::Symbol("make-adder".into()),
            make_adder_lambda,
        ]);
        run_in(&define_maker, env.clone()).unwrap();

        // (define add-two (make-adder 2))
        let call_maker = Expr::List(vec![Expr::Symbol("make-adder".into()), Expr::Integer(2)]);
        let define_add_two = Expr::List(vec![
            Expr::Symbol("define".into()),
            Expr::Symbol("add-two".into()),
            call_maker,
        ]);
        run_in(&define_add_two, env.clone()).unwrap();

        // (add-two 5)
        let call_add_two = Expr::List(vec![Expr::Symbol("add-two".into()), Expr::Integer(5)]);
        assert_eq!(expect_int(run_in(&call_add_two, env).unwrap()), 7);
    }
}
