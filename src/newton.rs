use std::{error::Error, collections::HashMap, iter::zip, borrow::BorrowMut};

use gmatlib::{Matrix, row_vec, col_vec};
use crate::errors::NewtonRaphsonSolverError;

const _DX_: f64 = 0.0001; 

// TODO: untangle the mess caused by having internally-produced closures return an externally defined error type

/// A basic implementation of the 1-D newton-raphson method.
/// This function allows the caller to choose an initial guess value,
/// a margin of error, and a maximum number of iterations prior to 
/// returning a value. 
/// 
/// # Example
/// ```
/// use std::io::Error;
/// use geqslib::newton::newton_raphson;
/// 
/// fn x_squared(x: f64) -> Result<f64, Error>
/// {
///     Ok(x * x)
/// }
/// 
/// let y = newton_raphson(x_squared, 1.0, 0.0001, 10).unwrap();
/// 
/// assert!(y < 0.0001); // solution is APPROXIMATE. In this case, very close to 0.
/// ```
pub fn newton_raphson<E>(f: impl Fn(f64) -> Result<f64, E>, guess: f64, margin: f64, limit: usize) -> anyhow::Result<f64>
where E: Error + Send + Sync + 'static
{
    // Catch illegal margin of error
    if margin <= 0.0
    {
        return Err(NewtonRaphsonSolverError::NegativeMargin.into());
    }

    // Allow user to manually prevent stack overflow
    if limit == 0
    {
        return Err(NewtonRaphsonSolverError::ReachedIterationLimit.into());
    }

    // Check if we are sufficiently close to the solution:
    let y = f(guess)?;
    if y.abs() <= margin
    {
        return Ok(guess); // ...if so, exit early
    }

    // ...if not, calculate next iteration
    let y_prime = (f(guess + _DX_)? - y) / _DX_;
    let next_guess = guess - y / y_prime;

    newton_raphson(f, next_guess, margin, limit - 1)
}


pub fn multivariate_newton_raphson<E>(f: Vec<impl Fn(&HashMap<String, f64>) -> Result<f64, E>>, guess: &mut HashMap<String, f64>, margin: f64, limit: usize) -> anyhow::Result<&mut HashMap<String, f64>>
where E: Error + Send + Sync + 'static
{
    // Catch illegal margin of error
    if margin <= 0.0
    {
        return Err(NewtonRaphsonSolverError::NegativeMargin.into());
    }

    // Allow user to manually prevent stack overflow
    if limit == 0
    {
        return Err(NewtonRaphsonSolverError::ReachedIterationLimit.into());
    }

    // Establish system size
    let n = f.len();
    if guess.len() != n
    {
        return Err(NewtonRaphsonSolverError::ImproperlyConstrainedSystem.into());
    }

    // Calculate current error
    let mut y = vec![0.0; 3];
    for i in 0..n
    {
        y[i] = f[i](guess)?;
    }

    // Return guess if it is close enough to solution
    if y.iter()
        .map(|v| v.abs())
        .sum::<f64>() < margin
    {
        return Ok(guess);
    }

    // Build jacobian w/ F(X) values... we will mutate them to F'(X) later
    let mut elements = vec![];
    for i in 0..n 
    {
        let row = &mut vec![f[i](guess)?; n];
        elements.append(row);
    }
    let mut jacobian = Matrix::from_vec(n, elements)?; // <- should this be a panic on failure?
    let vars = Vec::from_iter(guess.keys().map(|x| x.to_string()));

    // Correct jacobian values and invert
    for j in 0..n
    {
        if let Some(v) = guess.get_mut(&vars[j])
        {
            *v += _DX_;
        } 
        for i in 0..n
        {
            // mutate values to partial derivatives
            jacobian[(i, j)] = (f[i](&guess)? - jacobian[(i, j)]) / _DX_;
        }
        if let Some(v) = guess.get_mut(&vars[j])
        {
            *v -= _DX_;
        } 
    }
    jacobian.try_inplace_invert()?;

    // Build next guess vector
    let mut next_vals: Vec<f64> = (jacobian * Matrix::from_col_vec(y)).into();
    for var in vars
    {
        if let (Some(t), Some(u)) = (guess.get_mut(&var), next_vals.pop())
        {
            *t = u;
        }
    }

    // COMPUTER, ENHANCE!
    multivariate_newton_raphson(f, guess, margin, limit)
}