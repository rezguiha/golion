mod dev_db;
use tokio::sync::OnceCell;
use tracing::info;

use crate::{
    ctx::Ctx,
    model::{
        ModelManager,
        asset::{Asset, AssetControllers, AssetForCreate},
    },
};

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
    static INIT: OnceCell<ModelManager> = OnceCell::const_new();
    let mm = INIT
        .get_or_init(|| async {
            init_dev().await;
            ModelManager::try_new().await.unwrap()
        })
        .await;
    mm.clone()
}

pub async fn seed_assets(
    ctx: &Ctx,
    mm: &ModelManager,
    names: &[&str],
) -> crate::Result<Vec<Asset>> {
    let mut assets = Vec::new();
    for name in names {
        let asset_id = AssetControllers::create(
            ctx,
            mm,
            AssetForCreate { name: name.to_string(), country: "FRA".to_string() },
        )
        .await?;
        assets.push(AssetControllers::get(ctx, mm, asset_id).await?)
    }
    Ok(assets)
}
