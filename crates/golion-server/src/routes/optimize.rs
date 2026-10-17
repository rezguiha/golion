use crate::error::ServerError;
use axum::{Json, Router, routing::post};
use axum_valid::Garde;
use golion_contract::optimization::input::OptimizationInput;
use golion_contract::optimization::output::OptimizationOutput;
use golion_domain::optimizer::Optimizer;
use golion_domain::problem::OptimizationProblem;
use golion_optimization::HighsOptimizer;
use std::time::Instant;
use tokio::task;
use tracing::info;
pub fn router() -> Router {
    Router::new().route("/optimize", post(handler))
}

/// Solves the optimization problem of the request and reports its results
/// per perimeter.
///
/// The solve is CPU-bound, so it runs on tokio's blocking pool to keep the
/// async workers free to serve other requests.
async fn handler(
    Garde(Json(input)): Garde<Json<OptimizationInput>>,
) -> Result<Json<OptimizationOutput>, ServerError> {
    let start = Instant::now();
    let problem = OptimizationProblem::try_from(input)?;
    info!("{:<12} - {:?}\n", "OPTIMIZATION PROBLEM CONSTRUCTION IN : ", start.elapsed());
    let start = Instant::now();
    let solution =
        task::spawn_blocking(move || HighsOptimizer::default().optimize(problem))
            .await??;
    info!("{:<12} - {:?}\n", "OPTIMIZATION SOLVE SUCCESSFUL IN : ", start.elapsed());

    Ok(Json(solution.into()))
}
