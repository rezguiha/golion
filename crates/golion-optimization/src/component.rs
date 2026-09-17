use uuid::Uuid;

use crate::perimeter::ancillary::AncillaryPerimeter;
use crate::perimeter::wholesale::WholesalePerimeter;
use crate::physical::Asset;
use std::collections::HashMap;
#[derive(Debug)]
pub struct OptimizationComponent {
    pub physical: HashMap<Uuid, Asset>,
    pub wholesale_perimeters: Vec<WholesalePerimeter>,
    pub ancillary_perimeters: Vec<AncillaryPerimeter>,
}
