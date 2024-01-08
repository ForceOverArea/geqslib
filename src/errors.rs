use std::error::Error;
use std::fmt;
use std::fmt::Display;

/// More concise syntax for implementing `Error` and `Display` for both structs and enums
macro_rules! impl_err {
    ($s:ty, $e:expr) => {
        impl Error for $s {}
        impl Display for $s {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, $e)
            }
        }
    };
    ($s:ty, $($p:path, $e:expr),*) => {
        impl Error for $s {}
        impl Display for $s {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                match self {
                    $($p => write!(f, $e),)*
                }
            }
        }
    };
}

#[derive(Debug)]
pub enum ShuntingYardError {
    UnclosedParenthesis,
    LeftoverToken,
    UnknownToken,
    ContextMutation,
    ExpectedArg,
    DivisionByZero,
    NoTokens,
}
impl_err! {
    ShuntingYardError,
    ShuntingYardError::UnclosedParenthesis, "found an unclosed parenthesis while converting expression to reverse polish notation",
    ShuntingYardError::LeftoverToken, "found a token when none were expected",
    ShuntingYardError::UnknownToken, "found an unexpected token while converting expression to reverse polish notation",
    ShuntingYardError::ContextMutation, "found reserved token in context",
    ShuntingYardError::ExpectedArg, "expected to find function argument, but none was present on the stack",
    ShuntingYardError::DivisionByZero, "tried to divide by zero during postfix evaluation",
    ShuntingYardError::NoTokens, "expected to find one token in postfix evaluation stack but found none"
}

#[derive(Debug)]
pub struct CompiledExpressionLookupError;
impl_err!(CompiledExpressionLookupError, "failed to find given variable in the function's variable lookup table");

#[derive(Debug)]
pub enum ExpressionCompilationError {
    NoVarsFound,
    WrongVarCount,
    VarNotFoundInContext,
}
impl_err! {
    ExpressionCompilationError,
    ExpressionCompilationError::NoVarsFound, "failed to find a single unknown variable in the expression",
    ExpressionCompilationError::WrongVarCount, "found number of values not equal",
    ExpressionCompilationError::VarNotFoundInContext, "found a legal variable in the expression that did not have a variable in the given context"
}

#[derive(Debug)]
pub enum NewtonRaphsonSolverError {
    NegativeMargin,
    ReachedIterationLimit,
    ImproperlyConstrainedSystem,
}
impl_err! {
    NewtonRaphsonSolverError,
    NewtonRaphsonSolverError::NegativeMargin, "given margin value must be greater than 0",
    NewtonRaphsonSolverError::ReachedIterationLimit, "reached the maximum number of iterations without finding a solution",
    NewtonRaphsonSolverError::ImproperlyConstrainedSystem, "number of functions given did not match the number of variables"
}

#[derive(Debug)]
pub enum EquationSolverError {
    SingleUnknownNotFound,
}
impl_err!{
    EquationSolverError,
    EquationSolverError::SingleUnknownNotFound, "found either no unknowns in given context or too many to solve a single equation"
}