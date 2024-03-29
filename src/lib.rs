/// Contains functions for checking whether systems or equations are properly constrained for solving.
pub mod system;
/// Contains structs for passing information to the shunting yard algorithm. This is re-exported by the `shunting` module.
mod context;
/// Contains error types for different errors that this crate may throw.
pub mod errors;
/// Contains `extern "C"` function definitions for linking this library
/// against projects in different languages. Not intended for use in 
/// other Rust projects.
pub mod ffi;
/// Contains root-finding algorithms for building equation-solving tools. 
pub mod newton;
/// Contains a basic shunting yard algorithm for evaluating strings as mathematical expressions.
pub mod shunting;
/// Contains the `Variable` type for numbers that exist on a user-specified domain.
pub mod variable;

use std::collections::{HashMap, HashSet};

use context::ContextLike;
use errors::EquationSolverError;
use newton::newton_raphson;
use shunting::{ContextHashMap, compile_to_fn, compile_to_fn_of_hashmap, get_legal_variables_iter, new_context};
use system::get_equation_unknowns;

/// An internal function for formatting a single-unknown equation to an expression prior to tokenization 
pub (in crate) fn compile_equation_to_fn(equation: &str, ctx: &ContextHashMap) -> anyhow::Result<impl Fn(f64) -> anyhow::Result<f64>>
{
    // Ensure that we're solving just one equation
    let sides: Vec<&str> = equation.split('=').collect();
    match sides.len()
    {
        1 => return Err(EquationSolverError::FoundExpression.into()),
        2 => (),
        _ => return Err(EquationSolverError::FoundMultipleEquations.into()),
    }
    
    compile_to_fn(&format!("{} - ({})", sides[0], sides[1]), ctx)
}

/// An internal function for formatting an equation to an expression prior to tokenization 
pub (in crate) fn compile_equation_to_fn_of_hashmap(equation: &str, ctx: &mut ContextHashMap) -> anyhow::Result<impl Fn(&HashMap<String, f64>) -> anyhow::Result<f64>>
{
    // Ensure that we're solving just one equation
    let sides: Vec<&str> = equation.split('=').collect();
    match sides.len()
    {
        1 => return Err(EquationSolverError::FoundExpression.into()),
        2 => (),
        _ => return Err(EquationSolverError::FoundMultipleEquations.into()),
    }

    // Get the unknowns. Need to be owned to mutate ctx
    let unknowns: Vec<String> = get_equation_unknowns(equation, ctx)
        .map(|x| x.to_owned())
        .collect();

    // Add a default guess value of 1 for all unspecified vars
    for var in unknowns
    {
        ctx.add_var_with_domain_to_ctx(&var, 1.0, f64::NEG_INFINITY, f64::INFINITY);
    }

    compile_to_fn_of_hashmap(&format!("{} - ({})", sides[0], sides[1]), ctx)
}

/// Solves an equation given as a string for the SINGLE
/// unknown that is inferred based on the context and the given equation
/// string. The given context must contain all known symbols in the 
/// equation but NOT the variable that is to be solved for. 
/// E.g. the context for `"x + sin(y) = 9"` must define a value for `"y"` 
/// and `"sin"`, but NO value for `"x"` if `"x"` is the variable to be solved for.
/// 
/// # Example
/// ```
/// use geqslib::solve_equation_with_context;
/// use geqslib::shunting::new_context;
/// 
/// let mut ctx = new_context();
/// 
/// let (var, soln) = solve_equation_with_context("x + 4 = 12", &mut ctx, 1.0, f64::NEG_INFINITY, f64::INFINITY, 0.0001, 10)
///     .expect("failed to find a solution");
/// 
/// assert_eq!(var, "x");
/// assert!((soln - 8.0).abs() < 0.001);
/// ```
pub fn solve_equation_with_context(equation: &str, ctx: &mut ContextHashMap, guess: f64, min: f64, max: f64, margin: f64, limit: usize) -> anyhow::Result<(String, f64)>
{
    // Check constraints
    let unknowns: Vec<&str> = get_legal_variables_iter(equation)
        .filter(|&x| !ctx.contains_key(x))
        .collect::<HashSet<&str>>()
        .into_iter()
        .collect();

    // Exit early if equation is improperly constrained
    if unknowns.len() != 1
    {
        return Err(EquationSolverError::SingleUnknownNotFound.into());
    }
    
    ctx.add_var_with_domain_to_ctx(unknowns[0], guess, min, max);
    let f = compile_equation_to_fn(equation, ctx)?;

    Ok((unknowns[0].to_owned(), newton_raphson(f, 1.0, margin, limit)?))
}

/// Solves an equation given as a string for a SINGLE unknown variable.
/// This function infers the unknown variable from the given expression, 
/// using a new default `ContextHashMap` to account for common constants
/// and functions.
/// 
/// Intial guess values are set to 1.0f64 for the unknown variable if it 
/// can be inferred from the equation and the unknown variable is assumed to
/// exist on \[`f64::NEG_INFINITY`, `f64::INFINITY`\].
/// 
/// # Example
/// ```
/// use geqslib::solve_equation_from_str;
/// 
/// let (var, soln) = solve_equation_from_str("x + 4 = 12", 0.0001, 10).unwrap();
/// 
/// assert_eq!(var, "x");
/// assert!((soln - 8.0).abs() < 0.001);
/// ```
pub fn solve_equation_from_str(equation: &str, margin: f64, limit: usize) -> anyhow::Result<(String, f64)>
{
    let mut ctx = new_context();
    solve_equation_with_context(equation, &mut ctx, 1.0, f64::NEG_INFINITY, f64::INFINITY, margin, limit)
}
