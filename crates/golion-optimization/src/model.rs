/// Optimization model built out of a validated optimization problem.
use crate::perimeter::ancillary::AncillaryPerimeter;
use crate::perimeter::wholesale::WholesalePerimeter;
use crate::physical::PhysicalStore;
use golion_domain::market::revenue::RevenueStore;
use golion_domain::problem::OptimizationProblem;
use golion_domain::temporal::grid::RegularTimeGrid;
use good_lp::solvers::highs::HighsSolution;
use good_lp::{
    Constraint, Expression, IntoAffineExpression, ProblemVariables, SolverModel, highs,
};
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
    /// Objective of the problem: the revenue of every perimeter, penalizations
    /// included.
    fn revenue(&self) -> Expression {
        let mut revenue = 0.0.into_expression();
        revenue += self.ancillary_perimeter.revenue();
        revenue += self.wholesale_perimeter.revenue();
        revenue
    }
    /// Empties every store of its constraints, in one iterator for the solver.
    fn constraints(&mut self) -> impl Iterator<Item = Constraint> {
        self.physical
            .take_constraints()
            .chain(self.wholesale_perimeter.take_constraints())
            .chain(self.ancillary_perimeter.take_constraints())
    }
}

/// Builds the optimization model of a problem and solves it. Physical assets
/// are built first as perimeters link their variables to them. The model is
/// returned alongside the solution, as its variables are what reads it.
pub fn solve(problem: &OptimizationProblem) -> crate::Result<(Model, HighsSolution)> {
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
    let mut model = Model { physical, wholesale_perimeter, ancillary_perimeter };
    let objective = model.revenue();
    let solution =
        vars.maximise(objective).using(highs).with_all(model.constraints()).solve()?;
    Ok((model, solution))
}

// endregion: Model
