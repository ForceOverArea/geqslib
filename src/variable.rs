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
    where T: Into<f64> + Copy
    {
        Variable
        {
            value: value.into(),
            min: min.into(),
            max: max.into(),
        }
    }

    /// Sets the value of the variable, clamping its 
    /// value to `self.min` or `self.max` if the given 
    /// value exists outside the domain.
    /// 
    /// # Example
    /// ```
    /// use geqslib::variable::Variable;
    /// 
    /// let mut var = Variable::new(1, 0, 10);
    /// 
    /// var.set(11);
    /// 
    /// assert_eq!(f64::from(var), 10.0);
    /// 
    /// var.set(5.005);
    /// 
    /// assert_eq!(f64::from(var), 5.005);
    /// ```
    pub fn set<T>(&mut self, new_value: T)
    where T: Into<f64> + Copy
    {
        if new_value.into() > self.max
        {
            self.value = self.max;
        }
        else if new_value.into() < self.min
        {
            self.value = self.min;
        }
        else
        {
            self.value = new_value.into();
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
        self.set(self.value + rhs.into());
    }
}

impl <T> SubAssign<T> for Variable
where T: Into<f64>
{
    fn sub_assign(&mut self, rhs: T) 
    {
        self.set(self.value - rhs.into());
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
        self.set(self.value / rhs.into());
    }
}