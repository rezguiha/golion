use golion_domain::temporal::step::Step;
/// Support Methods and Definitions for handlers.
use good_lp::{Expression, IntoAffineExpression};
#[inline]
pub(crate) fn power_to_energy(v: impl IntoAffineExpression, step: &Step) -> Expression {
    step.total_hours_f64() * v.into_expression()
}
