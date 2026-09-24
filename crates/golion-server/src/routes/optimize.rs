use crate::error::ServerError;
use axum::{Json, Router, routing::post};
use axum_valid::Garde;
use golion_contract::optimization::{OptimizationInput, OptimizationOutput};
use golion_domain::problem::OptimizationProblem;
use golion_optimization::Model;

pub fn router() -> Router {
    Router::new().route("/optimize", post(handler))
}

/// Builds and solves the optimization model of the request. The solution
/// itself isn't reported back yet, so this only proves out the conversion
/// pipeline end to end.
async fn handler(
    Garde(Json(input)): Garde<Json<OptimizationInput>>,
) -> Result<Json<OptimizationOutput>, ServerError> {
    let problem = OptimizationProblem::try_from(input)?;
    let (model, _solution) = golion_optimization::solve(&problem)?;
    Ok(Json(output_from(&model)))
}

fn output_from(model: &Model) -> OptimizationOutput {
    OptimizationOutput {
        components_built: model.physical().len(),
        wholesale_perimeters_built: model.wholesale_perimeter().brp_perimeters().len(),
        ancillary_perimeters_built: model
            .ancillary_perimeter()
            .reserve_perimeters()
            .len(),
    }
}
