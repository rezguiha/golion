use crate::error::ServerError;
use axum::{Json, Router, routing::post};
use axum_valid::Garde;
use golion_contract::optimization::{OptimizationInput, OptimizationOutput};
use golion_domain::optimizer::Optimizer;
use golion_domain::problem::OptimizationProblem;
use golion_domain::solution::core::OptimizationSolution;
use golion_optimization::HighsOptimizer;

pub fn router() -> Router {
    Router::new().route("/optimize", post(handler))
}

/// Solves the optimization problem of the request. The solution itself isn't
/// reported back yet, so this only proves out the conversion pipeline end to end.
async fn handler(
    Garde(Json(input)): Garde<Json<OptimizationInput>>,
) -> Result<Json<OptimizationOutput>, ServerError> {
    let problem = OptimizationProblem::try_from(input)?;
    let solution = HighsOptimizer::default().optimize(&problem)?;
    Ok(Json(output_from(&problem, &solution)))
}

fn output_from(
    problem: &OptimizationProblem,
    solution: &OptimizationSolution,
) -> OptimizationOutput {
    OptimizationOutput {
        components_built: problem.assets().len(),
        wholesale_perimeters_built: solution.wholesale.len(),
        ancillary_perimeters_built: solution.ancillary.len(),
    }
}
