use chrono::{DateTime, Utc};
use uuid::Uuid;
/// Hash key to access physical variables
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct PhysicalKey {
    asset_id: Uuid,
    time: DateTime<Utc>,
}
