/// Optimization model built out of a validated optimization problem.
use crate::perimeter::ancillary::AncillaryPerimeter;
use crate::perimeter::wholesale::WholesalePerimeter;
use crate::physical::PhysicalStore;
use golion_domain::market::revenue::RevenueStore;
use golion_domain::problem::OptimizationProblem;
use golion_domain::solution::{FailureReason, OptimizationOutcome, OptimizationSolution};
use golion_domain::temporal::grid::RegularTimeGrid;
use good_lp::{Constraint, Expression, IntoAffineExpression, ProblemVariables};
use jiff::{SignedDuration, Timestamp};

// region: Horizon
/// Time horizon of the model. Owns the timestamps of the grid, computed
/// once, so that every component iterates over the exact same slots.
pub(crate) struct Horizon {
    /// Reference run time necessary to project initial soc for example
    /// for bess assets to optimization start.
    run_time: Timestamp,
    /// Time grid from optimization start until end on regular step.
    grid: RegularTimeGrid,
    /// Timestamps of grid computed once to be reused and not recomputed.
    timestamps: Box<[Timestamp]>,
}

impl Horizon {
    pub fn new(run_time: &Timestamp, grid: &RegularTimeGrid) -> Self {
        Self { run_time: *run_time, grid: *grid, timestamps: grid.iter().collect() }
    }
    pub(crate) fn grid(&self) -> &RegularTimeGrid {
        &self.grid
    }
    pub(crate) fn timestamps(&self) -> &[Timestamp] {
        &self.timestamps
    }
    pub(crate) fn duration_to_grid_start(&self) -> crate::Result<SignedDuration> {
        Ok(SignedDuration::try_from(self.grid.start() - self.run_time)
            .map_err(golion_domain::error::Error::JiffConversionError)?)
    }
}
// endregion: Horizon

// region: Build Environment
/// Read-only inputs shared by every model component during construction.
/// Variables are passed separately, as the only state mutated while building.
pub(crate) struct BuildEnv<'a> {
    horizon: Horizon,
    revenues: &'a RevenueStore,
}

impl<'a> BuildEnv<'a> {
    fn new(
        run_time: &Timestamp,
        grid: &RegularTimeGrid,
        revenues: &'a RevenueStore,
    ) -> Self {
        Self { horizon: Horizon::new(run_time, grid), revenues }
    }
    pub(crate) fn horizon(&self) -> &Horizon {
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
    /// Objective of the problem: the net revenue (revenue + penalty)
    /// of every perimeter
    pub(crate) fn net_revenue(&self) -> Expression {
        let mut net_revenue = 0.0.into_expression();
        net_revenue += self.ancillary_perimeter.revenue();
        net_revenue += self.ancillary_perimeter.penalty();
        net_revenue += self.wholesale_perimeter.revenue();
        net_revenue += self.wholesale_perimeter.penalty();
        net_revenue
    }
    /// Empties every store of its constraints, in one iterator for the solver.
    pub(crate) fn constraints(&mut self) -> impl Iterator<Item = Constraint> {
        self.physical
            .take_constraints()
            .chain(self.wholesale_perimeter.take_constraints())
            .chain(self.ancillary_perimeter.take_constraints())
    }
    /// Builds the optimization model of a problem, independently of the solver.
    /// Physical assets are built first as perimeters link their variables to
    /// them. The variables are returned alongside, for the solver to consume.
    pub(crate) fn try_new(
        problem: OptimizationProblem,
    ) -> crate::Result<(Self, ProblemVariables)> {
        let problem_data = problem.into_data();
        let env = BuildEnv::new(
            &problem_data.run_time,
            &problem_data.grid,
            &problem_data.revenues,
        );
        let mut vars = ProblemVariables::new();

        let physical = PhysicalStore::try_new(problem_data.assets, &env, &mut vars)?;
        let wholesale_perimeter = WholesalePerimeter::try_new(
            &problem_data.brp_perimeters,
            &physical,
            &env,
            &mut vars,
        )?;
        let ancillary_perimeter = AncillaryPerimeter::try_new(
            &problem_data.reserve_perimeters,
            &physical,
            &env,
            &mut vars,
        )?;
        let model = Self { physical, wholesale_perimeter, ancillary_perimeter };
        Ok((model, vars))
    }
}

// endregion: Model

// region : Solution Conversion

impl Model {
    fn to_solution(
        &self,
        solution: &impl good_lp::Solution,
    ) -> crate::Result<OptimizationSolution> {
        Ok(OptimizationSolution {
            wholesale: self
                .wholesale_perimeter
                .brp_perimeters()
                .iter()
                .map(|brp| brp.to_solution(solution))
                .collect::<crate::Result<_>>()?,
            ancillary: self
                .ancillary_perimeter
                .reserve_perimeters()
                .iter()
                .map(|reserve| reserve.to_solution(solution))
                .collect::<crate::Result<_>>()?,
            physical: self.physical.to_solution(solution),
        })
    }
    pub(crate) fn to_outcome(
        &self,
        resolution: Result<impl good_lp::Solution, good_lp::ResolutionError>,
    ) -> crate::Result<OptimizationOutcome> {
        match resolution {
            Err(e) => match e {
                good_lp::ResolutionError::Infeasible => {
                    Ok(OptimizationOutcome::Failure { reason: FailureReason::Infeasible })
                }
                good_lp::ResolutionError::Unbounded => {
                    Ok(OptimizationOutcome::Failure { reason: FailureReason::Unbounded })
                }
                e => Err(e.into()),
            },
            Ok(solution) => match solution.status() {
                good_lp::SolutionStatus::TimeLimit => Ok(OptimizationOutcome::Failure {
                    reason: FailureReason::LimitReached,
                }),
                good_lp::SolutionStatus::GapLimit => Ok(OptimizationOutcome::Success {
                    optimality: golion_domain::solution::Optimality::WithinGap,
                    solution: self.to_solution(&solution)?,
                }),
                good_lp::SolutionStatus::Optimal => Ok(OptimizationOutcome::Success {
                    optimality: golion_domain::solution::Optimality::Optimal,
                    solution: self.to_solution(&solution)?,
                }),
            },
        }
    }
}

// endregion : Solution Conversion
