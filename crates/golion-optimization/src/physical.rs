pub mod bess;
pub mod ccgt;
pub mod ren;
pub mod variables;
use crate::model::BuildEnv;
use crate::physical::{bess::core::Battery, ccgt::GasTurbine, ren::Renewable};
use golion_domain::problem::definition::AssetDefinition;
use good_lp::{Expression, IntoAffineExpression, ProblemVariables};
use jiff::Timestamp;
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

impl Asset {
    pub fn input_power_at(&self, dt: &Timestamp) -> crate::Result<Expression> {
        Ok(match self {
            Self::Bess(b) => b.variable_store.at(dt)?.input_power.into_expression(),
            // Non-storage assets never charge.
            Self::Ccgt(_) | Self::Ren(_) => 0.0.into_expression(),
        })
    }
    pub fn output_power_at(&self, dt: &Timestamp) -> crate::Result<Expression> {
        Ok(match self {
            Self::Bess(b) => b.variable_store.at(dt)?.output_power.into_expression(),
            Self::Ccgt(c) => c.variable_store.at(dt)?.output_power.into_expression(),
            Self::Ren(r) => r.variable_store.at(dt)?.output_power.into_expression(),
        })
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
    /// Builds the physical variables and constraints of every asset.
    pub(crate) fn try_new(
        assets: &HashMap<Uuid, AssetDefinition>,
        env: &BuildEnv<'_>,
        vars: &mut ProblemVariables,
    ) -> crate::Result<Self> {
        let store = assets
            .iter()
            .map(|(asset_id, definition)| {
                let asset = match definition {
                    AssetDefinition::Bess { specifications, initial_soc } => {
                        let step = *specifications.limits.availability.grid().step();
                        let battery = Battery::new(
                            env.horizon().timestamps(),
                            vars,
                            specifications,
                            *initial_soc,
                            step,
                        )?;
                        Asset::from(battery)
                    }
                };
                Ok((*asset_id, asset))
            })
            .collect::<crate::Result<_>>()?;
        Ok(Self(store))
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
    pub fn iter(&self) -> impl Iterator<Item = (&Uuid, &Asset)> {
        self.0.iter()
    }
}
// endregion: Asset store
