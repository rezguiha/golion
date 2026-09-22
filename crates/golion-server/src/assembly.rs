/// Build optimization components out of asset input data.
/// In the future this will also assemble them into one optimization problem,
use crate::error::ServerError;
use golion_contract::asset::core::{AssetData, BessData};

use golion_contract::optimization::OptimizationInput;
use golion_contract::perimeter::reserve::ReservePerimeter as ContractReservePerimeter;
use golion_contract::perimeter::wholesale::BrpPerimeter as ContractBrpPerimeter;
use golion_domain::asset::bess::specification::BessSpecifications;
use golion_domain::countries::Countries;
use golion_domain::market::commitment::Commitment;
use golion_domain::market::revenue::RevenueStore;
use golion_domain::temporal::grid::RegularTimeGrid;
use golion_domain::units::power::KiloWattHour;
use golion_optimization::ProblemVariables;
use golion_optimization::component::OptimizationComponent;
use golion_optimization::market::core::Market;
use golion_optimization::perimeter::ancillary::{AncillaryPerimeter, ReservePerimeter};
use golion_optimization::perimeter::wholesale::{BrpPerimeter, WholesalePerimeter};
use golion_optimization::physical::{Asset, PhysicalStore, bess::core::Battery};
use jiff::Timestamp;
use std::collections::HashMap;
use uuid::Uuid;

fn build_bess_component(
    data: &BessData,
    vars: &mut ProblemVariables,
    time_index: &[Timestamp],
) -> Result<Battery, ServerError> {
    let specifications: BessSpecifications = data.try_into()?;
    let step = *specifications.limits.availability.grid().step();
    let initial_soc = KiloWattHour(data.initial_soc);
    Ok(Battery::new(time_index, vars, specifications, initial_soc, step)?)
}

fn build_physical(
    input: &OptimizationInput,
    vars: &mut ProblemVariables,
    time_index: &[Timestamp],
) -> Result<PhysicalStore, ServerError> {
    let store: HashMap<Uuid, Asset> = input
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
        .collect::<Result<_, ServerError>>()?;
    Ok(PhysicalStore::new(store))
}

fn build_wholesale_perimeter_markets(
    perimeter_data: &ContractBrpPerimeter,
    country: &Countries,
    revenue_store: &RevenueStore,
    vars: &mut ProblemVariables,
    time_index: &[Timestamp],
    time_grid: &RegularTimeGrid,
) -> Result<Vec<Market>, ServerError> {
    let mut markets = Vec::<Market>::with_capacity(perimeter_data.markets.len());
    for market_choice in &perimeter_data.markets {
        let market_specs = market_choice.try_into_market_specs(country)?;
        let market_type = market_specs.market;
        let revenue = revenue_store.get(market_type, market_specs.country)?;
        let market = Market::try_new(
            time_grid.start(),
            time_index,
            time_grid.step(),
            vars,
            market_specs,
            revenue,
        )?;
        markets.push(market)
    }
    Ok(markets)
}

fn build_wholesale(
    brp_perimeters: &[ContractBrpPerimeter],
    country: &Countries,
    vars: &mut ProblemVariables,
    revenue_store: &RevenueStore,
    time_index: &[Timestamp],
    time_grid: &RegularTimeGrid,
    physical_store: &PhysicalStore,
) -> Result<WholesalePerimeter, ServerError> {
    let perimeters = brp_perimeters
        .iter()
        .map(|perimeter| {
            let markets = build_wholesale_perimeter_markets(
                perimeter,
                country,
                revenue_store,
                vars,
                time_index,
                time_grid,
            )?;
            Ok(BrpPerimeter::try_new(
                time_index,
                time_grid,
                perimeter
                    .commitments
                    .iter()
                    .flat_map(|series| {
                        series.values.iter().map(|wholsale_commitment| {
                            wholsale_commitment.to_commitment(time_grid.step())
                        })
                    })
                    .collect(),
                markets,
                perimeter.composition.clone(),
                physical_store,
            )?)
        })
        .collect::<Result<_, ServerError>>()?;
    Ok(WholesalePerimeter::try_new(perimeters)?)
}

fn build_ancillary_perimeter(
    reserve_perimeters: &[ContractReservePerimeter],
    time_index: &[Timestamp],
    time_grid: &RegularTimeGrid,
    country: &Countries,
    vars: &mut ProblemVariables,
    revenue_store: &RevenueStore,
    physical_store: &PhysicalStore,
) -> Result<AncillaryPerimeter, ServerError> {
    let perimeters: Vec<ReservePerimeter> = reserve_perimeters
        .iter()
        .map(|perimeter| {
            let market_specs = perimeter.market.try_into_market_specs(country)?;
            let revenue = revenue_store.get(market_specs.market, market_specs.country)?;
            let market = Market::try_new(
                time_grid.start(),
                time_index,
                time_grid.step(),
                vars,
                market_specs,
                revenue,
            )?;
            let commitments: Vec<Commitment> = perimeter
                .commitments
                .iter()
                .map(|ancillary_commitment| ancillary_commitment.into())
                .collect();
            Ok(ReservePerimeter::try_new(
                market,
                perimeter.composition.clone(),
                commitments,
                time_index,
                time_grid,
                vars,
            )?)
        })
        .collect::<Result<_, ServerError>>()?;
    Ok(AncillaryPerimeter::try_new(perimeters, time_index, physical_store)?)
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
    let physical_store = build_physical(input, vars, &time_index)?;
    let wholesale_perimeter = build_wholesale(
        &input.brp_perimeters,
        &input.country,
        vars,
        &revenue_store,
        &time_index,
        &time_grid,
        &physical_store,
    )?;
    let ancillary_perimeter = build_ancillary_perimeter(
        &input.reserve_perimeters,
        &time_index,
        &time_grid,
        &input.country,
        vars,
        &revenue_store,
        &physical_store,
    )?;
    Ok(OptimizationComponent {
        physical: physical_store,
        wholesale_perimeter,
        ancillary_perimeter,
    })
}
