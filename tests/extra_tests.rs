use std::collections::HashMap;
use geqslib::shunting::{new_context, ContextHashMap};
use geqslib::shunting::{eval_str, eval_str_with_context, ContextLike};
use geqslib::solve_equation_with_context;

#[test]
fn test_eval_str() 
{  
    let my_expr = "sin(-1 + 2 + 2 + 0.14)";
    let about_zero = eval_str(my_expr).unwrap().abs();

    // println!("{about_zero}");

    assert!(about_zero < 0.01)
}

#[test]
fn test_eval() 
{
    let mut ctx: ContextHashMap = HashMap::new();
    ctx.add_const_to_ctx("x", 0.0);
    let ans = eval_str_with_context("3 + 4 + x", &ctx).unwrap();
    assert_eq!(ans, 7.0);
}

#[test]
fn ensure_that_single_unknown_solver_can_solve_equation_with_if_statement()
{
    let mut ctx = new_context();
    ctx.add_const_to_ctx("x", 6.5);
    ctx.add_const_to_ctx("y", 2.5);

    let soln = solve_equation_with_context(
        "if(x,4.0,y,i-(1),i-(-1)) = 0", 
        &mut ctx, 
        -1.0, 
        f64::NEG_INFINITY, 
        f64::INFINITY, 
        0.0001, 
        100
    ).unwrap();

    assert_eq!(soln.0, "i".to_owned());
    assert!(soln.1 - 1.0 < 0.001);
}