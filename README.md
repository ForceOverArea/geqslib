# geqslib - Grant's Equation Solver Library

Geqslib defines several functions for evaluating expressions and solving equations given as strings. The crate provides both single and multiple-variable implementations of the Newton-Raphson root-finding algorithm and provides them for use in other scenarios where expressions may be better represented by a closure than a string.

Most of the provided functions, however, are focused on evaluating expressions or equations as strings:

# Example
```rust
use geqslib::solve_equation_from_str;

let (var, soln) = solve_equation_from_str("x + 4 = 12", 0.0001, 10).unwrap();

assert_eq!(var, "x");
assert!((soln - 8.0).abs() < 0.001);
```

Geqslib also provides a `SystemBuilder` struct for properly constraining a system of equations for later solving.

# Example
```rust
use geqslib::system::{System, SystemBuilder};
use geqslib::shunting::new_context;
let mut ctx = new_context();

// Build up the system:
let mut builder = SystemBuilder::new("x + y = 9", ctx).unwrap();
builder.try_constrain_with("x - y = 4");

// Convert to a constrained system
let mut sys = builder
    .build_system()
    .expect("Failed to constrain system...");

// Specify guess value and domain for variables if desired
sys.specify_variable("x", 6.5, 0.0, 7.0);

// Specify tolerance and iteration limit, then solve!
let soln = sys.solve(0.0001, 10)
    .expect("Failed to find a solution...");

// Solution is x = 6.5, y = 2.5
assert!((6.5 - soln["x"]).abs() < 0.001);
assert!((2.5 - soln["y"]).abs() < 0.001);
```