/// Optimization model built out of a validated optimization problem.
use crate::perimeter::ancillary::AncillaryPerimeter;
use crate::perimeter::wholesale::WholesalePerimeter;
use crate::physical::PhysicalStore;
use golion_domain::market::revenue::RevenueStore;
use golion_domain::problem::OptimizationProblem;
use golion_domain::temporal::grid::RegularTimeGrid;
use good_lp::ProblemVariables;
use jiff::Timestamp;

// region: Horizon
/// Time horizon of the model. Owns the timestamps of the grid, computed
/// once, so that every component iterates over the exact same slots.
pub(crate) struct Horizon<'a> {
    grid: &'a RegularTimeGrid,
    timestamps: Box<[Timestamp]>,
}

impl<'a> Horizon<'a> {
    fn new(grid: &'a RegularTimeGrid) -> Self {
        Self { grid, timestamps: grid.iter().collect() }
    }
    pub(crate) fn grid(&self) -> &RegularTimeGrid {
        self.grid
    }
    pub(crate) fn timestamps(&self) -> &[Timestamp] {
        &self.timestamps
    }
}
// endregion: Horizon

// region: Build Environment
/// Read-only inputs shared by every model component during construction.
/// Variables are passed separately, as the only state mutated while building.
pub(crate) struct BuildEnv<'a> {
    horizon: Horizon<'a>,
    revenues: &'a RevenueStore,
}

impl<'a> BuildEnv<'a> {
    fn new(grid: &'a RegularTimeGrid, revenues: &'a RevenueStore) -> Self {
        Self { horizon: Horizon::new(grid), revenues }
    }
    pub(crate) fn horizon(&self) -> &Horizon<'a> {
        &self.horizon
    }
    pub(crate) fn revenues(&self) -> &RevenueStore {
        self.revenues
    }
}
// endregion: Build Environment

// region: Model
pub struct Model {
    vars: ProblemVariables,
    physical: PhysicalStore,
    wholesale_perimeter: WholesalePerimeter,
    ancillary_perimeter: AncillaryPerimeter,
}

impl Model {
    pub fn physical(&self) -> &PhysicalStore {
        &self.physical
    }
    pub fn wholesale_perimeter(&self) -> &WholesalePerimeter {
        &self.wholesale_perimeter
    }
    pub fn ancillary_perimeter(&self) -> &AncillaryPerimeter {
        &self.ancillary_perimeter
    }
}

/// Builds the optimization model of a problem. Physical assets are built
/// first as perimeters link their variables to them.
pub fn build(problem: &OptimizationProblem) -> crate::Result<Model> {
    let env = BuildEnv::new(problem.grid(), problem.revenues());
    let mut vars = ProblemVariables::new();
    let physical = PhysicalStore::try_new(problem.assets(), &env, &mut vars)?;
    let wholesale_perimeter = WholesalePerimeter::try_new(
        problem.brp_perimeters(),
        &physical,
        &env,
        &mut vars,
    )?;
    let ancillary_perimeter = AncillaryPerimeter::try_new(
        problem.reserve_perimeters(),
        &physical,
        &env,
        &mut vars,
    )?;
    Ok(Model { vars, physical, wholesale_perimeter, ancillary_perimeter })
}

// endregion: Model
