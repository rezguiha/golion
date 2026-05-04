use super::core::{BatteryVars, BiddingVars, GeneratorVars};
use super::keys::{BiddingKey, PhysicalKey};
/// Variable Stores serve the purpose of having quick
/// single row search by a well defined hash key for physical variables
/// that will be needed to be linked with other components like
/// bidding variables to define constraints linking both. It also facilitates
/// solved variables retrieve from solution returned by good_lp.
use crate::optimization::error::StoreError;
use std::collections::HashMap;
use std::hash::Hash;
// region:     --- Generic Variable Store Definition
#[derive(Debug, Default)]
pub struct VarStore<K, V> {
    store: HashMap<K, V>,
}

impl<K, V> VarStore<K, V>
where
    K: Hash + Eq,
{
    pub fn require(&self, key: &K) -> Result<&V, StoreError>
    where
        K: std::fmt::Debug,
    {
        self.store.get(key).ok_or_else(|| StoreError::MissingKey {
            key: format!("{key:?}"),
        })
    }
    pub fn insert(&mut self, key: K, vars: V) {
        self.store.insert(key, vars);
    }
}
// endregion:     --- Generic Variable Store Definition

// region:     --- Physical Variable Store

pub type BatteryVarStore = VarStore<PhysicalKey, BatteryVars>;
pub type GeneratorVarStore = VarStore<PhysicalKey, GeneratorVars>;

// endregion:     --- Physical Variable Store

// region:     --- Bidding Variable Store

pub type BiddingVarStore = VarStore<BiddingKey, BiddingVars>;

// endregion:     --- Bidding Variable Store
