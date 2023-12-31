use std::collections::HashMap;
use geqslib::context::ContextHashMap;
use geqslib::shunting::{eval_str, eval_str_with_context};

#[test]
fn test_eval_str() {  
    let my_expr = "sin(-1 + 2 + 2 + 0.14)";
    let about_zero = eval_str(my_expr).unwrap().abs();

    // println!("{about_zero}");

    assert!(about_zero < 0.01)
}

#[test]
fn test_eval() {
    let ctx: ContextHashMap = HashMap::new();
    let ans = eval_str_with_context("3+4", &ctx).unwrap();
    assert_eq!(ans, 7.0);
}