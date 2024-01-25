use std::collections::HashMap;
use std::f64::consts::{PI, E};
use std::rc::Rc;
use std::cell::RefCell;

use crate::variable::Variable;

/// A specific kind of `HashMap` that allows functions in geqslib
/// to understand function, variable, and constant values in string-formatted
/// expressions and equations.
pub type ContextHashMap = HashMap<String, Token>;

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
    Var(Rc<RefCell<Variable>>),
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
fn abs(x: &[f64]) -> f64 {
    x[0].abs()
}

fn conditional(args: &[f64]) -> f64 {
    let a              = args[4];
    let op             = args[3];
    let b              = args[2];
    let if_true_return = args[1];
    let else_return    = args[0];
    
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

/// A module for sealing the `ContextLike` trait.
pub (crate) mod private
{
    use super::ContextHashMap;
    pub trait Sealed {}
    impl Sealed for ContextHashMap {}
}

/// Provides extra methods for `ContextHashMap`.
pub trait ContextLike: private::Sealed
{
    fn add_func_to_ctx(&mut self, name: &str, func: fn(&[f64]) -> f64, num_args: usize);

    fn add_const_to_ctx<T>(&mut self, name: &str, val: T)
    where
        T: Into<f64> + Copy;

    fn add_var_to_ctx<T>(&mut self, name: &str, val: T)
    where 
        T: Into<f64> + Copy;

    fn add_var_with_domain_to_ctx<T>(&mut self, name: &str, val: T, min: T, max: T)
    where
        T: Into<f64> + Copy;
} 

/// Provides extra methods for the `ContextHashMap` type.
impl ContextLike for ContextHashMap 
{
    /// Adds a named function to the `ContextHashMap`. 
    fn add_func_to_ctx(&mut self, name: &str, func: fn(&[f64]) -> f64, num_args: usize) {
        self.insert(name.to_owned(), Token::Func(num_args, func));
    }
    
    /// Adds a named constant value to the `ContextHashMap`.
    fn add_const_to_ctx<T>(&mut self, name: &str, val: T) 
    where
        T: Into<f64> + Copy 
    {
        self.insert(name.to_owned(), Token::Num(val.into()));
    }
    
    /// Adds a variable to the `ContextHashMap` with an infinite domain.
    fn add_var_to_ctx<T>(&mut self, name: &str, val: T)
    where 
        T: Into<f64> + Copy 
    {
        self.add_var_with_domain_to_ctx(name, val.into(), f64::NEG_INFINITY, f64::INFINITY);
    }

    /// Adds a named variable to the `ContextHashMap` with a specified domain.
    fn add_var_with_domain_to_ctx<T>(&mut self, name: &str, val: T, min: T, max: T) 
    where
        T: Into<f64> + Copy
    {
        self.insert(name.to_owned(), Token::Var(Rc::new(RefCell::new(Variable::new(val, min, max)))));
    }
}

/// Initializes a new `ContextHashMap` with basic trig, log, conditional, and absolute value
/// functions as well as pre-defined constants for pi and Euler's number.
/// 
/// # Example
/// ```
/// use geqslib::shunting::{new_context, Token};
/// use std::f64::consts::PI;
/// 
/// let ctx = new_context();
/// 
/// if let Token::Num(x) = ctx["pi"]
/// {
///     assert_eq!(PI, x);
/// }
/// ```
pub fn new_context() -> ContextHashMap {
    let mut ctx = HashMap::new();
    ctx.add_func_to_ctx("if",     conditional, 5);
    
    ctx.add_func_to_ctx("sin",    sin,         1);
    ctx.add_func_to_ctx("cos",    cos,         1);
    ctx.add_func_to_ctx("tan",    tan,         1);
    
    ctx.add_func_to_ctx("arcsin", arcsin,      1);
    ctx.add_func_to_ctx("arccos", arccos,      1);
    ctx.add_func_to_ctx("arctan", arctan,      1);
    
    ctx.add_func_to_ctx("sinh",   sinh,        1);
    ctx.add_func_to_ctx("cosh",   cosh,        1);
    ctx.add_func_to_ctx("tanh",   tanh,        1);
    
    ctx.add_func_to_ctx("ln",     ln,          1);
    ctx.add_func_to_ctx("log10",  log10,       1);
    ctx.add_func_to_ctx("log",    log,         2);
    
    ctx.add_func_to_ctx("abs",    abs,         1);
    
    ctx.add_const_to_ctx("pi",                PI);
    ctx.add_const_to_ctx("e",                  E);
    
    ctx
}