/// Build optimization components out of asset input data.
/// In the future this will also assemble them into one optimization problem,
use crate::error::ServerError;
use golion_contract::asset::core::{AssetData, BessData};

use golion_contract::optimization::OptimizationInput;
use golion_contract::perimeter::wholesale::WholesalePerimeter as ContractWholesalePerimeter;
use golion_domain::asset::bess::specification::BessSpecifications;
use golion_domain::countries::Countries;
use golion_domain::market::revenue::RevenueStore;
use golion_domain::temporal::grid::RegularTimeGrid;
use golion_domain::temporal::step::MinuteStep;
use golion_domain::units::power::KiloWattHour;
use golion_optimization::ProblemVariables;
use golion_optimization::component::OptimizationComponent;
use golion_optimization::market::core::Market;
use golion_optimization::perimeter::ancillary::AncillaryPerimeter;
use golion_optimization::perimeter::wholesale::WholesalePerimeter;
use golion_optimization::physical::{Asset, bess::core::Battery};
use jiff::Timestamp;
use std::collections::HashMap;
use uuid::Uuid;

fn build_bess_component(
    data: &BessData,
    vars: &mut ProblemVariables,
    time_index: &[Timestamp],
) -> Result<Battery, ServerError> {
    let specifications: BessSpecifications = data.try_into()?;
    let step = specifications.limits.availability.grid.step;
    let initial_soc = KiloWattHour(data.initial_soc);
    Ok(Battery::new(time_index, vars, specifications, initial_soc, step)?)
}

fn build_physical(
    input: &OptimizationInput,
    vars: &mut ProblemVariables,
    time_index: &[Timestamp],
) -> Result<HashMap<Uuid, Asset>, ServerError> {
    input
        .assets
        .iter()
        .map(|asset| match asset {
            AssetData::Bess(data) => {
                let battery = build_bess_component(data, vars, time_index)?;
                Ok((data.identification.asset_id, Asset::from(battery)))
            }
            AssetData::GasTurbine(_) | AssetData::Renewable(_) => {
                Err(ServerError::UnsupportedAssetType)
            }
        })
        .collect()
}

fn build_wholesale_perimeter_markets(
    perimeter_data: &ContractWholesalePerimeter,
    country: &Countries,
    revenue_store: &RevenueStore,
    vars: &mut ProblemVariables,
    time_index: &[Timestamp],
    step: MinuteStep,
    reference_time: Timestamp,
) -> Result<Vec<Market>, ServerError> {
    let mut markets = Vec::<Market>::with_capacity(perimeter_data.markets.len());
    for market_choice in &perimeter_data.markets {
        let market_specs = market_choice.try_into_market_specs(country)?;
        let market_type = market_specs.market;
        let revenue = revenue_store.get(market_type, market_specs.country)?;
        let market = Market::try_new(
            &reference_time,
            time_index,
            &step,
            vars,
            market_specs,
            revenue,
        )?;
        markets.push(market)
    }
    Ok(markets)
}

fn build_wholesale(
    input: &OptimizationInput,
    vars: &mut ProblemVariables,
    revenue_store: &RevenueStore,
    time_index: &[Timestamp],
) -> Result<Vec<WholesalePerimeter>, ServerError> {
    input
        .wholesale_perimeters
        .iter()
        .map(|perimeter| {
            let markets = build_wholesale_perimeter_markets(
                perimeter,
                &input.country,
                revenue_store,
                vars,
                time_index,
                input.optimization_step.try_into()?,
                input.optimization_start_at,
            )?;
            Ok(WholesalePerimeter { markets, composition: perimeter.composition.clone() })
        })
        .collect()
}

pub fn build_portfolio(
    input: &OptimizationInput,
    vars: &mut ProblemVariables,
) -> Result<OptimizationComponent, ServerError> {
    let time_grid = RegularTimeGrid::try_new_start_end(
        input.optimization_start_at,
        input.optimization_step.try_into()?,
        input.optimization_end_at,
    )?;
    let time_index: Vec<Timestamp> = time_grid.iter().collect();
    let revenue_store = RevenueStore::try_from(input)?;
    let physical = build_physical(input, vars, &time_index)?;
    let wholesale_perimeters = build_wholesale(input, vars, &revenue_store, &time_index)?;
    // Temporarily use an empty list of ancillary perimeters.
    let ancillary_perimeters = Vec::<AncillaryPerimeter>::new();
    Ok(OptimizationComponent { physical, wholesale_perimeters, ancillary_perimeters })
}
