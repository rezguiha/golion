use std::collections::HashMap;

/// Build optimization components out of asset input data.
/// In the future this will also assemble them into one optimization problem,
use crate::error::ServerError;
use golion_contract::asset::core::{AssetData, BessData};

use golion_contract::optimization::OptimizationInput;
use golion_domain::asset::bess::specification::BessSpecifications;
use golion_domain::countries::Countries;
use golion_domain::market::market_type::{
    AncillaryMarketType, MarketType, WholesaleMarketType,
};
use golion_domain::market::revenue::{RevenuePerKiloWatt, RevenuePerKiloWattHour};
use golion_domain::market::specification::MarketSpecs;
use golion_domain::temporal::series::TimeSeries;
use golion_domain::units::power::KiloWattHour;
use golion_optimization::ProblemVariables;
use golion_optimization::component::OptimizationComponent;
use golion_optimization::market::core::Market;
use golion_optimization::physical::bess::core::Battery;
use jiff::Timestamp;

#[derive(Debug)]
pub enum ContractToOptimizationError {
    MissingMarketRevenue { market: MarketType, country: Countries },
}
pub fn build_portfolio(
    input: &OptimizationInput,
    vars: &mut ProblemVariables,
) -> Result<Vec<OptimizationComponent<Battery>>, ServerError> {
    let ancillary_revenue_store: HashMap<
        (AncillaryMarketType, Countries),
        TimeSeries<RevenuePerKiloWatt>,
    > = input
        .ancillary_markets
        .iter()
        .map(|revenue_series| {
            let values = revenue_series
                .values
                .iter()
                .map(RevenuePerKiloWatt::from)
                .collect::<Vec<_>>();
            let series = TimeSeries::<RevenuePerKiloWatt>::try_from(values)?;
            Ok(((revenue_series.market, revenue_series.country), series))
        })
        .collect::<Result<_, ServerError>>()?;
    let wholesale_revenue_store: HashMap<
        (WholesaleMarketType, Countries),
        TimeSeries<RevenuePerKiloWattHour>,
    > = input
        .wholesale_markets
        .iter()
        .map(|revenue_series| {
            let values = revenue_series
                .values
                .iter()
                .map(RevenuePerKiloWattHour::from)
                .collect::<Vec<_>>();
            let series = TimeSeries::<RevenuePerKiloWattHour>::try_from(values)?;
            Ok(((revenue_series.market, revenue_series.country), series))
        })
        .collect::<Result<_, ServerError>>()?;
    input
        .assets
        .iter()
        .map(|asset| match asset {
            AssetData::Bess(data) => build_bess_component(
                data,
                input.optimization_start_at,
                vars,
                &ancillary_revenue_store,
                &wholesale_revenue_store,
            ),
            AssetData::GasTurbine(_) | AssetData::Renewable(_) => {
                Err(ServerError::UnsupportedAssetType)
            }
        })
        .collect()
}

fn build_bess_component(
    data: &BessData,
    reference_time: Timestamp,
    vars: &mut ProblemVariables,
    ancillary_revenue_store: &HashMap<
        (AncillaryMarketType, Countries),
        TimeSeries<RevenuePerKiloWatt>,
    >,
    wholesale_revenue_store: &HashMap<
        (WholesaleMarketType, Countries),
        TimeSeries<RevenuePerKiloWattHour>,
    >,
) -> Result<OptimizationComponent<Battery>, ServerError> {
    let specifications: BessSpecifications = data.try_into()?;
    let time_index: Vec<Timestamp> =
        specifications.limits.availability.grid.iter().collect();
    let step = specifications.limits.availability.grid.step;
    let initial_soc = KiloWattHour(data.initial_soc);

    let mut wholesale_markets = Vec::new();
    let mut ancillary_markets = Vec::new();
    for choice in &data.market_choices {
        let market_specs: MarketSpecs = choice.try_into()?;
        let market_type = market_specs.market;

        match market_type {
            MarketType::WholeSale(m) => {
                let revenue = wholesale_revenue_store
                    .get(&(m, market_specs.country))
                    .ok_or_else(|| ContractToOptimizationError::MissingMarketRevenue {
                        market: m.into(),
                        country: market_specs.country,
                    })?;
                let market = Market::try_new(
                    &reference_time,
                    &time_index,
                    &step,
                    vars,
                    market_specs,
                    revenue,
                )?;
                wholesale_markets.push(market);
            }
            MarketType::Ancillary(m) => {
                let revenue = &ancillary_revenue_store
                    .get(&(m, market_specs.country))
                    .ok_or_else(|| ContractToOptimizationError::MissingMarketRevenue {
                        market: m.into(),
                        country: market_specs.country,
                    })?;
                let market = Market::try_new(
                    &reference_time,
                    &time_index,
                    &step,
                    vars,
                    market_specs,
                    revenue,
                )?;
                ancillary_markets.push(market);
            }
        }
    }

    let battery = Battery::new(&time_index, vars, specifications, initial_soc, step)?;
    Ok(OptimizationComponent::new(battery, wholesale_markets, ancillary_markets))
}
