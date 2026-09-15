use crate::asset::core::AssetData;
use crate::market::revenue::{AncillaryRevenueSeries, WholesaleRevenueSeries};
use garde::Validate;
use golion_domain::temporal::{grid::RegularTimeGrid, step::MinuteStep};
use jiff::{SignedDuration, Timestamp};
use serde::{Deserialize, Serialize};
/// Optimization full payload struct.
#[derive(Debug, Validate, Serialize, Deserialize)]
pub struct OptimizationInput {
    /// Reference time for the optimization run, used among other things
    /// to compute which market bid windows are still open.
    #[garde(skip)]
    pub optimization_start_at: Timestamp,
    #[garde(skip)]
    pub optimization_end_at: Timestamp,
    #[garde(skip)]
    pub optimization_step: SignedDuration,
    #[garde(dive)]
    pub assets: Vec<AssetData>,
    #[garde(dive)]
    pub ancillary_revenues: AncillaryRevenueSeries,
    #[garde(dive)]
    pub wholesale_revenues: WholesaleRevenueSeries,
}
/// Optimization output after solve.
/// This is just a temporary definition as project is still
/// in progress and reflects simple information like how many components
/// were built.
/// In the future , this will return optimization results.
#[derive(Debug, Serialize)]
pub struct OptimizationOutput {
    pub components_built: usize,
}

// region: Domain Conversions.
impl TryFrom<&OptimizationInput> for RegularTimeGrid {
    type Error = crate::Error;
    fn try_from(value: &OptimizationInput) -> crate::Result<Self> {
        let step: MinuteStep = value.optimization_step.try_into()?;
        let grid = RegularTimeGrid::try_new_start_end(
            value.optimization_start_at,
            step,
            value.optimization_end_at,
        )?;
        Ok(grid)
    }
}
// endregion: Domain Conversions.
