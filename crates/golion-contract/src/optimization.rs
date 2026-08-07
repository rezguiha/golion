use crate::asset::core::AssetData;
use crate::market::revenue::{AncillaryRevenueSeries, WholeSaleRevenueSeries};
use chrono::{DateTime, TimeDelta, Utc};
use golion_domain::temporal::{grid::RegularTimeGrid, step::MinuteStep};
use serde::{Deserialize, Serialize};

/// Optimization full payload struct.
#[derive(Debug, Serialize, Deserialize)]
pub struct OptimizationInput {
    optimization_start_at: DateTime<Utc>,
    optimization_end_at: DateTime<Utc>,
    optimization_step: TimeDelta,
    assets: Vec<AssetData>,
    ancillary_markets: Vec<AncillaryRevenueSeries>,
    wholesale_markets: Vec<WholeSaleRevenueSeries>,
}
// region: Domain Conversions.
impl TryFrom<OptimizationInput> for RegularTimeGrid {
    type Error = crate::Error;
    fn try_from(value: OptimizationInput) -> crate::Result<Self> {
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
