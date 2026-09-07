use crate::market::variables::BidVariables;
use golion_domain::market::bid::KiloWattIncrement;
use golion_domain::market::{bid::BidSpecs, temporality::bid_time_bounds::BidTimeBounds};
use golion_domain::temporal::grid::RegularTimeGrid;
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
        time_index: &[Timestamp],
        step: &MinuteStep,
        vars: &mut ProblemVariables,
        bid_specs: &BidSpecs,
        bid_time_bounds: Option<BidTimeBounds>,
    ) -> crate::Result<Self> {
        let constraints: Vec<Constraint> = Vec::new();
        let Some(bounds) = bid_time_bounds else {
            return Ok(Market {
                bid_step: bid_specs.step,
                increment: bid_specs.increment,
                variable_store: None,
                constraints,
            });
        };
        // Make sure bid bounds are inside time index.
        if bounds.start_at < time_index[0]
            || bounds.end_at > time_index[time_index.len() - 1]
        {
            return Err(MarketError::TimeBoundsOutsideIndex {
                index_start: time_index[0],
                index_end: time_index[time_index.len() - 1],
                bidding_start: bounds.start_at,
                bidding_end: bounds.end_at,
            }
            .into());
        }
        // Construct a regular time grid struct to make use of its validation
        // and its iterator.
        let regular_time_grid = RegularTimeGrid::try_new_start_end(
            bounds.start_at,
            bid_specs.step,
            bounds.end_at,
        )?;
        let mut variable_store: Vec<BidVariables> =
            Vec::with_capacity(regular_time_grid.length);
        for (start, end) in regular_time_grid.iter().tuple_windows() {
            let input_variable = vars.add(variable().min(0).integer());
            let output_variable = vars.add(variable().min(0).integer());
            for dt in start.series(step.span()).take_while(|dt| dt < &end) {
                // We associate same variables for the whole window of the bid step.
                variable_store.push(BidVariables {
                    start_at: dt,
                    input_power: input_variable * bid_specs.increment.value(),
                    output_power: output_variable * bid_specs.increment.value(),
                })
            }
        }
        Ok(Market {
            bid_step: bid_specs.step,
            increment: bid_specs.increment,
            variable_store: Some(variable_store.try_into()?),
            constraints,
        })
    }
}
