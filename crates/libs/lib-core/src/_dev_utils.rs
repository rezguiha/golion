mod dev_db;
use tokio::sync::OnceCell;
use tracing::info;

use crate::{
    ctx::Ctx,
    model::{
        ModelManager,
        asset::{Asset, AssetControllers, AssetForCreate},
        brp::{BrpControllers, BrpForCreate},
        organization::{OrganizationControllers, OrganizationForCreate},
    },
};
const COUNTRY_DEV: &str = "FRA";

/// Initialize local dev environment.
pub async fn init_dev() {
    static INIT: OnceCell<()> = OnceCell::const_new();
    INIT.get_or_init(|| async {
        info!("{:<12} - init_dev_all()", "FOR-DEV-ONLY");
        dev_db::init_dev_db().await.unwrap();
    })
    .await;
}
/// Initialize test environment.
pub async fn init_test() -> ModelManager {
    init_dev().await;
    ModelManager::try_new().await.unwrap()
}

pub async fn seed_assets(
    ctx: &Ctx,
    mm: &ModelManager,
    names: &[&str],
    brp_id: i64,
) -> crate::Result<Vec<Asset>> {
    let mut assets = Vec::new();
    for name in names {
        let asset_id = AssetControllers::create(
            ctx,
            mm,
            AssetForCreate {
                name: name.to_string(),
                country: COUNTRY_DEV.to_string(),
                brp_id,
            },
        )
        .await?;
        assets.push(AssetControllers::get(ctx, mm, asset_id).await?)
    }
    Ok(assets)
}

pub async fn seed_organization(
    ctx: &Ctx,
    mm: &ModelManager,
    name: &str,
) -> crate::Result<i64> {
    OrganizationControllers::create(
        ctx,
        mm,
        OrganizationForCreate { name: name.to_string() },
    )
    .await
}

pub async fn seed_brp(ctx: &Ctx, mm: &ModelManager, name: &str) -> crate::Result<i64> {
    BrpControllers::create(
        ctx,
        mm,
        BrpForCreate { name: name.to_string(), country: COUNTRY_DEV.to_string() },
    )
    .await
}
