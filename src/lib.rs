/// Contains structs for passing information to the shunting yard algorithm.
pub mod context;
/// Contains error types for different errors that this crate may throw.
pub mod errors;
/// Contains root-finding algorithms for building equation-solving tools. 
pub mod newton;
/// Contains a basic shunting yard algorithm for evaluating strings as mathematical expressions.
pub mod shunting;

// use std::collections::HashMap;
// use anyhow::Result;
// use shunting::{ContextHashMap, eval_str_with_context, get_legal_variables_iter, compile_to_fn_of_hashmap};

// pub fn solve_equation(equation: &str, ctx: &mut ContextHashMap) -> Result<HashMap<String, f64>>
// {
//     let mut unknown = "";
//     for var in get_legal_variables_iter(equation)
//     {
//         if !ctx.contains_key(var) && (unknown.is_empty() || unknown == var)
//         {
//             unknown = var;
//         }
//     }
//     let func = compile_to_fn_of_hashmap(equation, &ctx);
    


//     match func(&HashMap::from([(unknown, )]))
//     {
//         Ok(_)  => ,
//         Err(_) => ,
//     }
// }