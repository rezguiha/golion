use golion_domain::market::revenue::{Revenue, RevenueUnit};
use good_lp::Expression;

use crate::market::variables::BidVariables;

pub trait RevenueSetter {
    fn revenue_expression(&self, bidding_variables: &BidVariables) -> Expression;
}

impl RevenueSetter for Revenue {
    fn revenue_expression(&self, bidding_variables: &BidVariables) -> Expression {
        let (input, output) = match self.unit {
            RevenueUnit::PerKiloWatt => {
                (&bidding_variables.input_power, &bidding_variables.output_power)
            }
            RevenueUnit::PerKiloWattHour => {
                (&bidding_variables.input_energy, &bidding_variables.output_energy)
            }
        };
        let mut revenue = Expression::with_capacity(2);
        revenue.add_mul(self.input_revenue, input);
        revenue.add_mul(self.output_revenue, output);
        revenue
    }
}
