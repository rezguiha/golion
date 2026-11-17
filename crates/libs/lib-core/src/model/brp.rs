use crate::model::{ModelManager, base};
use crate::{ctx::Ctx, model::base::BaseControllers};
use modql::field::Fields;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
// region:      --- Brp Types
#[derive(Debug, Serialize, FromRow, Fields)]
pub struct Brp {
    pub id: i64,
    pub name: String,
}
#[derive(Deserialize, Fields)]
pub struct BrpForCreate {
    pub name: String,
    pub country: String,
}
#[derive(Deserialize, Fields)]
pub(in crate::model) struct BrpForInsert {
    pub name: String,
    pub country: String,
    pub org_id: i64,
}

impl BrpForInsert {
    fn from_brp_for_create(ctx: &Ctx, brp_c: BrpForCreate) -> Self {
        Self { name: brp_c.name, country: brp_c.country, org_id: ctx.org_id() }
    }
}
#[derive(Deserialize, Fields)]
pub struct BrpForUpdate {
    pub name: Option<String>,
}
// endregion:      --- Brp Types

// region:         --- Brp Controllers

pub struct BrpControllers;
impl BaseControllers for BrpControllers {
    const TABLE_NAME: &'static str = "brp";
}
impl BrpControllers {
    pub async fn create(
        ctx: &Ctx,
        mm: &ModelManager,
        brp_c: BrpForCreate,
    ) -> crate::Result<i64> {
        base::create::<Self, _>(ctx, mm, BrpForInsert::from_brp_for_create(ctx, brp_c))
            .await
    }
    pub async fn get(ctx: &Ctx, mm: &ModelManager, id: i64) -> crate::Result<Brp> {
        base::get::<Self, Brp>(ctx, mm, id).await
    }
    pub async fn list(ctx: &Ctx, mm: &ModelManager) -> crate::Result<Vec<Brp>> {
        base::list::<Self, Brp>(ctx, mm).await
    }
    pub async fn delete(ctx: &Ctx, mm: &ModelManager, id: i64) -> crate::Result<()> {
        base::delete::<Self>(ctx, mm, id).await
    }
    pub async fn update(
        ctx: &Ctx,
        mm: &ModelManager,
        id: i64,
        brp_u: BrpForUpdate,
    ) -> crate::Result<()> {
        base::update::<Self, BrpForUpdate>(ctx, mm, id, brp_u).await
    }
}

// endregion:         --- Brp Controllers

// region:          --- Tests
#[cfg(test)]
mod tests {
    use super::{BrpControllers, BrpForCreate, BrpForUpdate};
    use crate::_dev_utils::{self, seed_brp};
    use crate::ctx::Ctx;
    use anyhow::Result;

    #[serial_test::serial]
    #[tokio::test]
    async fn test_create_brp_ok() -> Result<()> {
        let mm = _dev_utils::init_test().await;
        let ctx = Ctx::root_user();
        let fx_name = "test_create_brp_ok name";
        let fx_country = "FRA";
        let brp_c =
            BrpForCreate { name: fx_name.to_string(), country: fx_country.to_string() };
        let id = BrpControllers::create(&ctx, &mm, brp_c).await?;
        // -- Check
        let brp = BrpControllers::get(&ctx, &mm, id).await?;
        assert_eq!(brp.id, id);
        assert_eq!(brp.name, fx_name);

        // -- Clean
        BrpControllers::delete(&ctx, &mm, id).await?;
        Ok(())
    }
    #[serial_test::serial]
    #[tokio::test]
    async fn test_get_brp_err_not_found() -> Result<()> {
        let mm = _dev_utils::init_test().await;
        let ctx = Ctx::root_user();
        // ids start at 1000 so this id is not to be found
        let brp = BrpControllers::get(&ctx, &mm, 1).await;
        assert!(matches!(brp, Err(crate::Error::EntityNotFound { name: "brp", id: 1 })));
        Ok(())
    }
    #[serial_test::serial]
    #[tokio::test]
    async fn test_brp_list_ok() -> Result<()> {
        let mm = _dev_utils::init_test().await;
        let ctx = Ctx::root_user();
        let fx_names = &["test_brp_list_ok 01", "test_brp_list_ok 02"];
        for name in fx_names {
            seed_brp(&ctx, &mm, name).await?;
        }
        let brps = BrpControllers::list(&ctx, &mm).await?;
        let brps: Vec<_> =
            brps.into_iter().filter(|b| b.name.starts_with("test_brp_list_ok")).collect();
        assert_eq!(brps.len(), 2, "number of created brps");
        Ok(())
    }
    #[serial_test::serial]
    #[tokio::test]
    async fn test_delete_brp_err_not_found() -> Result<()> {
        let mm = _dev_utils::init_test().await;
        let ctx = Ctx::root_user();
        // ids start at 1000 so this id is not to be found
        let brp = BrpControllers::delete(&ctx, &mm, 1).await;
        assert!(matches!(brp, Err(crate::Error::EntityNotFound { name: "brp", id: 1 })));
        Ok(())
    }
    #[serial_test::serial]
    #[tokio::test]
    async fn test_update_brp_ok() -> Result<()> {
        let mm = _dev_utils::init_test().await;
        let ctx = Ctx::root_user();
        let fx_name = "test_update_brp_ok - brp_1";
        let fx_name_new = "test_update_brp_ok - brp_1 - new";
        let fx_brp_id = seed_brp(&ctx, &mm, fx_name).await?;
        let brp_u = BrpForUpdate { name: fx_name_new.to_string().into() };
        BrpControllers::update(&ctx, &mm, fx_brp_id, brp_u).await?;
        let brp = BrpControllers::get(&ctx, &mm, fx_brp_id).await?;
        assert_eq!(&brp.name, fx_name_new);
        Ok(())
    }
}

// endregion:       --- Tests
