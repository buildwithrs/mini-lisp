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