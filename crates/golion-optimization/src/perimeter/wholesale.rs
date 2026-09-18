use crate::{
    market::{core::Market, variables::BidVariables},
    support::power_to_energy,
};
use golion_domain::{
    market::commitment::Commitment,
    temporal::{grid::RegularTimeGrid, series::TimeSeries},
};
use good_lp::{Expression, IntoAffineExpression};
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
}

impl WholesalePerimeter {
    pub fn try_new(
        time_index: &[Timestamp],
        time_grid: &RegularTimeGrid,
        commitments: Vec<Commitment>,
        markets: Vec<Market>,
        composition: Vec<Uuid>,
    ) -> crate::Result<Self> {
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
        Ok(Self { markets, composition, commitments, variable_store })
    }
}
