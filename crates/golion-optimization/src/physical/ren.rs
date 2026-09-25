/// Temporary implementation of a renewable asset
use golion_domain::{
    temporal::{series::TimeSeries, step::MinuteStep},
    units::power::KiloWatt,
};

use crate::physical::variables::OtherAssetVariables;

use good_lp::Constraint;

#[derive(Debug)]
pub struct Renewable {
    pub(crate) step: MinuteStep,
    pub(crate) variable_store: TimeSeries<OtherAssetVariables>,
    pub(crate) constraints: Vec<Constraint>,
    pub(crate) rated_output_power: KiloWatt,
}
