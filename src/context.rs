use std::collections::HashMap;
use std::f64::consts::{PI, E};
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
fn abs(x: &[f64]) -> f64 {
    x[0].abs()
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
    fn add_func_to_ctx(&mut self, name: &str, func: fn(&[f64]) -> f64, num_args: usize) -> ();

    fn add_const_to_ctx<T>(&mut self, name: &str, val: T) -> ()
    where
        T: Into<f64>;

    fn add_var_to_ctx<T>(&mut self, name: &str, val: T) -> ()
    where
        T: Into<f64>;
} 


impl ContextLike for ContextHashMap 
{
    /// Adds a named function to the `ContextHashMap`. 
    fn add_func_to_ctx(&mut self, name: &str, func: fn(&[f64]) -> f64, num_args: usize) {
        self.insert(name.to_string(), Token::Func(num_args, func));
    }
    
    /// Adds a named constant value to the `ContextHashMap`.
    fn add_const_to_ctx<T>(&mut self, name: &str, val: T) 
    where
        T: Into<f64>
    {
        self.insert(name.to_string(), Token::Num(val.into()));
    }
    
    /// Adds a named variable to the `ContextHashMap`. 
    /// 
    /// Under the hood, the 'variable' value is stored as an 
    /// `Rc<RefCell<f64>>`. This allows other algorithms to 
    /// manipulate the variable's value.
    fn add_var_to_ctx<T>(&mut self, name: &str, val: T) 
    where
        T: Into<f64>
    {
        self.insert(name.to_string(), Token::Var(Rc::new(RefCell::new(val.into()))));
    }
}

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