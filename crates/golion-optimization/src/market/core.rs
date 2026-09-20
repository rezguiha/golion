use crate::market::{revenue::RevenueSetter, variables::BidVariables};
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
    bid_step: MinuteStep,
    increment: KiloWattIncrement,
    variable_store: Option<TimeSeries<BidVariables>>,
    constraints: Vec<Constraint>,
    revenue: Expression,
}

impl Market {
    /// Bids placed on this market, empty while the market is unavailable.
    pub fn bid_variables(&self) -> impl Iterator<Item = &BidVariables> {
        self.variable_store.iter().flat_map(|store| store.data().iter())
    }

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
                let output_power = output_variable * increment_value;
                let bidding_variables =
                    BidVariables::new(dt, input_power, output_power, step.duration());
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
