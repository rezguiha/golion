use uuid::Uuid;

#[derive(Debug)]
pub enum ProblemError {
    /// A perimeter references assets that are not part of the problem.
    UnknownAssetsInPerimeter { perimeter_id: Uuid, asset_ids: Vec<Uuid> },
    /// An asset can be in at most one balance responsible party perimeter.
    AssetInMultipleBrps { asset_ids: Vec<Uuid> },
}
