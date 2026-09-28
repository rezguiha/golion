use crate::model::Model;
use golion_domain::optimizer::Optimizer;
use golion_domain::problem::OptimizationProblem;
use golion_domain::solution::OptimizationSolution;
use good_lp::{SolverModel, highs};

// region: HiGHS Optimizer
/// Optimizer adapter solving the model with HiGHS.
#[derive(Debug, Clone, Copy)]
pub struct HighsOptimizer {
    /// Maximum solving time in seconds.
    time_limit: f64,
    /// Relative gap under which a MIP solution is deemed optimal.
    mip_rel_gap: f32,
    /// Whether HiGHS logs its progress.
    verbose: bool,
}

impl Default for HighsOptimizer {
    fn default() -> Self {
        Self { time_limit: 60.0, mip_rel_gap: 1e-3, verbose: false }
    }
}

impl Optimizer for HighsOptimizer {
    type Error = crate::Error;

    fn optimize(
        &self,
        problem: &OptimizationProblem,
    ) -> crate::Result<OptimizationSolution> {
        let (mut model, vars) = Model::try_new(problem)?;
        let objective = model.net_revenue();
        let mut highs_problem = vars
            .maximise(objective)
            .using(highs)
            .set_time_limit(self.time_limit)
            .set_mip_rel_gap(self.mip_rel_gap)?;
        highs_problem.set_verbose(self.verbose);
        let solution = highs_problem.with_all(model.constraints()).solve()?;
        model.to_solution(&solution)
    }
}
// endregion: HiGHS Optimizer
