/// Support Methods and Definitions for handlers.
use chrono::Duration;
use good_lp::{Expression, Variable};
#[inline]
pub(crate) fn power_to_energy(v: Variable, duration: Duration) -> Expression {
    (duration.as_seconds_f64() / 3600.0) * v
}
