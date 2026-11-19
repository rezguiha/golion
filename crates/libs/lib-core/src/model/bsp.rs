use crate::generate_common_crud_controllers;
use crate::model::{ModelManager, base};
use crate::{ctx::Ctx, model::base::BaseControllers};
use modql::field::Fields;
use serde::{Deserialize, Serialize};
use sqlx::{self, FromRow};
// region:      --- Bsp Types
#[derive(sqlx::Type, Debug, Serialize, Deserialize, derive_more::Display)]
#[sqlx(type_name = "ancillary_service_type")]
pub enum AncillaryServiceType {
    Fcr,
    Afrr,
    AfrrFree,
}

impl From<AncillaryServiceType> for sea_query::Value {
    fn from(value: AncillaryServiceType) -> Self {
        value.to_string().into()
    }
}
#[derive(Debug, Serialize, FromRow, Fields)]
pub struct Bsp {
    pub id: i64,
    pub name: String,
    pub country: String,
    #[field(cast_as = "ancillary_service_type")]
    pub service_type: AncillaryServiceType,
}
#[derive(Deserialize, Fields)]
pub struct BspForCreate {
    pub name: String,
    pub country: String,
    #[field(cast_as = "ancillary_service_type")]
    pub service_type: AncillaryServiceType,
}
#[derive(Deserialize, Fields)]
pub(in crate::model) struct BspForInsert {
    pub name: String,
    pub country: String,
    #[field(cast_as = "ancillary_service_type")]
    pub service_type: AncillaryServiceType,
    pub org_id: i64,
}

impl BspForInsert {
    fn from_bsp_for_create(ctx: &Ctx, bsp_c: BspForCreate) -> Self {
        Self {
            name: bsp_c.name,
            country: bsp_c.country,
            service_type: bsp_c.service_type,
            org_id: ctx.org_id(),
        }
    }
}

#[derive(Deserialize, Fields)]
pub struct BspForUpdate {
    pub name: Option<String>,
}
// endregion:      --- Bsp Types

// region:         --- Bsp Controllers

pub struct BspControllers;
impl BaseControllers for BspControllers {
    const TABLE_NAME: &'static str = "bsp";
    fn is_org_scoped() -> bool {
        true
    }
}
// This will generate the `impl BspControllers {...}` with the default CRUD functions.
generate_common_crud_controllers!(
    ModController: BspControllers,
    Entity:Bsp,
    ForUpdate:BspForUpdate,
);
impl BspControllers {
    pub async fn create(
        ctx: &Ctx,
        mm: &ModelManager,
        brp_c: BspForCreate,
    ) -> crate::Result<i64> {
        base::create::<Self, _>(ctx, mm, BspForInsert::from_bsp_for_create(ctx, brp_c))
            .await
    }
}

// endregion:         --- Bsp Controllers

// region:          --- Tests
#[cfg(test)]
mod tests {
    use super::{BspControllers, BspForCreate, BspForUpdate};
    use crate::_dev_utils::{self, seed_bsp};
    use crate::ctx::Ctx;
    use crate::model::bsp::AncillaryServiceType;
    use anyhow::Result;

    #[serial_test::serial]
    #[tokio::test]
    async fn test_create_bsp_ok() -> Result<()> {
        let mm = _dev_utils::init_test().await;
        let ctx = Ctx::root_user();
        let fx_name = "test_create_bsp_ok name";
        let fx_country = "FRA";
        let bsp_c = BspForCreate {
            name: fx_name.to_string(),
            country: fx_country.to_string(),
            service_type: AncillaryServiceType::AfrrFree,
        };
        let id = BspControllers::create(&ctx, &mm, bsp_c).await?;
        // -- Check
        let bsp = BspControllers::get(&ctx, &mm, id).await?;
        assert_eq!(bsp.id, id);
        assert_eq!(bsp.name, fx_name);

        // -- Clean
        BspControllers::delete(&ctx, &mm, id).await?;
        Ok(())
    }
    #[serial_test::serial]
    #[tokio::test]
    async fn test_get_bsp_err_not_found() -> Result<()> {
        let mm = _dev_utils::init_test().await;
        let ctx = Ctx::root_user();
        // ids start at 1000 so this id is not to be found
        let bsp = BspControllers::get(&ctx, &mm, 1).await;
        assert!(matches!(bsp, Err(crate::Error::EntityNotFound { name: "bsp", id: 1 })));
        Ok(())
    }
    #[serial_test::serial]
    #[tokio::test]
    async fn test_bsp_list_ok() -> Result<()> {
        let mm = _dev_utils::init_test().await;
        let ctx = Ctx::root_user();
        let fx_names = &["test_bsp_list_ok 01", "test_bsp_list_ok 02"];
        let fx_service_types = [AncillaryServiceType::Afrr, AncillaryServiceType::Fcr];
        for (name, service_type) in fx_names.iter().zip(fx_service_types) {
            seed_bsp(&ctx, &mm, name, service_type).await?;
        }
        let bsps = BspControllers::list(&ctx, &mm).await?;
        let bsps: Vec<_> =
            bsps.into_iter().filter(|b| b.name.starts_with("test_bsp_list_ok")).collect();
        assert_eq!(bsps.len(), 2, "number of created bsps");
        Ok(())
    }
    #[serial_test::serial]
    #[tokio::test]
    async fn test_delete_bsp_err_not_found() -> Result<()> {
        let mm = _dev_utils::init_test().await;
        let ctx = Ctx::root_user();
        // ids start at 1000 so this id is not to be found
        let bsp = BspControllers::delete(&ctx, &mm, 1).await;
        assert!(matches!(bsp, Err(crate::Error::EntityNotFound { name: "bsp", id: 1 })));
        Ok(())
    }
    #[serial_test::serial]
    #[tokio::test]
    async fn test_update_bsp_ok() -> Result<()> {
        let mm = _dev_utils::init_test().await;
        let ctx = Ctx::root_user();
        let fx_name = "test_update_brp_ok - brp_1";
        let fx_name_new = "test_update_brp_ok - brp_1 - new";
        let fx_service_type = AncillaryServiceType::AfrrFree;
        let fx_bsp_id = seed_bsp(&ctx, &mm, fx_name, fx_service_type).await?;
        let brp_u = BspForUpdate { name: fx_name_new.to_string().into() };
        BspControllers::update(&ctx, &mm, fx_bsp_id, brp_u).await?;
        let bsp = BspControllers::get(&ctx, &mm, fx_bsp_id).await?;
        assert_eq!(&bsp.name, fx_name_new);
        Ok(())
    }
}

// endregion:       --- Tests
