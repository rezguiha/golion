pub mod bess;
pub mod ccgt;
pub mod ren;
pub mod variables;

use crate::physical::{bess::core::Battery, ccgt::GasTurbine, ren::Renewable};
use std::collections::HashMap;
use uuid::Uuid;
// region: Asset Enum
#[derive(Debug)]
pub enum Asset {
    Bess(Battery),
    Ccgt(GasTurbine),
    Ren(Renewable),
}

impl From<Battery> for Asset {
    fn from(value: Battery) -> Self {
        Self::Bess(value)
    }
}
impl From<GasTurbine> for Asset {
    fn from(value: GasTurbine) -> Self {
        Self::Ccgt(value)
    }
}
impl From<Renewable> for Asset {
    fn from(value: Renewable) -> Self {
        Self::Ren(value)
    }
}

// endregion: Asset Enum

// region: Asset store
#[derive(Debug)]
pub enum PhysicalError {
    MissingAssetInStore { asset_id: Uuid },
}

#[derive(Debug)]
pub struct PhysicalStore(HashMap<Uuid, Asset>);
impl PhysicalStore {
    pub fn new(store: HashMap<Uuid, Asset>) -> Self {
        Self(store)
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    pub fn get(&self, asset_id: &Uuid) -> crate::Result<&Asset> {
        self.0.get(asset_id).ok_or_else(|| {
            PhysicalError::MissingAssetInStore { asset_id: *asset_id }.into()
        })
    }
}
// endregion: Asset store
