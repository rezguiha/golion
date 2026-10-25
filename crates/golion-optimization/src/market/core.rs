use crate::market::{revenue::RevenueSetter, variables::BidVariables};
use crate::model::BuildEnv;
use golion_domain::market::bid::KiloWattIncrement;
use golion_domain::market::market_type::MarketType;
use golion_domain::market::specification::MarketSpecs;
use golion_domain::temporal::series::TimeSeries;
use golion_domain::temporal::step::Step;
use good_lp::{Constraint, Expression, IntoAffineExpression, ProblemVariables, variable};

#[derive(Debug)]
pub struct Market {
    market_type: MarketType,
    bid_step: Step,
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

    /// Market Revenue Expression
    pub fn revenue(&self) -> &Expression {
        &self.revenue
    }
    /// Market type associated
    pub fn market_type(&self) -> &MarketType {
        &self.market_type
    }
    /// Market step in minutes
    pub fn step(&self) -> &Step {
        &self.bid_step
    }
    /// Builds the market bid variables, valued with the market revenues.
    pub(crate) fn try_new(
        market_specs: &MarketSpecs,
        env: &BuildEnv<'_>,
        vars: &mut ProblemVariables,
    ) -> crate::Result<Self> {
        let market_revenues =
            env.revenues().get(market_specs.market, market_specs.country)?;
        let grid = env.horizon().grid();
        let step = grid.step();
        let bid_windows =
            market_specs.get_bid_windows(grid, &env.horizon().run_time())?;
        let constraints: Vec<Constraint> = Vec::new();
        let mut revenue: Expression = 0.0.into_expression();

        if bid_windows.is_empty() {
            // In case market is unavailable. No bid variables are
            // defined.
            Ok(Market {
                market_type: market_specs.market,
                bid_step: market_specs.product.step,
                increment: market_specs.product.increment,
                variable_store: None,
                constraints,
                revenue,
            })
        } else {
            let increment_value = market_specs.product.increment.value();
            let mut variable_store: Vec<BidVariables> = Vec::new();

            for bid_window in bid_windows.into_iter() {
                let input_variable = vars.add(variable().min(0).integer());
                let output_variable = vars.add(variable().min(0).integer());
                for dt in bid_window.try_iter(&step)? {
                    // We associate same variables for the whole window of the bid step.
                    let input_power = input_variable * increment_value;
                    let output_power = output_variable * increment_value;
                    let bidding_variables =
                        BidVariables::new(dt, input_power, output_power, &step);
                    let revenue_setter = market_revenues.at(&dt)?;
                    revenue += revenue_setter.revenue_expression(&bidding_variables);
                    variable_store.push(bidding_variables);
                }
            }
            Ok(Market {
                market_type: market_specs.market,
                bid_step: market_specs.product.step,
                increment: market_specs.product.increment,
                variable_store: Some(variable_store.try_into()?),
                constraints,
                revenue,
            })
        }
    }
    /// Moves the market constraints out, leaving it empty.
    pub(crate) fn take_constraints(&mut self) -> Vec<Constraint> {
        std::mem::take(&mut self.constraints)
    }
}
