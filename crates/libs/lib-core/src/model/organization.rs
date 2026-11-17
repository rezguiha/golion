use crate::model::{ModelManager, base};
use crate::{ctx::Ctx, model::base::BaseControllers};
use modql::field::Fields;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
// region:      --- Organization Types
#[derive(Debug, Serialize, FromRow, Fields)]
pub struct Organization {
    pub id: i64,
    pub name: String,
}
#[derive(Deserialize, Fields)]
pub struct OrganizationForCreate {
    pub name: String,
}

#[derive(Deserialize, Fields)]
pub struct OrganizationForUpdate {
    pub name: Option<String>,
}
// endregion:      --- Organization Types

// region:         --- Organization Controllers

pub struct OrganizationControllers;
impl BaseControllers for OrganizationControllers {
    const TABLE_NAME: &'static str = "organization";
}
impl OrganizationControllers {
    pub async fn create(
        ctx: &Ctx,
        mm: &ModelManager,
        org_c: OrganizationForCreate,
    ) -> crate::Result<i64> {
        base::create::<Self, _>(ctx, mm, org_c).await
    }
    pub async fn get(
        ctx: &Ctx,
        mm: &ModelManager,
        id: i64,
    ) -> crate::Result<Organization> {
        base::get::<Self, Organization>(ctx, mm, id).await
    }
    pub async fn list(ctx: &Ctx, mm: &ModelManager) -> crate::Result<Vec<Organization>> {
        base::list::<Self, Organization>(ctx, mm).await
    }
    pub async fn delete(ctx: &Ctx, mm: &ModelManager, id: i64) -> crate::Result<()> {
        base::delete::<Self>(ctx, mm, id).await
    }
    pub async fn update(
        ctx: &Ctx,
        mm: &ModelManager,
        id: i64,
        org_u: OrganizationForUpdate,
    ) -> crate::Result<()> {
        base::update::<Self, OrganizationForUpdate>(ctx, mm, id, org_u).await
    }
}

// endregion:      --- Organization Controllers
