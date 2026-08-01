use crate::asset::core::AssetData;
use chrono::{DateTime, TimeDelta, Utc};
use golion_domain::temporal::{grid::RegularTimeGrid, step::MinuteGranularity};
use serde::{Deserialize, Serialize};

/// Optimization full payload struct.
#[derive(Debug, Serialize, Deserialize)]
pub struct OptimizationInput {
    optimization_start_at: DateTime<Utc>,
    optimization_end_at: DateTime<Utc>,
    optimization_granularity: TimeDelta,
    assets: Vec<AssetData>,
}
// region: Domain Conversions.
impl TryFrom<OptimizationInput> for RegularTimeGrid {
    type Error = crate::Error;
    fn try_from(value: OptimizationInput) -> crate::Result<Self> {
        let step: MinuteGranularity = value.optimization_granularity.try_into()?;
        let grid = RegularTimeGrid::try_new_start_end(
            value.optimization_start_at,
            step,
            value.optimization_end_at,
        )?;
        Ok(grid)
    }
}
// endregion: Domain Conversions.
