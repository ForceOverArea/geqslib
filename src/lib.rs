/// Contains structs for passing information to the shunting yard algorithm.
pub mod context;
/// Contains error types for different errors that this crate may throw.
pub mod errors;
/// Contains root-finding algorithms for building equation-solving tools. 
pub mod newton;
/// Contains a basic shunting yard algorithm for evaluating strings as mathematical expressions.
pub mod shunting;

use anyhow::Ok;
use context::{ContextLike, Token};
use errors::EquationSolverError;
use newton::newton_raphson;
use shunting::{ContextHashMap, compile_to_fn, get_legal_variables_iter, new_context};

/// Solves an equation given as a string for the SINGLE
/// `Token::Var` in `ctx`. If a different number of variables
/// are given, no solution will be attempted.
/// 
/// Guess values for the unknown variable are pulled from the
/// value that it was initialized to in the `ContextHashMap`.
/// 
/// # Example
/// ```
/// use geqslib::solve_equation_with_context;
/// use geqslib::context::{ContextHashMap, ContextLike};
/// 
/// let mut ctx = ContextHashMap::new();
/// ctx.add_var_to_ctx("x", 7.0);
/// 
/// let (var, soln) = solve_equation_with_context("x + 4 = 12", &mut ctx, 0.0001, 10).unwrap();
/// 
/// assert_eq!(var, "x");
/// assert!((soln - 8.0).abs() < 0.001);
/// ```
pub fn solve_equation_with_context(equation: &str, ctx: &mut ContextHashMap, margin: f64, limit: usize) -> anyhow::Result<(String, f64)>
{
    // Check constraints
    let unknowns: Vec<(&String, &Token)> = ctx.iter()
        .filter(|x| {
            match x
            {
                (_, Token::Var(_)) => true, 
                _ => false, 
            }
        })
        .collect();

    // Exit early if equation is improperly constrained
    if unknowns.len() != 1
    {
        return Err(EquationSolverError::SingleUnknownNotFound.into());
    }
    
    let guess = match unknowns[0].1
    {
        Token::Var(r) => *r.borrow(),
        _ => 1.0, // This branch should never be used. If it is, it will just set the initial guess to 1
    };

    // Ensure that we're solving just one equation
    let sides: Vec<&str> = equation.split('=').collect();
    match sides.len()
    {
        1 => return Err(EquationSolverError::FoundExpression.into()),
        2 => (),
        _ => return Err(EquationSolverError::FoundMultipleEquations.into()),
    }

    let f = compile_to_fn(&format!("{} - ({})", sides[0], sides[1]), &ctx)?;

    Ok((unknowns[0].0.to_string(), newton_raphson(f, guess, margin, limit)?))
}

/// Solves an equation given as a string for a SINGLE unknown variable.
/// This function infers the unknown variable from the given expression, 
/// meaning that it cannot contain more than 1 unknown variable and no
/// constants or unknown function names.
/// 
/// Intial guess values are set to 1.0f64 for the unknown variable if it 
/// can be inferred from the equation.
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
    let unknowns: Vec<&str> = get_legal_variables_iter(equation)
        .filter(|&x| !ctx.contains_key(x))
        .collect();

    if unknowns.len() != 1
    {
        return Err(EquationSolverError::SingleUnknownNotFound.into())
    }

    ctx.add_var_to_ctx(unknowns[0], 1.0);
    solve_equation_with_context(equation, &mut ctx, margin, limit)
}