use garde::Validate;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;
use uuid::Uuid;
// region: Asset Identification
#[derive(Debug, Serialize, Deserialize, Validate, TypedBuilder)]
pub struct AssetIdentification {
    /// The asset unique identification.
    #[builder(default=Uuid::new_v4())]
    #[garde(skip)]
    pub asset_id: Uuid,
    /// The connection point identification which will serve
    /// as a grouping to model grid facing limitations/constraints.
    #[builder(default=Uuid::new_v4())]
    #[garde(skip)]
    pub connection_point_id: Uuid,
    /// The balancing service provider id which will serve as a grouping
    /// to model ancillary services facing interface. (FCR, aFRR, mFRR)
    #[builder(default=Uuid::new_v4())]
    #[garde(skip)]
    pub bsp_id: Uuid,

    /// The balancing role party id which will serve as a grouping
    /// to model imbalance facing interface.(Day ahead, intraday ,imbalance)
    #[builder(default=Uuid::new_v4())]
    #[garde(skip)]
    pub brp_id: Uuid,
}
// endregion: Asset Identification
