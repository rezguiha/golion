/// Creates associations of assets to a bsp.
use super::bsp::BspIden;
use crate::ctx::Ctx;
use crate::model::ModelManager;
use crate::model::base::BaseControllers;
use crate::model::bsp::{AncillaryServiceType, BspControllers};
use modql::field::{Fields, HasSeaFields};
use sea_query::ExprTrait;
use sea_query_sqlx::SqlxBinder;
use serde::Serialize;
use sqlx::{AssertSqlSafe, FromRow};

use sea_query::{Expr, PostgresQueryBuilder, Query, enum_def};
#[enum_def]
#[derive(Debug, Serialize, FromRow, Fields)]
pub struct AssetBspAssociation {
    pub asset_id: i64,
    pub org_id: i64,
    pub bsp_id: i64,
    pub country: String,
    #[field(cast_as = "ancillary_service_type")]
    pub service_type: AncillaryServiceType,
}
pub struct AssetBspControllers;
impl BaseControllers for AssetBspControllers {
    const TABLE_NAME: &'static str = "asset_bsp_association";
    fn is_org_scoped() -> bool {
        true
    }
}

impl AssetBspControllers {
    pub async fn assign(
        ctx: &Ctx,
        mm: &ModelManager,
        asset_id: i64,
        bsp_id: i64,
    ) -> crate::Result<()> {
        let mut bsp_row = Query::select();

        bsp_row
            .from(BspControllers::TABLE_NAME)
            .expr(Expr::val(asset_id))
            .columns([
                BspIden::OrgId,
                BspIden::Id,
                BspIden::Country,
                BspIden::ServiceType,
            ])
            .and_where(Expr::column(BspIden::Id).eq(bsp_id))
            .and_where(Expr::column(BspIden::OrgId).eq(ctx.org_id()));
        let mut query = Query::insert();
        query
            .into_table(AssetBspAssociationIden::Table)
            .columns([
                AssetBspAssociationIden::AssetId,
                AssetBspAssociationIden::OrgId,
                AssetBspAssociationIden::BspId,
                AssetBspAssociationIden::Country,
                AssetBspAssociationIden::ServiceType,
            ])
            .select_from(bsp_row.take())?;
        let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
        let nb_rows = sqlx::query_with(AssertSqlSafe(sql), values)
            .execute(mm.db())
            .await?
            .rows_affected();
        if nb_rows == 0 {
            Err(crate::Error::EntityNotFound {
                name: BspControllers::TABLE_NAME,
                id: bsp_id,
            })
        } else {
            Ok(())
        }
    }
    pub async fn unassign(
        ctx: &Ctx,
        mm: &ModelManager,
        asset_id: i64,
        bsp_id: i64,
    ) -> crate::Result<()> {
        let mut query = Query::delete();
        query
            .from_table(Self::TABLE_NAME)
            .and_where(Expr::column(AssetBspAssociationIden::AssetId).eq(asset_id))
            .and_where(Expr::column(AssetBspAssociationIden::BspId).eq(bsp_id))
            .and_where(Expr::column(AssetBspAssociationIden::OrgId).eq(ctx.org_id()));
        let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
        let nb_rows = sqlx::query_with(AssertSqlSafe(sql), values)
            .execute(mm.db())
            .await?
            .rows_affected();
        if nb_rows == 0 {
            Err(crate::Error::EntityNotFound {
                name: BspControllers::TABLE_NAME,
                id: bsp_id,
            })
        } else {
            Ok(())
        }
    }
    pub async fn list_asset_associations(
        ctx: &Ctx,
        mm: &ModelManager,
        bsp_id: i64,
    ) -> crate::Result<Vec<AssetBspAssociation>> {
        let mut query = Query::select();
        query
            .from(Self::TABLE_NAME)
            .columns(AssetBspAssociation::sea_column_refs())
            .and_where(Expr::col(AssetBspAssociationIden::BspId).eq(bsp_id))
            .and_where(Expr::col(AssetBspAssociationIden::OrgId).eq(ctx.org_id()));
        let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
        let assets_associations =
            sqlx::query_as_with(AssertSqlSafe(sql), values).fetch_all(mm.db()).await?;
        Ok(assets_associations)
    }
    pub async fn get(
        ctx: &Ctx,
        mm: &ModelManager,
        bsp_id: i64,
        asset_id: i64,
    ) -> crate::Result<AssetBspAssociation> {
        let mut query = Query::select();
        query
            .from(Self::TABLE_NAME)
            .columns(AssetBspAssociation::sea_column_refs())
            .and_where(Expr::col(AssetBspAssociationIden::BspId).eq(bsp_id))
            .and_where(Expr::col(AssetBspAssociationIden::OrgId).eq(ctx.org_id()))
            .and_where(Expr::col(AssetBspAssociationIden::AssetId).eq(asset_id));
        let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
        let asset_association = sqlx::query_as_with(AssertSqlSafe(sql), values)
            .fetch_optional(mm.db())
            .await?
            .ok_or(crate::Error::EntityNotFound {
                name: Self::TABLE_NAME,
                id: asset_id,
            })?;
        Ok(asset_association)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::_dev_utils::{self, seed_assets, seed_brp, seed_bsp};
    use anyhow::Result;
    #[serial_test::serial]
    #[tokio::test]
    async fn test_assign_asset_bsp_ok() -> Result<()> {
        let mm = _dev_utils::init_test().await;
        let ctx = Ctx::root_user();
        let fx_name = "test_create_asset_bsp_ok name";
        let fx_brp_name = "test_create_asset_ok brp_name";
        let fx_brp_id = seed_brp(&ctx, &mm, fx_brp_name).await?;
        let fx_service_type = AncillaryServiceType::Fcr;
        let fx_bsp_id = seed_bsp(&ctx, &mm, fx_name, fx_service_type).await?;
        let fx_asset = seed_assets(&ctx, &mm, &[fx_name], fx_brp_id).await?.remove(0);
        AssetBspControllers::assign(&ctx, &mm, fx_asset.id, fx_bsp_id).await?;
        // Check
        let asset_bsp_association =
            AssetBspControllers::get(&ctx, &mm, fx_bsp_id, fx_asset.id).await?;
        assert_eq!(fx_asset.id, asset_bsp_association.asset_id);
        assert_eq!(fx_bsp_id, asset_bsp_association.bsp_id);
        assert_eq!(fx_service_type, asset_bsp_association.service_type);
        assert_eq!(fx_service_type, asset_bsp_association.service_type);
        // Clean
        AssetBspControllers::unassign(&ctx, &mm, fx_bsp_id, fx_asset.id).await?;
        Ok(())
    }
}
