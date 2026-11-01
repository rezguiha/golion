/// Support Methods and Definitions for handlers.
use good_lp::{Expression, IntoAffineExpression};
use lib_domain::temporal::step::Step;
#[inline]
pub(crate) fn power_to_energy(v: impl IntoAffineExpression, step: &Step) -> Expression {
    step.total_hours_f64() * v.into_expression()
}
