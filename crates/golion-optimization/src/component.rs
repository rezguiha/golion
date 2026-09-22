use crate::perimeter::ancillary::AncillaryPerimeter;
use crate::perimeter::wholesale::WholesalePerimeter;
use crate::physical::PhysicalStore;

#[derive(Debug)]
pub struct OptimizationComponent {
    pub physical: PhysicalStore,
    pub wholesale_perimeter: WholesalePerimeter,
    pub ancillary_perimeter: AncillaryPerimeter,
}
