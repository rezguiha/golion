use crate::asset::core::AssetData;
use crate::market::revenue::{AncillaryRevenueSeries, WholesaleRevenueSeries};
use crate::perimeter::reserve::ReservePerimeter;
use crate::perimeter::wholesale::BrpPerimeter;
use garde::Validate;
use golion_domain::countries::Countries;
use golion_domain::market::revenue::{Revenue, RevenueStore};
use golion_domain::problem::{OptimizationProblem, ProblemData};
use golion_domain::temporal::{grid::RegularTimeGrid, series::TimeSeries, step::Step};
use jiff::{SignedDuration, Timestamp, TimestampRound, Unit};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
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
    #[garde(skip)]
    pub country: Countries,
    #[garde(dive)]
    pub assets: Vec<AssetData>,
    #[garde(dive)]
    pub reserve_perimeters: Vec<ReservePerimeter>,
    #[garde(dive)]
    pub brp_perimeters: Vec<BrpPerimeter>,
    #[garde(dive)]
    pub ancillary_revenues: AncillaryRevenueSeries,
    #[garde(dive)]
    pub wholesale_revenues: WholesaleRevenueSeries,
}

// region: Domain Conversions.
impl TryFrom<&OptimizationInput> for RevenueStore {
    type Error = crate::Error;
    fn try_from(value: &OptimizationInput) -> crate::Result<Self> {
        let ancillary = value.ancillary_revenues.iter().map(|revenue_series| {
            let values: Vec<Revenue> =
                revenue_series.values.iter().map(Revenue::from).collect();
            let series = TimeSeries::<Revenue>::try_from(values)?;
            Ok((revenue_series.market.into(), series))
        });
        let wholesale = value.wholesale_revenues.iter().map(|revenue_series| {
            let values: Vec<Revenue> =
                revenue_series.values.iter().map(Revenue::from).collect();
            let series = TimeSeries::<Revenue>::try_from(values)?;
            Ok((revenue_series.market.into(), series))
        });
        let store: HashMap<_, _> =
            ancillary.chain(wholesale).collect::<crate::Result<_>>()?;
        Ok(store.into())
    }
}

impl TryFrom<&OptimizationInput> for RegularTimeGrid {
    type Error = crate::Error;
    fn try_from(value: &OptimizationInput) -> crate::Result<Self> {
        let step: Step = value.optimization_step.try_into()?;
        // Round to next multiple of grid step.
        let start = value
            .optimization_start_at
            .round(
                TimestampRound::new()
                    .smallest(Unit::Minute)
                    .increment(step.duration().as_mins())
                    .mode(jiff::RoundMode::Ceil),
            )
            .map_err(golion_domain::Error::JiffConversionError)?;
        let grid =
            RegularTimeGrid::try_new_start_end(start, step, value.optimization_end_at)?;
        Ok(grid)
    }
}

impl TryFrom<OptimizationInput> for OptimizationProblem {
    type Error = crate::Error;
    fn try_from(value: OptimizationInput) -> crate::Result<Self> {
        let grid = RegularTimeGrid::try_from(&value)?;
        let revenues = RevenueStore::try_from(&value)?;
        let OptimizationInput {
            country, assets, brp_perimeters, reserve_perimeters, ..
        } = value;
        let assets = assets
            .into_iter()
            .map(AssetData::try_into_definition)
            .collect::<crate::Result<BTreeMap<_, _>>>()?;
        let brp_perimeters = brp_perimeters
            .into_iter()
            .map(|perimeter| perimeter.try_into_definition(&country, &grid))
            .collect::<crate::Result<_>>()?;
        let reserve_perimeters = reserve_perimeters
            .into_iter()
            .map(|perimeter| perimeter.try_into_definition(&country, &grid))
            .collect::<crate::Result<_>>()?;
        Ok(ProblemData {
            run_time: value.optimization_start_at,
            grid,
            assets,
            brp_perimeters,
            reserve_perimeters,
            revenues,
        }
        .try_into()?)
    }
}
// endregion: Domain Conversions.
