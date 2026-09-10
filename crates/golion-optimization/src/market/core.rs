use crate::market::variables::BidVariables;
use golion_domain::market::bid::KiloWattIncrement;
use golion_domain::market::specification::MarketSpec;
use golion_domain::temporal::series::TimeSeries;
use golion_domain::temporal::step::MinuteStep;
use good_lp::{Constraint, ProblemVariables, variable};
use itertools::Itertools;
use jiff::Timestamp;

#[derive(Debug)]
pub enum MarketError {
    TimeBoundsOutsideIndex {
        index_start: Timestamp,
        index_end: Timestamp,
        bidding_start: Timestamp,
        bidding_end: Timestamp,
    },
}
#[derive(Debug)]
pub struct Market {
    bid_step: MinuteStep,
    increment: KiloWattIncrement,
    variable_store: Option<TimeSeries<BidVariables>>,
    constraints: Vec<Constraint>,
}

impl Market {
    pub fn try_new(
        reference_time: &Timestamp,
        time_index: &[Timestamp],
        step: &MinuteStep,
        vars: &mut ProblemVariables,
        market_specs: MarketSpec,
    ) -> crate::Result<Self> {
        let bid_time_bounds = market_specs.get_bid_time_bounds(reference_time)?;
        let constraints: Vec<Constraint> = Vec::new();
        let Some(bounds) = bid_time_bounds else {
            // In case market is unavailable. No bid variables are
            // defined.
            return Ok(Market {
                bid_step: market_specs.product.step,
                increment: market_specs.product.increment,
                variable_store: None,
                constraints,
            });
        };
        // Make sure bid bounds are inside time index.
        if bounds.start < time_index[0] || bounds.end > time_index[time_index.len() - 1] {
            return Err(MarketError::TimeBoundsOutsideIndex {
                index_start: time_index[0],
                index_end: time_index[time_index.len() - 1],
                bidding_start: bounds.start,
                bidding_end: bounds.end,
            }
            .into());
        }
        let increment_value = market_specs.product.increment.value();
        let mut variable_store: Vec<BidVariables> = Vec::with_capacity(bounds.length);
        for (start, end) in bounds.iter().tuple_windows() {
            let input_variable = vars.add(variable().min(0).integer());
            let output_variable = vars.add(variable().min(0).integer());
            for dt in start.series(step.span()).take_while(|dt| dt < &end) {
                // We associate same variables for the whole window of the bid step.
                variable_store.push(BidVariables {
                    start_at: dt,
                    input_power: input_variable * increment_value,
                    output_power: output_variable * increment_value,
                })
            }
        }
        Ok(Market {
            bid_step: market_specs.product.step,
            increment: market_specs.product.increment,
            variable_store: Some(variable_store.try_into()?),
            constraints,
        })
    }
}
