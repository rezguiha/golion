pub mod bess;
pub mod ccgt;
pub mod ren;
pub mod variables;
use crate::model::BuildEnv;
use crate::physical::{bess::core::Battery, ccgt::GasTurbine, ren::Renewable};
use golion_domain::problem::definition::AssetDefinition;
use good_lp::{Constraint, Expression, IntoAffineExpression, ProblemVariables};
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
    /// Active charge power in kW.
    pub fn input_power_at(&self, dt: &Timestamp) -> crate::Result<Expression> {
        Ok(match self {
            Self::Bess(b) => b.variable_store.at(dt)?.input_power.into_expression(),
            // Non-storage assets never charge.
            Self::Ccgt(_) | Self::Ren(_) => 0.0.into_expression(),
        })
    }
    /// Active discharge power in kW.
    pub fn output_power_at(&self, dt: &Timestamp) -> crate::Result<Expression> {
        Ok(match self {
            Self::Bess(b) => b.variable_store.at(dt)?.output_power.into_expression(),
            Self::Ccgt(c) => c.variable_store.at(dt)?.output_power.into_expression(),
            Self::Ren(r) => r.variable_store.at(dt)?.output_power.into_expression(),
        })
    }
    /// Asset level ancillary signal for charge in kW.
    /// Represents the how much we reserve on asset to deliver
    /// the perimeter commitments and bids.
    pub fn ancillary_input_power_at(&self, dt: &Timestamp) -> crate::Result<Expression> {
        Ok(match self {
            Self::Bess(b) => b.variable_store.at(dt)?.input_ancillary.into_expression(),
            // Non-storage assets never charge.
            Self::Ccgt(_) | Self::Ren(_) => 0.0.into_expression(),
        })
    }
    /// Asset level ancillary signal for discharge in kW.
    /// Represents the how much we reserve on asset to deliver
    /// the perimeter commitments and bids.
    pub fn ancillary_output_power_at(&self, dt: &Timestamp) -> crate::Result<Expression> {
        Ok(match self {
            Self::Bess(b) => b.variable_store.at(dt)?.output_ancillary.into_expression(),
            Self::Ccgt(c) => c.variable_store.at(dt)?.output_ancillary.into_expression(),
            Self::Ren(r) => r.variable_store.at(dt)?.output_ancillary.into_expression(),
        })
    }
    /// Moves  asset physical constraints out leaving it empty
    pub(crate) fn take_constraints(&mut self) -> Vec<Constraint> {
        match self {
            Self::Bess(b) => std::mem::take(&mut b.constraints),
            Self::Ccgt(c) => std::mem::take(&mut c.constraints),
            Self::Ren(r) => std::mem::take(&mut r.constraints),
        }
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
    /// Moves all asset physical constraints out into an iterator leaving each one empty.
    pub(crate) fn take_constraints(&mut self) -> impl Iterator<Item = Constraint> {
        self.0.values_mut().flat_map(|asset| asset.take_constraints())
    }
}
// endregion: Asset store
