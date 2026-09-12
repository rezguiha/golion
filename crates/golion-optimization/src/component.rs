use crate::market::core::Market;
#[derive(Debug)]
pub struct OptimizationComponent<A> {
    physical: A,
    wholesale_markets: Vec<Market>,
    ancillary_markets: Vec<Market>,
}

impl<A> OptimizationComponent<A> {
    pub fn new(
        physical: A,
        wholesale_markets: Vec<Market>,
        ancillary_markets: Vec<Market>,
    ) -> Self {
        Self { physical, wholesale_markets, ancillary_markets }
    }
}
