use crate::physical::{bess::core::Battery, ccgt::GasTurbine, ren::Renewable};

pub mod bess;
pub mod ccgt;
pub mod ren;
pub mod variables;
#[derive(Debug)]
pub enum Asset {
    Bess(Battery),
    Ccgt(GasTurbine),
    Ren(Renewable),
}

impl From<Battery> for Asset {
    fn from(value: Battery) -> Self {
        Self::Bess(value)
    }
}
impl From<GasTurbine> for Asset {
    fn from(value: GasTurbine) -> Self {
        Self::Ccgt(value)
    }
}
impl From<Renewable> for Asset {
    fn from(value: Renewable) -> Self {
        Self::Ren(value)
    }
}
