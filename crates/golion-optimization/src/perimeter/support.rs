use crate::{
    market::{core::Market, variables::BidVariables},
    model::Horizon,
};
use golion_domain::{
    market::commitment::Commitment,
    temporal::{
        grid::RegularTimeGrid,
        series::{TimeSeries, TimeStampedUtc},
    },
};
use good_lp::{Expression, IntoAffineExpression, ProblemVariables, variable};
use jiff::Timestamp;
/// Aggregates perimeter commitments and market bids into one
/// perimeter level bidding series, indexed on the optimization grid.
pub(crate) fn aggregate_bidding_and_commitments(
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
        for bid_variables in market.bid_variables() {
            let index = time_grid.index_of(bid_variables.start_at())?;
            input_power_targets[index] += bid_variables.input_power();
            output_power_targets[index] += bid_variables.output_power();
        }
    }
    // Create Bid Variables structs, walking time_index so the resulting
    // series stays chronologically ordered.
    let bid_variables: Vec<BidVariables> = time_index
        .iter()
        .zip(input_power_targets)
        .zip(output_power_targets)
        .map(|((start_at, input_power), output_power)| {
            BidVariables::new(
                *start_at,
                input_power,
                output_power,
                time_grid.step().duration(),
            )
        })
        .collect();
    let variable_store: TimeSeries<BidVariables> = bid_variables.try_into()?;
    Ok(variable_store)
}

pub(crate) fn build_penalization_variables(
    horizon: &Horizon,
    vars: &mut ProblemVariables,
) -> crate::Result<TimeSeries<BidVariables>> {
    let variables: Vec<BidVariables> = horizon
        .timestamps()
        .iter()
        .map(|dt| {
            BidVariables::new(
                *dt,
                vars.add(variable().min(0.0)).into_expression(),
                vars.add(variable().min(0.0)).into_expression(),
                horizon.grid().step().duration(),
            )
        })
        .collect();
    TimeSeries::try_from(variables).map_err(|e| e.into())
}

pub(crate) fn build_penalization_expression(
    variable_store: &TimeSeries<BidVariables>,
    penalty: f64,
) -> Expression {
    variable_store
        .data()
        .iter()
        .map(|bid_var| {
            let mut penalization = 0.0.into_expression();
            penalization.add_mul(penalty, bid_var.input_power());
            penalization.add_mul(penalty, bid_var.output_power());
            penalization
        })
        .sum()
}
