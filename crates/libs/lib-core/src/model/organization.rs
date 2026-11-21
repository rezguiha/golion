use crate::generate_common_crud_controllers;
use crate::model::ModelManager;
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
    fn is_org_scoped() -> bool {
        false
    }
}
// This will generate the `impl OrganizationControllers {...}` with the default CRUD functions.
generate_common_crud_controllers!(ModController: OrganizationControllers,
Entity:Organization,
ForCreate:OrganizationForCreate,
ForUpdate:OrganizationForUpdate,);
// endregion:      --- Organization Controllers
