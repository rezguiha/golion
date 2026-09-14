use golion_domain::market::revenue::{RevenuePerKiloWatt, RevenuePerKiloWattHour};
use good_lp::Expression;

use crate::market::variables::BidVariables;

pub trait RevenueSetter {
    fn revenue_expression(&self, bidding_variables: &BidVariables) -> Expression;
}

impl RevenueSetter for RevenuePerKiloWatt {
    fn revenue_expression(&self, bidding_variables: &BidVariables) -> Expression {
        let mut revenue = Expression::with_capacity(2);
        revenue.add_mul(self.input_revenue, &bidding_variables.input_power);
        revenue.add_mul(self.output_revenue, &bidding_variables.output_power);
        revenue
    }
}

impl RevenueSetter for RevenuePerKiloWattHour {
    fn revenue_expression(&self, bidding_variables: &BidVariables) -> Expression {
        let mut revenue = Expression::with_capacity(2);
        revenue.add_mul(self.input_revenue, &bidding_variables.input_energy);
        revenue.add_mul(self.output_revenue, &bidding_variables.output_energy);
        revenue
    }
}
