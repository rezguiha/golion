use derive_more::{Display, Div, Into, Mul};
#[derive(Debug, Display)]
#[display("Invalid Efficiency value : {value} (must be in [{lower_bound},{upper_bound}]")]
pub struct InvalidEfficiency {
    value: f64,
    lower_bound: f64,
    upper_bound: f64,
}
#[derive(PartialEq, Into, Debug, Clone, Copy, Mul, Div)]

pub struct Efficiency(f64);
impl TryFrom<f64> for Efficiency {
    type Error = crate::Error;
    fn try_from(value: f64) -> Result<Self, Self::Error> {
        let lower_bound = 0.0;
        let upper_bound = 1.0;
        if (value <= lower_bound) | (value > upper_bound) {
            Err(InvalidEfficiency { value, lower_bound, upper_bound }.into())
        } else {
            Ok(Efficiency(value))
        }
    }
}
impl Efficiency {
    pub fn value(&self) -> &f64 {
        &self.0
    }
}
