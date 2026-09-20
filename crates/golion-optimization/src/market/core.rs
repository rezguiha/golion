use crate::{
    market::{revenue::RevenueSetter, variables::BidVariables},
    support::power_to_energy,
};
use golion_domain::market::bid::KiloWattIncrement;
use golion_domain::market::revenue::Revenue;
use golion_domain::market::specification::MarketSpecs;
use golion_domain::temporal::series::TimeSeries;
use golion_domain::temporal::step::MinuteStep;
use good_lp::{Constraint, Expression, IntoAffineExpression, ProblemVariables, variable};
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
    pub bid_step: MinuteStep,
    pub increment: KiloWattIncrement,
    pub variable_store: Option<TimeSeries<BidVariables>>,
    pub constraints: Vec<Constraint>,
    pub revenue: Expression,
}

impl Market {
    pub fn try_new(
        reference_time: &Timestamp,
        time_index: &[Timestamp],
        step: &MinuteStep,
        vars: &mut ProblemVariables,
        market_specs: MarketSpecs,
        market_revenues: &TimeSeries<Revenue>,
    ) -> crate::Result<Self> {
        let bid_time_bounds = market_specs.get_bid_time_bounds(reference_time)?;
        let constraints: Vec<Constraint> = Vec::new();
        let mut revenue: Expression = 0.0.into_expression();
        let Some(bounds) = bid_time_bounds else {
            // In case market is unavailable. No bid variables are
            // defined.
            return Ok(Market {
                bid_step: market_specs.product.step,
                increment: market_specs.product.increment,
                variable_store: None,
                constraints,
                revenue,
            });
        };
        // Make sure bid bounds are inside time index.
        if *bounds.start() < time_index[0]
            || *bounds.end() > time_index[time_index.len() - 1]
        {
            return Err(MarketError::TimeBoundsOutsideIndex {
                index_start: time_index[0],
                index_end: time_index[time_index.len() - 1],
                bidding_start: *bounds.start(),
                bidding_end: *bounds.end(),
            }
            .into());
        }
        let increment_value = market_specs.product.increment.value();
        let mut variable_store: Vec<BidVariables> = Vec::with_capacity(bounds.length());

        for (start, end) in bounds.iter().tuple_windows() {
            let input_variable = vars.add(variable().min(0).integer());
            let output_variable = vars.add(variable().min(0).integer());
            for dt in start.series(step.span()).take_while(|dt| dt < &end) {
                // We associate same variables for the whole window of the bid step.
                let input_power = input_variable * increment_value;
                let input_energy = power_to_energy(&input_power, step.duration());
                let output_power = output_variable * increment_value;
                let output_energy = power_to_energy(&output_power, step.duration());
                let bidding_variables = BidVariables {
                    start_at: dt,
                    input_power,
                    output_power,
                    input_energy,
                    output_energy,
                };
                let revenue_setter = market_revenues.at(&dt)?;
                revenue += revenue_setter.revenue_expression(&bidding_variables);
                variable_store.push(bidding_variables);
            }
        }
        Ok(Market {
            bid_step: market_specs.product.step,
            increment: market_specs.product.increment,
            variable_store: Some(variable_store.try_into()?),
            constraints,
            revenue,
        })
    }
}
