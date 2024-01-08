use std::collections::HashMap;
use geqslib::context::ContextHashMap;
use geqslib::shunting::{eval_str, eval_str_with_context, ContextLike};

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