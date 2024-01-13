use std::collections::HashMap;
use crate::shunting::{get_legal_variables_iter, ContextHashMap, Token};
use crate::compile_equation_to_fn_of_hashmap;

pub enum ConstrainResult
{
    WillConstrain,
    WillNotConstrain,
    WillOverConstrain,
}

pub struct SystemBuilder<'a>
{
    context: &'a ContextHashMap,
    system_vars: Vec<&'a str>,
    system_equations: Vec<Box<dyn Fn(&HashMap<String, f64>) -> anyhow::Result<f64>>>,
}
impl <'a> SystemBuilder<'a>
{
    pub fn new(equation: &'a str, ctx: &'a ContextHashMap) -> anyhow::Result<SystemBuilder<'a>>
    {
        Ok(SystemBuilder
        {
            context: ctx,
            system_vars: get_equation_unknowns(equation, ctx).collect(),
            system_equations: vec![
                Box::new(compile_equation_to_fn_of_hashmap(equation, ctx)?)
            ],
        })
    }

    pub fn try_add_equation(&mut self, equation: &'a str) -> anyhow::Result<ConstrainResult> 
    {
        let mut num_unknowns = 0;
        let mut maybe_new_var = None;
        let sys_equations = self.system_equations.len();
        let sys_unknowns = self.system_vars.len();

        for unknown in get_equation_unknowns(equation, self.context)
            .filter(|x| self.system_vars.contains(&x))
        {
            num_unknowns += 1;
            maybe_new_var = Some(unknown);
        }

        if  num_unknowns > 1 
        {
            // Return early if adding the equation will not gainfully constrain the system
            return Ok(ConstrainResult::WillNotConstrain);
        }
        else if (sys_equations + 1) > (sys_unknowns + num_unknowns) 
        {
            // Return early if the system will be over-constrained
            return Ok(ConstrainResult::WillOverConstrain);
        }

        // Add the equation to the system
        self.system_equations.push(
            Box::new(compile_equation_to_fn_of_hashmap(equation, self.context)?)
        );

        // Add possible newly-found variable to the system
        if let Some(new_var) = maybe_new_var
        {
            self.system_vars.push(new_var);
        }

        // Indicate that addition was successful
        Ok(ConstrainResult::WillConstrain)
    }

    fn check_constraint(&self) -> bool
    {
        self.system_equations.len() == self.system_vars.len()
    }

    pub fn try_constraining_with(&mut self, equations: Vec<&'a str>) -> anyhow::Result<()>
    {
        let mut still_learning = true;
        while still_learning && !self.check_constraint()
        {
            still_learning = false;
            for equation in &equations
            {
                match self.try_add_equation(equation)
                {
                    Ok(ConstrainResult::WillNotConstrain) => {}, 
                    Ok(ConstrainResult::WillConstrain) => {
                        still_learning = true;
                    },
                    Ok(ConstrainResult::WillOverConstrain) => {
                        break;
                    },
                    Err(e) => {
                        return Err(e.into());
                    }
                }
            }
        }
        Ok(())
    }
}

pub fn get_equation_unknowns<'a>(equation: &'a str, ctx: &'a ContextHashMap) -> impl Iterator<Item = &'a str>
{
    get_legal_variables_iter(equation)
        .filter(|&x| {
            match ctx[x] 
            {
                Token::Var(_) => true,
                _ => false,
            }
        })
}
