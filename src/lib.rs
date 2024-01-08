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

pub fn solve_equation(equation: &str, ctx: &mut ContextHashMap, margin: f64, limit: usize) -> anyhow::Result<(String, f64)>
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
        _ => 1.0,
    };

    let f = compile_to_fn(equation, &ctx)?;

    Ok((unknowns[0].0.to_string(), newton_raphson(f, guess, margin, limit)?))
}

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
    solve_equation(equation, &mut ctx, margin, limit)
}