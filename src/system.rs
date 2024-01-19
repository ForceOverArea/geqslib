use std::collections::HashMap;
use crate::newton::multivariate_newton_raphson;
use crate::shunting::{get_legal_variables_iter, ContextHashMap, Token};
use crate::compile_equation_to_fn_of_hashmap;

/// An enum for indicating why an equation could or could not be added
/// to a system of equations in a `SystemBuilder`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ConstrainResult
{
    /// Indicates that the equation added at most 1 unknown variable to 
    /// the system of equations. 
    WillConstrain,

    /// Indicates that the equation added more than 1 unknown variable
    /// to the system of equations.
    WillNotConstrain,

    /// Indicates that the equation given will over-constrain the system,
    /// giving it more equations than degrees of freedom. 
    WillOverConstrain,
}

/// An object for building up a system of equations and ensuring that it is 
/// fully constrained prior to attempting to solve it.
pub struct SystemBuilder<'a>
{
    context: &'a mut ContextHashMap,
    system_vars: Vec<String>,
    system_equations: Vec<Box<dyn Fn(&HashMap<String, f64>) -> anyhow::Result<f64>>>,
}
impl <'a> SystemBuilder<'a>
{
    /// Constructs a new `SystemBuilder` instance.
    /// 
    /// # Example
    /// ```
    /// use geqslib::system::SystemBuilder;
    /// use geqslib::shunting::new_context;
    /// 
    /// let mut ctx = new_context();
    /// 
    /// let my_sys = SystemBuilder::new("x + y = 4", &mut ctx)
    ///     .expect("failed to build system!");
    /// ```
    pub fn new(equation: &'a str, ctx: &'a mut ContextHashMap) -> anyhow::Result<SystemBuilder<'a>>
    {
        let system_vars = get_equation_unknowns(equation, ctx)
            .map(|x| x.to_owned())
            .collect();

        let starting_eqn = Box::new(compile_equation_to_fn_of_hashmap(equation, ctx)?);

        Ok(SystemBuilder
        {
            context: ctx,
            system_vars,
            system_equations: vec![starting_eqn],
        })
    }

    /// Gives a reference to the unknown variables in the system.
    /// 
    /// # Example
    /// ```
    /// use geqslib::system::SystemBuilder;
    /// use geqslib::shunting::new_context;
    /// 
    /// let mut ctx = new_context();
    /// 
    /// let my_sys = SystemBuilder::new("x + y = 4", &mut ctx)
    ///     .expect("failed to build system!");
    /// 
    /// assert_eq!(2, my_sys.get_vars().len());
    /// assert!(my_sys.get_vars().contains(&"x".to_owned()));
    /// assert!(my_sys.get_vars().contains(&"y".to_owned()));
    /// ```
    pub fn get_vars(&self) -> &Vec<String>
    {
        &self.system_vars
    }

    /// Attempts to constrain the system of equations by adding an equation.
    /// If the equation adds at most 1 unknown variable, it will be added to
    /// the system and an `Ok(ConstrainResult::WillConstrain)` will be returned.
    /// If the given equation will over-constrain the system, then an 
    /// `Ok(ConstrainResult::WillOverConstrain)` is returned. If neither of 
    /// these happen, but no errors occur during the 
    /// 
    /// # Equation
    /// ```
    /// use geqslib::system::{ConstrainResult, SystemBuilder};
    /// use geqslib::shunting::{ContextHashMap, ContextLike};
    /// 
    /// let mut ctx = ContextHashMap::new();
    /// 
    /// let mut my_sys = SystemBuilder::new("x + y = 9", &mut ctx)
    ///     .expect("failed to build system!");
    /// 
    /// // Too many unknowns to be useful to system.
    /// let res = my_sys.try_constrain_with("i - j = 4").unwrap();
    /// assert_eq!(res, ConstrainResult::WillNotConstrain);
    /// 
    /// // Adds 0 unknowns and 1 equation. Will not over-constrain
    /// // the system, and this will not add too many unknowns.
    /// let res = my_sys.try_constrain_with("x - y = 4").unwrap();
    /// assert_eq!(res, ConstrainResult::WillConstrain);
    /// 
    /// // System is already properly constrained. This will not
    /// // be useful to add.
    /// let res = my_sys.try_constrain_with("x - y = 4").unwrap();
    /// assert_eq!(res, ConstrainResult::WillOverConstrain);
    /// ```
    pub fn try_constrain_with(&mut self, equation: &'a str) -> anyhow::Result<ConstrainResult> 
    {
        let mut num_unknowns = 0;
        let mut maybe_new_var = None;
        let sys_equations = self.system_equations.len();
        let sys_unknowns = self.system_vars.len();

        let unknowns: Vec<String> = get_equation_unknowns(equation, self.context)
            .filter(|&x| !self.system_vars.contains(&x.to_owned()))
            .map(|x| x.to_owned())
            .collect();

        for unknown in unknowns
        {
            num_unknowns += 1;
            maybe_new_var = Some(unknown);
        }

        if  num_unknowns > 1 
        {
            // Return early if adding the equation will not gainfully constrain the system
            return Ok(ConstrainResult::WillNotConstrain);
        }
        else if (sys_equations + 1) > (sys_unknowns + 1) || self.is_fully_constrained()
        {
            // Return early if the system will be over-constrained or 
            // no longer fully constrained.
            return Ok(ConstrainResult::WillOverConstrain);
        }

        // Add the equation to the system
        self.system_equations.push(
            Box::new(compile_equation_to_fn_of_hashmap(equation, &mut self.context)?)
        );

        // Add possible newly-found variable to the system
        if let Some(new_var) = maybe_new_var
        {
            self.system_vars.push(new_var);
        }

        // Indicate that addition was successful
        Ok(ConstrainResult::WillConstrain)
    }

    /// Returns a boolean value indicating whether a system is 
    /// fully constrained. I.e. the number of equations is equal to
    /// the number of degrees of freedom.
    /// 
    /// # Example
    /// ```
    /// use geqslib::system::{ConstrainResult, SystemBuilder};
    /// use geqslib::shunting::{ContextHashMap, ContextLike};
    /// 
    /// let mut ctx = ContextHashMap::new();
    /// 
    /// let mut my_sys = SystemBuilder::new("x + y = 9", &mut ctx).unwrap();
    /// 
    /// assert!(!my_sys.is_fully_constrained());
    /// 
    /// my_sys.try_constrain_with("x - y = 4")
    ///     .expect("failed to constrain more!");
    /// 
    /// // Now that we have 2 equations and 2 unknowns, the system is
    /// // constrained and can be solved.
    /// assert!(my_sys.is_fully_constrained());
    /// ```
    pub fn is_fully_constrained(&self) -> bool
    {
        self.system_equations.len() == self.system_vars.len()
    }

    /// Attempts to fully constrain a system using a given `Vec`
    /// of equations.
    /// 
    /// # Example
    /// ```
    /// use geqslib::system::{ConstrainResult, SystemBuilder};
    /// use geqslib::shunting::{ContextHashMap, ContextLike};
    /// 
    /// let mut ctx = ContextHashMap::new();
    /// ctx.add_var_to_ctx("x", 1.0);
    /// ctx.add_var_to_ctx("y", 1.0);
    /// ctx.add_var_to_ctx("z", 1.0);
    /// 
    /// let mut my_sys = SystemBuilder::new("x + y + z = 9", &mut ctx).unwrap();
    /// 
    /// my_sys.try_fully_constrain_with(vec![
    ///     "(4 * x) + (5 * y) + (6 * z) = 7", 
    ///     "(8 * x) + (9 * y) - (10 * z) = 11"])
    ///     .expect("failed to constrain system!");
    /// ```
    pub fn try_fully_constrain_with(&mut self, equations: Vec<&'a str>) -> anyhow::Result<bool>
    {
        let mut still_learning = true;
        while still_learning && !self.is_fully_constrained()
        {
            still_learning = false;
            for equation in &equations
            {
                match self.try_constrain_with(equation)
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
        Ok(self.is_fully_constrained())
    }

    /// Consumes `self` in order to produce a `System` object, representing 
    /// a constrained system of equations.
    pub fn get_system(self) -> Option<System<'a>>
    {
        if self.is_fully_constrained()
        {
            return Some(System {
                context: self.context,
                system_vars: self.system_vars,
                system_equations: self.system_equations,
            });
        }
        
        None
    }
}

/// A Constrained system of equations that can either have more information specified about its
/// variables or just be solved after construction.
/// 
/// This object can only be built using a `SystemBuilder` object.
pub struct System<'a>
{
    context: &'a mut ContextHashMap,
    system_vars: Vec<String>,
    system_equations: Vec<Box<dyn Fn(&HashMap<String, f64>) -> anyhow::Result<f64>>>,
}
impl <'a> System<'a>
{
    /// Traps the value of the given variable between `min` and `max`.
    /// 
    /// # Example
    /// ```
    /// use geqslib::system::{System, SystemBuilder};
    /// use geqslib::shunting::new_context;
    ///  
    /// let mut ctx = new_context();
    /// 
    /// let mut builder = SystemBuilder::new("x + y = 9", &mut ctx)
    ///     .expect("Failed to create a system...");
    /// builder.try_constrain_with("x - y = 4");
    /// 
    /// let mut sys = builder
    ///     .get_system()
    ///     .unwrap();
    /// 
    /// sys.specify_domain("x", 0.0, 7.0);
    /// ```
    pub fn specify_domain(&mut self, var: &str, min: f64, max: f64) -> bool
    {
        if !self.system_vars.contains(&var.into())
        {
            return false;
        }

        match &self.context[var]
        {
            Token::Var(value) => {
                (*value.borrow_mut()).min = min;
                (*value.borrow_mut()).max = max;
            },
            _ => return false,
        };

        true
    }

    /// Sets a guess value for the given variable.
    /// 
    /// # Example
    /// ```
    /// use geqslib::system::{System, SystemBuilder};
    /// use geqslib::shunting::new_context;
    ///  
    /// let mut ctx = new_context();
    /// 
    /// let mut builder = SystemBuilder::new("x + y = 9", &mut ctx)
    ///     .expect("Failed to create a system...");
    /// builder.try_constrain_with("x - y = 4");
    /// 
    /// let mut sys = builder
    ///     .get_system()
    ///     .expect("Failed to constrain system...");
    /// 
    /// sys.specify_guess_value("x", 6.5);
    /// ```
    pub fn specify_guess_value(&mut self, var: &str, guess: f64) -> bool
    {
        if !self.system_vars.contains(&var.into())
        {
            return false;
        }

        match &self.context[var]
        {
            Token::Var(value) => {
                (*value.borrow_mut()).set(guess)
            },
            _ => return false,
        };

        true
    }

    /// Tries to solve the system of equations to within the radius `margin` 
    /// of the actual solution in `limit` iterations. 
    /// 
    /// # Example
    /// ```
    /// use geqslib::system::{System, SystemBuilder};
    /// use geqslib::shunting::new_context;
    /// 
    /// let mut ctx = new_context();
    /// 
    /// let mut builder = SystemBuilder::new("x + y = 9", &mut ctx)
    ///     .expect("Failed to create a system...");
    /// builder.try_constrain_with("x - y = 4");
    /// 
    /// let mut sys = builder
    ///     .get_system()
    ///     .expect("Failed to constrain system...");
    /// 
    /// let soln = sys.solve(0.0001, 10)
    ///     .expect("Failed to find a solution...");
    /// 
    /// // Solution is x = 6.5, y = 2.5
    /// assert!((6.5 - soln["x"]).abs() < 0.001);
    /// assert!((2.5 - soln["y"]).abs() < 0.001);
    /// ```
    pub fn solve(self, margin: f64, limit: usize) -> anyhow::Result<HashMap<String, f64>>
    {
        let mut guess = HashMap::new();
        for (key, var) in self.context
        {
            match var
            {
                Token::Var(x) => guess.insert(key.into(), (*x.borrow()).into()),
                _ => continue,
            };
        }

        let res = multivariate_newton_raphson(
            self.system_equations, 
            &mut guess,
            margin, 
            limit
        )?;

        Ok(res.clone())
    }
}

/// Returns an iterator with the unknown variables in a given equation or expression. 
/// Note that the variables must exist in the given context in order to ensure that
/// they are variables and not constants or functions.
/// 
/// # Example
/// ```
/// use geqslib::system::get_equation_unknowns;
/// use geqslib::shunting::{ContextHashMap, ContextLike};
/// 
/// let mut ctx = ContextHashMap::new();
/// 
/// for unknown in get_equation_unknowns("x + j = 9", &ctx)
/// {
///     assert!(unknown == "x" || unknown == "j"); // the only variable in our equation specified in ctx
///     assert_ne!(unknown, "y"); // doesn't appear because it is not in ctx
/// }
/// ```
pub fn get_equation_unknowns<'a>(equation: &'a str, ctx: &'a ContextHashMap) -> impl Iterator<Item = &'a str>
{
    get_legal_variables_iter(equation).filter(|&x| !ctx.contains_key(x))
}
