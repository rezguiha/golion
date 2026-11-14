use crate::model::{ModelManager, base};
use crate::{ctx::Ctx, model::base::BaseControllers};
use modql::field::Fields;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
// region:      --- Asset Types
#[derive(Debug, Serialize, FromRow, Fields)]
pub struct Asset {
    pub id: i64,
    pub name: String,
    pub country: String,
    pub bsp_id: Option<i64>,
}
#[derive(Deserialize, Fields)]
pub struct AssetForCreate {
    pub name: String,
    pub country: String,
}

#[derive(Deserialize, Fields)]
pub struct AssetForUpdate {
    pub name: Option<String>,
    pub country: Option<String>,
}
// endregion:      --- Asset Types

// region:         --- Asset Controllers

pub struct AssetControllers;
impl BaseControllers for AssetControllers {
    const TABLE_NAME: &'static str = "asset";
}
impl AssetControllers {
    pub async fn create(
        ctx: &Ctx,
        mm: &ModelManager,
        asset_c: AssetForCreate,
    ) -> crate::Result<i64> {
        base::create::<Self, _>(ctx, mm, asset_c).await
    }
    pub async fn get(ctx: &Ctx, mm: &ModelManager, id: i64) -> crate::Result<Asset> {
        base::get::<Self, Asset>(ctx, mm, id).await
    }
    pub async fn list(ctx: &Ctx, mm: &ModelManager) -> crate::Result<Vec<Asset>> {
        base::list::<Self, Asset>(ctx, mm).await
    }
    pub async fn delete(ctx: &Ctx, mm: &ModelManager, id: i64) -> crate::Result<()> {
        base::delete::<Self>(ctx, mm, id).await
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
    async fn test_get_asset_err_not_found() -> Result<()> {
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
    async fn test_asset_list_ok() -> Result<()> {
        let mm = _dev_utils::init_test().await;
        let ctx = Ctx::root_user();
        let fx_names = &["test_asset_list_ok 01", "test_asset_list_ok 02"];
        _dev_utils::seed_assets(&ctx, &mm, fx_names).await?;
        let assets = AssetControllers::list(&ctx, &mm).await?;
        let assets: Vec<_> = assets
            .into_iter()
            .filter(|a| a.name.starts_with("test_asset_list_ok"))
            .collect();
        assert_eq!(assets.len(), 2, "number of created assets");
        Ok(())
    }
    #[serial_test::serial]
    #[tokio::test]
    async fn test_delete_asset_err_not_found() -> Result<()> {
        let mm = _dev_utils::init_test().await;
        let ctx = Ctx::root_user();
        // ids start at 1000 so this id is not to be found
        let asset = AssetControllers::delete(&ctx, &mm, 1).await;
        println!("{asset:?}");
        assert!(matches!(
            asset,
            Err(crate::Error::EntityNotFound { name: "asset", id: 1 })
        ));
        Ok(())
    }
}

// endregion:       --- Tests
