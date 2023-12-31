use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

pub type ContextHashMap = HashMap<String, Token>;
#[allow(dead_code)]
#[derive(Clone)]
#[derive(Debug)]
#[derive(PartialEq)]
pub enum Token {
    LeftParenthesis,
    Comma,
    Exp,
    Mul,
    Div,
    Plus,
    Minus,
    Num(f64),
    Var(Rc<RefCell<f64>>),
    Func(usize, fn(&[f64]) -> f64),  
}

fn sin(x:  &[f64]) -> f64 {
    x[0].sin()
}
fn cos(x: &[f64]) -> f64 {
    x[0].cos()
}
fn tan(x: &[f64]) -> f64 {
    x[0].tan()
}
fn arcsin(x: &[f64]) -> f64 {
    x[0].asin()
}
fn arccos(x: &[f64]) -> f64 {
    x[0].acos()
}
fn arctan(x: &[f64]) -> f64 {
    x[0].atan()
}
fn sinh(x: &[f64]) -> f64 {
    x[0].sinh()
}
fn cosh(x: &[f64]) -> f64 {
    x[0].cosh()
}
fn tanh(x: &[f64]) -> f64 {
    x[0].tanh()
}
fn ln(x: &[f64]) -> f64 {
    x[0].ln()
}
fn log10(x: &[f64]) -> f64 {
    x[0].log10()
}
fn log(x: &[f64]) -> f64 {
    x[0].log(x[1])
}

fn conditional(args: &[f64]) -> f64 {
    let a              = args[0];
    let op             = args[1];
    let b              = args[2];
    let if_true_return = args[3];
    let else_return    = args[4];
    
    let decision = |predicate| {
    if predicate {
        if_true_return
    } else {
        else_return
    }
    };
    
    match op.round() as usize {
    1 => decision(a == b),
    2 => decision(a <= b),
    3 => decision(a >= b),
    4 => decision(a <  b),
    5 => decision(a >  b),
    _ => decision(a != b),
    }
}

pub fn add_func_to_ctx(ctx: &mut ContextHashMap, name: &str, func: fn(&[f64]) -> f64, num_args: usize) {
    ctx.insert(name.to_string(), Token::Func(num_args, func));
}

pub fn add_const_to_ctx<T>(ctx: &mut ContextHashMap, name: &str, val: T) 
where
    T: Into<f64>
{
    ctx.insert(name.to_string(), Token::Num(val.into()));
}

pub fn add_var_to_ctx<T>(ctx: &mut ContextHashMap, name: &str, val: T) 
where
    T: Into<f64>
{
    ctx.insert(name.to_string(), Token::Var(Rc::new(RefCell::new(val.into()))));
}

pub fn new_context() -> ContextHashMap {
    let mut ctx = HashMap::new();
    add_func_to_ctx(&mut ctx, "if",     conditional,  5);
    add_func_to_ctx(&mut ctx, "sin",    sin,          1);
    add_func_to_ctx(&mut ctx, "cos",    cos,          1);
    add_func_to_ctx(&mut ctx, "tan",    tan,          1);
    add_func_to_ctx(&mut ctx, "arcsin", arcsin,       1);
    add_func_to_ctx(&mut ctx, "arccos", arccos,       1);
    add_func_to_ctx(&mut ctx, "arctan", arctan,       1);
    add_func_to_ctx(&mut ctx, "sinh",   sinh,         1);
    add_func_to_ctx(&mut ctx, "cosh",   cosh,         1);
    add_func_to_ctx(&mut ctx, "tanh",   tanh,         1);
    add_func_to_ctx(&mut ctx, "ln",     ln,           1);
    add_func_to_ctx(&mut ctx, "log10",  log10,        1);
    add_func_to_ctx(&mut ctx, "log",    log,          2);
    ctx
}