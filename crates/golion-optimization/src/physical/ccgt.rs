/// Temporary implementation of a gas turbine asset.
use golion_domain::temporal::{series::TimeSeries, step::MinuteStep};

use crate::physical::variables::OtherAssetVariables;

use good_lp::Constraint;

#[derive(Debug)]
pub struct GasTurbine {
    step: MinuteStep,
    variable_store: TimeSeries<OtherAssetVariables>,
    constraints: Vec<Constraint>,
}
