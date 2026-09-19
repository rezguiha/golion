use crate::{
    market::{core::Market, variables::BidVariables},
    physical::PhysicalStore,
    support::power_to_energy,
};
use golion_domain::{
    market::commitment::Commitment,
    temporal::{grid::RegularTimeGrid, series::TimeSeries},
};
use good_lp::{Constraint, Expression, IntoAffineExpression, constraint};
use jiff::Timestamp;
use uuid::Uuid;
#[derive(Debug)]
pub struct WholesalePerimeter {
    /// List of markets to bid on
    pub markets: Vec<Market>,
    /// List of ids of assets inside the perimeter
    pub composition: Vec<Uuid>,
    /// List of commitments on all wholesale markets
    /// for the perimeter.
    pub commitments: Vec<Commitment>,
    /// Perimeter level bidding variables.
    pub variable_store: TimeSeries<BidVariables>,
    /// Perimeter constraints.
    pub constraints: Vec<Constraint>,
}

impl WholesalePerimeter {
    pub fn try_new(
        time_index: &[Timestamp],
        time_grid: &RegularTimeGrid,
        commitments: Vec<Commitment>,
        markets: Vec<Market>,
        composition: Vec<Uuid>,
        physical_store: &PhysicalStore,
    ) -> crate::Result<Self> {
        let variable_store =
            Self::build_variable_store(time_index, time_grid, &commitments, &markets)?;

        // Add Repartition Constraints
        let mut constraints = Vec::<Constraint>::with_capacity(time_index.len());
        Self::build_repartition_constraints(
            &mut constraints,
            &variable_store,
            &composition,
            time_index,
            physical_store,
        )?;
        Ok(Self { markets, composition, commitments, variable_store, constraints })
    }

    /// Aggregates perimeter commitments and market bids into one
    /// perimeter level bidding series, indexed on the optimization grid.
    fn build_variable_store(
        time_index: &[Timestamp],
        time_grid: &RegularTimeGrid,
        commitments: &[Commitment],
        markets: &[Market],
    ) -> crate::Result<TimeSeries<BidVariables>> {
        // Initialize perimiter target powers to 0.0
        let mut input_power_targets: Vec<Expression> =
            time_index.iter().map(|_| 0.0.into_expression()).collect();
        let mut output_power_targets: Vec<Expression> =
            time_index.iter().map(|_| 0.0.into_expression()).collect();
        // Aggregate commitments and market bids.
        for commitment in commitments.iter() {
            let index = time_grid.index_of(&commitment.start_at)?;
            input_power_targets[index] += commitment.input_power.0;
            output_power_targets[index] += commitment.output_power.0;
        }
        for market in markets.iter() {
            if let Some(market_variables) = &market.variable_store {
                for bid_variables in market_variables.data.iter() {
                    let index = time_grid.index_of(&bid_variables.start_at)?;
                    input_power_targets[index] += &bid_variables.input_power;
                    output_power_targets[index] += &bid_variables.output_power;
                }
            }
        }
        // Create Bid Variables structs, walking time_index so the resulting
        // series stays chronologically ordered.
        let bid_variables: Vec<BidVariables> = time_index
            .iter()
            .zip(input_power_targets)
            .zip(output_power_targets)
            .map(|((start_at, input_power), output_power)| BidVariables {
                start_at: *start_at,
                input_energy: power_to_energy(&input_power, time_grid.step.duration()),
                output_energy: power_to_energy(&output_power, time_grid.step.duration()),
                input_power,
                output_power,
            })
            .collect();
        let variable_store: TimeSeries<BidVariables> = bid_variables.try_into()?;
        Ok(variable_store)
    }

    fn build_repartition_constraints(
        constraints: &mut Vec<Constraint>,
        variable_store: &TimeSeries<BidVariables>,
        composition: &[Uuid],
        time_index: &[Timestamp],
        physical_store: &PhysicalStore,
    ) -> crate::Result<()> {
        for dt in time_index.iter() {
            let perimeter_variables = variable_store.at(dt)?;
            // We defined perimeter net as a 0 expression and add to it input power
            // and subtract output power to avoid moving values behind them.
            // This enables us to avoid that.
            let mut perimeter_net = 0.0.into_expression();
            perimeter_net.add_mul(1.0, &perimeter_variables.input_power);
            perimeter_net.add_mul(-1.0, &perimeter_variables.output_power);
            let mut sum_asset_net = 0.0.into_expression();
            for asset_id in composition.iter() {
                let asset = physical_store.get(asset_id)?;
                sum_asset_net += asset.input_power_at(dt)? - asset.output_power_at(dt)?;
            }
            constraints.push(constraint!(sum_asset_net == perimeter_net));
        }
        Ok(())
    }
}
