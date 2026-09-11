/// Defines a BESS asset specificationns.
/// These are the minimal attributes needed in input
/// to be able to model it.
use crate::asset::bess::{efficiency::BessPowerEfficiencies, limits::BessLimits};

#[derive(Debug)]
pub struct BessSpecifications {
    pub limits: BessLimits,
    pub efficiencies: BessPowerEfficiencies,
}
