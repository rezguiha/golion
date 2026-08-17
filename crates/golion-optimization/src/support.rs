use good_lp::{Expression, IntoAffineExpression};
/// Support Methods and Definitions for handlers.
use jiff::SignedDuration;
#[inline]
pub(crate) fn power_to_energy(
    v: impl IntoAffineExpression,
    duration: &SignedDuration,
) -> Expression {
    (duration.as_secs_f64() / 3600.0) * v.into_expression()
}
