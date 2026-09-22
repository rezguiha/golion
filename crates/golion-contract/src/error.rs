use derive_more::From;
use uuid::Uuid;
#[derive(Debug, From)]
pub enum Error {
    /// The payload contains an asset type the domain cannot model yet,
    /// as the solution only supports batteries for now.
    UnsupportedAssetType { asset_id: Uuid },
    // --- External
    #[from]
    Domain(golion_domain::Error),
}

pub type Result<T> = core::result::Result<T, Error>;
