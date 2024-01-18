use std::ops::{MulAssign, AddAssign, SubAssign, DivAssign};

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Variable
{
    value: f64,
    pub min: f64,
    pub max: f64, 
}

impl Variable
{
    pub fn new<T>(value: T, min: T, max: T) -> Variable
    where T: Into<f64>
    {
        Variable
        {
            value: value.into(),
            min: min.into(),
            max: max.into(),
        }
    }

    pub fn set<T>(&mut self, new_value: T) -> ()
    where T: Into<f64>
    {
        if new_value.into() > self.max
        {
            self.value = self.max;
        }
        else if new_value.into() < self.min
        {
            self.value = self.min;
        }
    }
}

impl From<Variable> for f64
{
    fn from(value: Variable) -> Self 
    {
        value.value
    }
}

impl <T> AddAssign<T> for Variable
where T: Into<f64>
{
    fn add_assign(&mut self, rhs: T) 
    {
        self.set(self.value * rhs.into());
    }
}

impl <T> SubAssign<T> for Variable
where T: Into<f64>
{
    fn sub_assign(&mut self, rhs: T) 
    {
        self.set(self.value * rhs.into());
    }
}

impl <T> MulAssign<T> for Variable
where T: Into<f64>
{
    fn mul_assign(&mut self, rhs: T) 
    {
        self.set(self.value * rhs.into());
    }
}

impl <T> DivAssign<T> for Variable
where T: Into<f64>
{
    fn div_assign(&mut self, rhs: T) 
    {
        self.set(self.value * rhs.into());
    }
}