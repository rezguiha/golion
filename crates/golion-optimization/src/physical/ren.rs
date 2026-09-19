/// Temporary implementation of a renewable asset
use golion_domain::temporal::{series::TimeSeries, step::MinuteStep};

use crate::physical::variables::OtherAssetVariables;

use good_lp::Constraint;

#[derive(Debug)]
pub struct Renewable {
    pub step: MinuteStep,
    pub variable_store: TimeSeries<OtherAssetVariables>,
    pub constraints: Vec<Constraint>,
}
