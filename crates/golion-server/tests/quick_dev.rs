use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use golion_contract::asset::{
    availability::StorageAvailability,
    core::{AssetData, BessData},
    identification::AssetIdentification,
    specifications::BessSpecs,
};
use golion_contract::market::choice::MarketChoice;
use golion_contract::market::commitments::{AncillaryCommitment, WholesaleCommitment};
use golion_domain::countries::Countries;
use golion_domain::market::market_type::{
    AncillaryMarketType, EnergyAncillaryMarketType, WholesaleMarketType,
};
use std::collections::HashMap;
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
        MarketChoice::EnergyAncillaryChoice {
            market: EnergyAncillaryMarketType::AfrrFree,
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

    let ancillary_commitments = HashMap::from([(
        AncillaryMarketType::Energy(EnergyAncillaryMarketType::AfrrFree),
        timestamps
            .iter()
            .map(|t| {
                AncillaryCommitment::builder()
                    .start_at(t.to_utc())
                    .upward_power(10.0)
                    .downward_power(10.0)
                    .build()
            })
            .collect(),
    )]);
    let wholesale_commitments = HashMap::from([(
        WholesaleMarketType::IntradayAuction,
        timestamps
            .iter()
            .map(|t| {
                WholesaleCommitment::builder()
                    .start_at(t.to_utc())
                    .net_position(2.6)
                    .build()
            })
            .collect(),
    )]);
    let asset = AssetData::Bess(
        BessData::builder()
            .availability(availability)
            .specs(BessSpecs::builder().build())
            .identification(AssetIdentification::builder().build())
            .market_choices(market_choices)
            .ancillary_commitments(ancillary_commitments)
            .wholesale_commitments(wholesale_commitments)
            .build(),
    );

    hc.do_post("/optimize", serde_json::to_value(asset)?).await?.print().await?;

    Ok(())
}
