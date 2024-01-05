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
pub enum ExpressionCompilationError {
    NoVarsFound,
}
impl_err! {
    ExpressionCompilationError,
    ExpressionCompilationError::NoVarsFound, "failed to find a single unknown variable in the expression"
}