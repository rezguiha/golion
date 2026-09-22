use crate::assembly;
use crate::error::ServerError;
use axum::{Json, Router, routing::post};
use axum_valid::Garde;
use golion_contract::optimization::{OptimizationInput, OptimizationOutput};
use golion_optimization::ProblemVariables;

pub fn router() -> Router {
    Router::new().route("/optimize", post(handler))
}

/// Builds the LP-facing components for every asset in the request. Actually
/// solving the resulting optimization problem isn't wired up yet (no
/// objective function exists in golion-optimization), so this only proves
/// out the conversion pipeline end to end.
async fn handler(
    Garde(Json(input)): Garde<Json<OptimizationInput>>,
) -> Result<Json<OptimizationOutput>, ServerError> {
    let mut vars = ProblemVariables::new();
    let components = assembly::build_portfolio(&input, &mut vars)?;
    println!("{:#?}", components);

    Ok(Json(OptimizationOutput {
        components_built: components.physical.len(),
        wholesale_perimeters_built: components.wholesale_perimeters.len(),
        ancillary_perimeters_built: components
            .ancillary_perimeter
            .reserve_perimeters()
            .len(),
    }))
}
