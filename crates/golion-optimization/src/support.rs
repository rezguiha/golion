/// Support Methods and Definitions for handlers.
use chrono::Duration;
use good_lp::{Expression, IntoAffineExpression};
#[inline]
pub(crate) fn power_to_energy(
    v: impl IntoAffineExpression,
    duration: Duration,
) -> Expression {
    (duration.as_seconds_f64() / 3600.0) * v.into_expression()
}
