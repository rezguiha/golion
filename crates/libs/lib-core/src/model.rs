// region:      --- Modules
pub mod asset;
pub mod store;
// endregion:      --- Modules

use crate::model::store::{Db, new_db_pool};

#[derive(Clone)]
pub struct ModelManager {
    db: Db,
}

impl ModelManager {
    pub async fn try_new() -> crate::Result<Self> {
        Ok(Self { db: new_db_pool().await? })
    }
    pub(in crate::model) fn db(&self) -> &Db {
        &self.db
    }
}
