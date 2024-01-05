use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use crate::errors::{ShuntingYardError, ExpressionCompilationError};
pub use crate::context::*;
use anyhow;

use lazy_static::lazy_static;
use regex::Regex;

/// Identifies and returns variables found in a geqslib-legal string.
pub fn get_legal_variables_iter(text: &str) -> impl Iterator<Item = &str> 
{
    lazy_static! 
    {
        static ref RE: Regex = Regex::new(r"(?i)[a-z][a-z0-9_]*").unwrap();
    }
    RE.find_iter(text).map(|i| i.as_str())
}

const _OPERATORS_: &str = "()^*/+-";

fn precedence(op: &str) -> i32 
{
    match op 
    {
        "^" => 4,
        "/" => 3,
        "*" => 3,
        "-" => 2,
        "+" => 2,
         _  => 1,
    }
}

fn prec_check(o1: &str, o2: &str) -> bool 
{
    let check1 = o2 != "(";
    let check2 = precedence(o2) > precedence(o1);
    let check3 = precedence(o2) == precedence(o1) && o1 != "^";
    check1 && (check2 || check3)
}

fn punctuate(expr: &str) -> String 
{
    let mut output = String::new();
    for c in expr.chars() 
    {
        if _OPERATORS_.contains(c) || c == ','
        {
            output += &format!(" {c} ");
        }
        else 
        {
            output.push(c);
        }
    }
    output.trim().to_string()
}

fn tokenize(tok: &str) -> anyhow::Result<Token> 
{
    let token = match tok 
    {
        "^" => Token::Exp,
        "/" => Token::Div,
        "*" => Token::Mul,
        "-" => Token::Minus,
        "+" => Token::Plus,
        "," => Token::Comma,
        "(" => Token::LeftParenthesis,
        maybe_num => match maybe_num.parse::<f64>() 
        {
            Ok(num) => Token::Num(num),
            Err(_) => return Err(ShuntingYardError::UnknownToken.into()),
        }
    };
    Ok(token)
}

/// Tokenizes a string, but checks `context` and
/// creates tokens for values stored there.
fn tokenize_with_context(tok: &str, context: &ContextHashMap) -> anyhow::Result<Token> 
{  
    if let Some(cnst_var_or_fn) = context.get(tok) 
    {
        let token = match cnst_var_or_fn 
        {
            Token::Func(args, func) => Token::Func(*args, *func),
            Token::Var(val) => Token::Var(Rc::clone(val)),
            Token::Num(num) => Token::Num(*num),
            _ => return Err(ShuntingYardError::ContextMutation.into()),
        };
        Ok(token)
    } 
    else 
    {
        tokenize(tok)
    }
}

pub type RPNVarMap = HashMap<String, Rc<RefCell<f64>>>;

/// See shunting yard implementation details at: 
/// https://en.wikipedia.org/wiki/Shunting_yard_algorithm
fn rpnify(expr: &str, context: &ContextHashMap) -> anyhow::Result<Vec<Token>> 
{
    let punctuated = punctuate(expr);
    let words = punctuated.split(' ').filter(|c| *c != "");

    let mut stack: Vec<&str> = Vec::new();
    let mut queue: Vec<Token> = Vec::new();
    let mut unary_minus = true; // Indicator for whether the next '-' token is a unary operator

    for word in words 
    {
        match word 
        {
            
            "," => {
                while let Some(op) = stack.pop() 
                {
                    if op != "(" {
                        queue.push(tokenize_with_context(op, context)?); // ditto the comment for the previous branch
                    } else {
                        break;
                    }
                }
                unary_minus = true;
            },

            "(" => {
                stack.push(word);
                unary_minus = true;
            },

            ")" => {
                while let Some(op) = stack.pop() 
                {
                    if op != "(" 
                    {
                        queue.push(tokenize_with_context(op, context)?);
                    } 
                    else if op == "(" 
                    {
                        break;
                    } 
                    else 
                    {
                        return Err(ShuntingYardError::UnclosedParenthesis.into())
                    }
                }
                unary_minus = false;
            },

            "^" | "/" | "*" | "+" | "-" => {
                let o1 = word;

                // if we find a minus and we're expecting a unary operator...
                if unary_minus && o1 == "-" 
                { 
                    queue.push(Token::Num(-1.0));
                    stack.push("*");
                    unary_minus = true;
                } 
                else 
                {
                    while let Some(o2) = stack.pop() 
                    {
                        if prec_check(o1, o2) 
                        {
                            queue.push(tokenize_with_context(o2, context)?);
                        } 
                        else 
                        {
                            stack.push(o2); // put the prec-check-denied element back on the stack
                            break;
                        }
                    }
                    stack.push(word);
                    unary_minus = true;
                }
            },

            other => {

                if let Ok(num) = other.parse::<f64>() 
                {
                    queue.push(Token::Num(num));
                    unary_minus = false;
                } 
                
                else if context.contains_key(other) 
                {
                    match &context[other] 
                    {
                        Token::Num(val) => {
                            queue.push(Token::Num(*val));
                            unary_minus = false;
                        },
                        Token::Var(val) => {
                            queue.push(Token::Var(Rc::clone(&val)));
                            unary_minus = false;
                        }
                        Token::Func(_, _) => {
                            stack.push(word);
                            unary_minus = true;
                        },
                        _ => return Err(ShuntingYardError::ContextMutation.into())
                    }
                }
                
                else {
                    return Err(ShuntingYardError::UnknownToken.into())
                }
            },
        }   
    }
    
    while let Some(tok) = stack.pop() 
    {
        if "()".contains(tok) 
        {
            return Err(ShuntingYardError::LeftoverToken.into())
        } 
        queue.push(tokenize_with_context(tok, context)?);
    }

    Ok(queue)
}

// TODO: Both compile functions have a sematic bug in them.
//
// If an expression contains more than 1 instance of the same variable (e.g. x^2 + x + 1)
// only the last occurrence of that variable will be added to the variable hashmap or the
// single function argument. The first Rc<RefCell<f64>> will never be mutated on calling the
// returned function.
//
// Furthermore, I'm not sure that this algorithm is valid for expressions with parenthesis, as
// variables in the parenthesis may be reordered prior to variables preceding the expression
// in text format. This would make the assumption that all variables retain the same ordering
// in RPN and string format invalid.  

/// 'Compiles' a `&str` expression to a function that takes a hashmap as an argument.
/// 
/// Under the hood, this rearranges the string expression to a token stack **once** 
/// prior to being moved to the returned closure value. The variables in the expression 
/// are added to a `HashMap` that allows the function to quickly find and mutate the 
/// values in the token stack to reduce the number of steps performed when the 
/// closure is called. 
/// 
/// # Example
/// ```
/// use std::collections::HashMap;
/// use geqslib::shunting::{
///   compile_to_fn_of_hashmap,
///   add_var_to_ctx,
///   new_context,
/// };
/// 
/// let my_expr = "x + 4";
///
/// // add variable value to context
/// let mut my_hm = new_context();
/// add_var_to_ctx(&mut my_hm, "x", 4);
/// 
/// // get a closure from the expression
/// let my_fn = compile_to_fn_of_hashmap(my_expr, &my_hm).unwrap();
/// 
/// // make an input hashmap
/// let mut my_input = HashMap::from([("x".to_string(), 8.0)]);
/// 
/// assert_eq!(my_fn(&my_input).unwrap(), 12.0);
/// ```
pub fn compile_to_fn_of_hashmap(expr: &str, context: &ContextHashMap) -> anyhow::Result<impl Fn(&HashMap<String, f64>) -> anyhow::Result<f64>> 
{
    let mut var_order = get_legal_variables_iter(expr);  
    let mut arg_hm: HashMap<String, Rc<RefCell<f64>>> = HashMap::new();

    let rpn = rpnify(expr, context)?;
    for token in &rpn 
    {
        if let Token::Var(num) = token 
        {
            arg_hm.insert(
                var_order.next().unwrap().to_string(), 
                Rc::clone(num)
            );
        }
    }

    Ok(move |x: &HashMap<String, f64>| {
        for (var, value) in x 
        {
            *arg_hm[var].borrow_mut() = *value;
        }
        eval_rpn_expression(&rpn)
    })
}

pub fn compile_to_fn(expr: &str, context: &ContextHashMap) -> anyhow::Result<impl Fn(f64) -> anyhow::Result<f64>> 
{
    let mut var_order = get_legal_variables_iter(expr);  

    let mut var: Rc<RefCell<f64>> = Rc::new(RefCell::new(f64::MIN));
    let rpn = rpnify(expr, context)?;
    for token in &rpn 
    {
        match token
        {
            Token::Var(num) => {
                var = Rc::clone(num); 
                var_order.next();
            },
            Token::Num(_) => {
                var_order.next();
            },
            _ => {},
        };
    }

    if *var.borrow() == f64::MIN
    {
        return Err(ExpressionCompilationError::NoVarsFound.into())
    }

    Ok(move |x: f64| {
        *var.borrow_mut() = x;
        eval_rpn_expression(&rpn)
    })
}

fn eval_rpn_expression(expr: &Vec<Token>) -> anyhow::Result<f64> 
{    
    let mut stack: Vec<f64> = Vec::new();
    
    for token in expr 
    {
        match token 
        {

            Token::Num(num) => stack.push(*num),
            
            Token::Var(val) => stack.push(*val.borrow()),

            Token::Func(args, func) => {

                let mut arguments: Vec<f64> = Vec::new();
                for _ in 0..*args 
                {
                    if let Some(num) = stack.pop() 
                    {
                        arguments.push(num);
                    } 
                    else 
                    {
                        return Err(ShuntingYardError::ExpectedArg.into())
                    }
                }
                stack.push(
                    func(&arguments)
                );
            },

            Token::Exp => {
                if let (Some(arg2), Some(arg1)) = (stack.pop(), stack.pop()) 
                {
                    stack.push(arg1.powf(arg2));
                } 
                else 
                {
                    return Err(ShuntingYardError::ExpectedArg.into());
                }
            },

            Token::Div => {
                if let (Some(arg2), Some(arg1)) = (stack.pop(), stack.pop()) 
                {
                    if arg2 == 0.0 
                    { 
                        return Err(ShuntingYardError::DivisionByZero.into()) 
                    }
                    stack.push(arg1 / arg2);
                } 
                else 
                {
                    return Err(ShuntingYardError::ExpectedArg.into());
                }
            },

            Token::Mul => {
                if let (Some(arg2), Some(arg1)) = (stack.pop(), stack.pop()) 
                {
                    stack.push(arg1 * arg2);
                } 
                else 
                {
                    return Err(ShuntingYardError::ExpectedArg.into());
                }
            },

            Token::Minus => {
                if let (Some(arg2), Some(arg1)) = (stack.pop(), stack.pop()) 
                {
                    stack.push(arg1 - arg2);
                } 
                else 
                {
                    return Err(ShuntingYardError::ExpectedArg.into());
                }
            },

            Token::Plus => {
                if let (Some(arg2), Some(arg1)) = (stack.pop(), stack.pop()) 
                {
                    stack.push(arg1 + arg2);
                } 
                else 
                {
                    return Err(ShuntingYardError::ExpectedArg.into());
                }
            },

            _ => {
                return Err(ShuntingYardError::LeftoverToken.into())
            },
        }
    
    }

    match stack.len() {
        1 => Ok(stack[0]),
        0 => Err(ShuntingYardError::NoTokens.into()),
        _ => {
            Err(ShuntingYardError::LeftoverToken.into())
        }
    }
}

/// Evaluates a string as a mathematical expression with built in functions including logarithms, 
/// trig functions, and even a conditional function.
/// 
/// # Example
/// ```
/// use geqslib::shunting::eval_str;
/// 
/// let my_expr = "sin(-1 + 2 + 2 + 0.14)";
/// let about_zero = eval_str(my_expr).unwrap().abs();
///
/// assert!(about_zero < 0.01);
/// ```
pub fn eval_str(expr: &str) -> anyhow::Result<f64> 
{
    eval_rpn_expression(&rpnify(expr, &new_context())?)
}

/// Evaluates a string as a mathematical expression using functions,
/// constants, and variables from a given `ContextHashMap`.
/// 
/// # Example
/// ```
/// use geqslib::shunting::{
///   eval_str_with_context,
///   add_const_to_ctx,
///   new_context,
/// };
/// 
/// let my_expr = "sin(pi)";
/// 
/// let mut my_ctx = new_context();
/// add_const_to_ctx(&mut my_ctx, "pi", 3.14);
/// 
/// let about_zero = eval_str_with_context(my_expr, &my_ctx).unwrap().abs();
///
/// assert!(about_zero < 0.01);
/// ```
pub fn eval_str_with_context(expr: &str, context: &ContextHashMap) -> anyhow::Result<f64> 
{
    eval_rpn_expression(&rpnify(expr, context)?)
}

#[test]
fn test_punctuate() 
{
    let my_expr = "3+4";
    let punctuated = punctuate(my_expr);
    assert_eq!(punctuated, "3 + 4");

    let tokens = Vec::from_iter(punctuated.split(' '));
    assert_eq!(
        tokens,
        vec!["3", "+", "4"]
    )
}

#[test]
fn test_rpnify() 
{
    let ctx: ContextHashMap = HashMap::new();
    let rpn = rpnify("3+4", &ctx).unwrap();
    assert_eq!(rpn, vec![Token::Num(3.0), Token::Num(4.0), Token::Plus])
}

#[test]
fn test_unary_minus() 
{
    let ctx: ContextHashMap = new_context();
    let rpn = rpnify("sin(-1 + 2 + 2 + 0.14)", &ctx).unwrap();

    assert_eq!(rpn[0], Token::Num(-1.0));
}