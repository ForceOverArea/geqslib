# geqslib - Grant's Equation Solver Library

Defines several functions for evaluating expressions and equations given as strings.

# Example
```rust
use std::io::Error;
use std::collections::HashMap;
use geqslib::newton::multivariate_newton_raphson;

fn f1(x: &HashMap<String, f64>) -> Result<f64, Error>
{
    Ok(x["x"] + x["y"] - 9.0)
}

fn f2(x: &HashMap<String, f64>) -> Result<f64, Error>
{
    Ok(x["x"] - x["y"] - 4.0)
}

let mut guess = HashMap::from([
    ("x".to_string(), 7.0),
    ("y".to_string(), 2.0),
]);

let soln = multivariate_newton_raphson(
    vec![f1, f2],
    &mut guess,
    0.0001,
    50,
).unwrap();

assert!(soln["x"] - 6.5 < 0.0001);
assert!(soln["y"] - 2.5 < 0.0001);
```