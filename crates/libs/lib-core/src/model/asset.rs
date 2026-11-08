use crate::ctx::Ctx;
use crate::model::ModelManager;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
// region:      --- Asset Types
#[derive(Debug, Serialize, Clone, FromRow)]
pub struct Asset {
    pub id: i64,
    pub name: String,
    pub country: String,
    pub bsp_id: Option<i64>,
}
#[derive(Deserialize, Clone)]
pub struct AssetForCreate {
    pub name: String,
    pub country: String,
}

#[derive(Deserialize, Clone)]
pub struct AssetForUpdate {
    pub name: Option<String>,
    pub country: Option<String>,
}
// endregion:      --- Asset Types

// region:         --- Asset Controllers

pub struct AssetControllers;

impl AssetControllers {
    pub async fn create(
        _ctx: &Ctx,
        mm: &ModelManager,
        asset_c: AssetForCreate,
    ) -> crate::Result<i64> {
        let db = mm.db();
        let (id,) = sqlx::query_as::<_, (i64,)>(
            "INSERT INTO asset (name,country) VALUES ($1,$2) returning id",
        )
        .bind(asset_c.name)
        .bind(asset_c.country)
        .fetch_one(db)
        .await?;
        Ok(id)
    }
    pub async fn get(_ctx: &Ctx, mm: &ModelManager, id: i64) -> crate::Result<Asset> {
        let db = mm.db();
        let asset: Asset = sqlx::query_as("SELECT * from asset where id=$1")
            .bind(id)
            .fetch_optional(db)
            .await?
            .ok_or(crate::Error::EntityNotFound { name: "asset", id })?;
        Ok(asset)
    }
    pub async fn delete(_ctx: &Ctx, mm: &ModelManager, id: i64) -> crate::Result<()> {
        let db = mm.db();
        let count = sqlx::query("DELETE from asset where id=$1")
            .bind(id)
            .execute(db)
            .await?
            .rows_affected();
        if count == 0 {
            Err(crate::Error::EntityNotFound { name: "asset", id })
        } else {
            Ok(())
        }
    }
}

// endregion:      --- Asset Controllers

// region:          --- Tests
#[cfg(test)]
mod tests {
    use super::*;
    use crate::_dev_utils;
    use anyhow::Result;
    #[serial_test::serial]
    #[tokio::test]
    async fn test_create_asset_ok() -> Result<()> {
        let mm = _dev_utils::init_test().await;
        let ctx = Ctx::root_user();
        let fx_name = "test_create_asset_ok name";
        let fx_country = "FRA";
        let asset_c =
            AssetForCreate { name: fx_name.to_string(), country: fx_country.to_string() };
        let id = AssetControllers::create(&ctx, &mm, asset_c).await?;
        // -- Check
        let asset = AssetControllers::get(&ctx, &mm, id).await?;
        assert_eq!(asset.name, fx_name);
        assert_eq!(asset.country, fx_country);

        // -- Clean
        AssetControllers::delete(&ctx, &mm, id).await?;
        Ok(())
    }
    #[serial_test::serial]
    #[tokio::test]
    async fn test_get_err_not_found() -> Result<()> {
        let mm = _dev_utils::init_test().await;
        let ctx = Ctx::root_user();
        // ids start at 1000 so this id is not to be found
        let asset = AssetControllers::get(&ctx, &mm, 1).await;
        assert!(matches!(
            asset,
            Err(crate::Error::EntityNotFound { name: "asset", id: 1 })
        ));
        Ok(())
    }
    #[serial_test::serial]
    #[tokio::test]
    async fn test_delete_err_not_found() -> Result<()> {
        let mm = _dev_utils::init_test().await;
        let ctx = Ctx::root_user();
        // ids start at 1000 so this id is not to be found
        let asset = AssetControllers::delete(&ctx, &mm, 1).await;
        assert!(matches!(
            asset,
            Err(crate::Error::EntityNotFound { name: "asset", id: 1 })
        ));
        Ok(())
    }
}

// endregion:       --- Tests
