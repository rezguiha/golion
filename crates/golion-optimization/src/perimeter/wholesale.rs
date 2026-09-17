use crate::market::core::Market;
use uuid::Uuid;
#[derive(Debug)]
pub struct WholesalePerimeter {
    /// List of markets to bid on
    pub markets: Vec<Market>,
    /// List of ids of assets inside the perimeter
    pub composition: Vec<Uuid>,
}
