/// Generic Container for timeseries that is linked to a market and country.
/// It enables setting different data structs for different market types.
use garde::Validate;
use golion_domain::countries::Countries;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

/// Container for timeseries data that corresponds to a particular
/// market in a particular country
#[derive(Debug, Deserialize, Serialize, Validate, TypedBuilder)]
pub struct MarketSeries<M, V: Validate<Context = ()>> {
    #[garde(skip)]
    pub market: M,
    #[garde(skip)]
    pub country: Countries,
    #[garde(dive)]
    pub values: Vec<V>,
}
