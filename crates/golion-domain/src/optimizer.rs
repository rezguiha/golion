use crate::problem::OptimizationProblem;
use crate::solution::OptimizationSolution;

// region: Optimizer Port
/// Port through which an optimization problem gets solved.
pub trait Optimizer {
    /// Failures specific to the adapter, such as those of its solver.
    type Error;
    /// Solves the problem, reporting the optimal commitments and revenues.
    fn optimize(
        &self,
        problem: &OptimizationProblem,
    ) -> Result<OptimizationSolution, Self::Error>;
}
// endregion: Optimizer Port
