use crate::{errors::EvalError, value::{self, Builtin, Value}};

pub fn arithmetic(name: &'static str) -> Builtin {
    Builtin {
        name,
        function: match name {
            "+" => add,
            "-" => sub,
            "*" => mul,
            "/" => div,
            _ => unreachable!(),
        },
    }
}

fn add(args: &[Value]) -> Result<Value, EvalError> {
    value::expect_arity(args, 2)?;
    Ok(Value::Integer(
        value::expect_integer(&args[0])?
        + value::expect_integer(&args[1])?
    ))
}

fn sub(args: &[Value]) -> Result<Value, EvalError> {
    value::expect_arity(args, 2)?;
    Ok(Value::Integer(
        value::expect_integer(&args[0])?
        - value::expect_integer(&args[1])?
    ))
}

fn mul(args: &[Value]) -> Result<Value, EvalError> {
    value::expect_arity(args, 2)?;
    Ok(Value::Integer(
        value::expect_integer(&args[0])?
        * value::expect_integer(&args[1])?
    ))
}

fn div(args: &[Value]) -> Result<Value, EvalError> {
    value::expect_arity(args, 2)?;
    let b = value::expect_integer(&args[1])?;
    if b == 0 { return Err(EvalError::DivisionByZero); }
    Ok(Value::Integer(value::expect_integer(&args[0])? / b))
}

// ===== Comparison =====

pub fn comparison(name: &'static str) -> Builtin {
    Builtin {
        name,
        function: match name {
            "=" => eq,
            "<" => lt,
            "<=" => lte,
            ">" => gt,
            ">=" => gte,
            _ => unreachable!("unknown comparison op: {}", name),
        },
    }
}

fn eq(args: &[Value]) -> Result<Value, EvalError> {
    value::expect_arity(args, 2)?;
    Ok(Value::Boolean(
        value::expect_integer(&args[0])? == value::expect_integer(&args[1])?,
    ))
}

fn lt(args: &[Value]) -> Result<Value, EvalError> {
    value::expect_arity(args, 2)?;
    Ok(Value::Boolean(
        value::expect_integer(&args[0])? < value::expect_integer(&args[1])?,
    ))
}

fn lte(args: &[Value]) -> Result<Value, EvalError> {
    value::expect_arity(args, 2)?;
    Ok(Value::Boolean(
        value::expect_integer(&args[0])? <= value::expect_integer(&args[1])?,
    ))
}

fn gt(args: &[Value]) -> Result<Value, EvalError> {
    value::expect_arity(args, 2)?;
    Ok(Value::Boolean(
        value::expect_integer(&args[0])? > value::expect_integer(&args[1])?,
    ))
}

fn gte(args: &[Value]) -> Result<Value, EvalError> {
    value::expect_arity(args, 2)?;
    Ok(Value::Boolean(
        value::expect_integer(&args[0])? >= value::expect_integer(&args[1])?,
    ))
}

// ===== Boolean =====

pub fn boolean(name: &'static str) -> Builtin {
    Builtin {
        name,
        function: match name {
            "not" => not,
            _ => unreachable!("unknown boolean op: {}", name),
        },
    }
}

fn not(args: &[Value]) -> Result<Value, EvalError> {
    value::expect_arity(args, 1)?;
    // Lisp truthiness: only #f and nil are false.
    let truthy = !matches!(&args[0], Value::Boolean(false) | Value::Nil);
    Ok(Value::Boolean(!truthy))
}

// ===== List operations =====

pub fn list_ops(name: &'static str) -> Builtin {
    Builtin {
        name,
        function: match name {
            "list" => list,
            "cons" => cons,
            "car" => car,
            "cdr" => cdr,
            "null?" => null_predicate,
            _ => unreachable!("unknown list op: {}", name),
        },
    }
}

fn list(args: &[Value]) -> Result<Value, EvalError> {
    // (list)  => nil
    // (list 1 2)  => (1 2)
    Ok(Value::list(args.to_vec()))
}

fn cons(args: &[Value]) -> Result<Value, EvalError> {
    value::expect_arity(args, 2)?;
    let head = args[0].clone();
    // expect_list accepts both Nil (=> &[]) and List (=> items).
    let tail = value::expect_list(&args[1])?;
    let mut items = Vec::with_capacity(tail.len() + 1);
    items.push(head);
    items.extend_from_slice(tail);
    Ok(Value::list(items))
}

fn car(args: &[Value]) -> Result<Value, EvalError> {
    value::expect_arity(args, 1)?;
    match &args[0] {
        // Invariant: Value::List is always non-empty, so [0] is safe.
        Value::List(items) => Ok(items[0].clone()),
        // Design §2.4: car rejects nil rather than silently returning nil.
        Value::Nil => Err(EvalError::BadArg("car: cannot take car of nil")),
        _ => Err(EvalError::BadArg("car: expected list")),
    }
}

fn cdr(args: &[Value]) -> Result<Value, EvalError> {
    value::expect_arity(args, 1)?;
    match &args[0] {
        Value::List(items) if items.len() == 1 => Ok(Value::Nil),
        Value::List(items) => Ok(Value::list(items[1..].to_vec())),
        Value::Nil => Err(EvalError::BadArg("cdr: cannot take cdr of nil")),
        _ => Err(EvalError::BadArg("cdr: expected list")),
    }
}

fn null_predicate(args: &[Value]) -> Result<Value, EvalError> {
    value::expect_arity(args, 1)?;
    Ok(Value::Boolean(matches!(args[0], Value::Nil)))
}

// ===== Output =====

pub fn output(name: &'static str) -> Builtin {
    Builtin {
        name,
        function: match name {
            "print" => print,
            _ => unreachable!("unknown output op: {}", name),
        },
    }
}

fn print(args: &[Value]) -> Result<Value, EvalError> {
    value::expect_arity(args, 1)?;
    println!("{}", args[0]);
    Ok(args[0].clone())
}