use super::power::KiloWattHour;
use derive_more::{Add, Display, Div, Mul};
#[derive(Debug, Add, Div, Mul, PartialEq, PartialOrd)]
pub struct SocFraction(f64);
#[derive(Debug, Display)]
#[display("Invalid Soc Fraction:{value}.Value must be in [0,1]")]
pub struct InvalidSocFraction {
    value: f64,
}

impl TryFrom<f64> for SocFraction {
    type Error = crate::Error;
    fn try_from(value: f64) -> Result<Self, Self::Error> {
        if (0.0..=1.0).contains(&value) {
            Ok(SocFraction(value))
        } else {
            Err(InvalidSocFraction { value }.into())
        }
    }
}

impl SocFraction {
    pub fn value(&self) -> &f64 {
        &self.0
    }
    pub fn into_energy_kwh(&self, available_energy_kw: &KiloWattHour) -> KiloWattHour {
        KiloWattHour(self.0 * available_energy_kw.0)
    }
}
