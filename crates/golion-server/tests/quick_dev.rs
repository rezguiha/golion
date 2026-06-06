use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use golion_domain::asset::{
    availability::StorageAvailability,
    core::{AssetData, AssetIdentification, BessData},
    specifications::BessSpecs,
};
use golion_domain::constants::{
    Countries, UncertifiedAncillaryMarketType, WholesaleMarketType,
};
use golion_domain::market::choice::MarketChoice;
/// Temporary simple test of sending assetdata as a payload
/// on optimize endpoint.
#[tokio::test]
async fn test_optimize_bess() -> Result<()> {
    let hc = httpc_test::new_client("http://127.0.0.1:3000")?;

    // 3 days of 15-minute slots = 288 periods
    let n = 3 * 24 * 4;
    let start = Utc::now();
    let timestamps: Vec<DateTime<Utc>> =
        std::iter::successors(Some(start), |t| Some(*t + Duration::minutes(15)))
            .take(n)
            .collect();
    let market_choices = vec![
        MarketChoice::WholeSaleChoice {
            market: WholesaleMarketType::SpotDayAhead,
            country: Countries::FR,
        },
        MarketChoice::UncertifiedAncillaryChoice {
            market: UncertifiedAncillaryMarketType::AfrrFree,
            country: Countries::BE,
        },
    ];
    let availability = timestamps
        .iter()
        .map(|t| {
            StorageAvailability::builder()
                .start_at(t.to_utc())
                .max_charge_power(20.0)
                .max_discharge_power(20.0)
                .max_usable_energy(100.0)
                .build()
        })
        .collect();
    let asset = AssetData::Bess(
        BessData::builder()
            .availability(availability)
            .specs(BessSpecs::builder().build())
            .identification(AssetIdentification::builder().build())
            .market_choices(market_choices)
            .build(),
    );

    hc.do_post("/optimize", serde_json::to_value(asset)?).await?.print().await?;

    Ok(())
}
