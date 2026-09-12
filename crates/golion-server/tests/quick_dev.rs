use anyhow::Result;
use golion_contract::asset::{
    availability::StorageAvailability,
    core::{AssetData, BessData},
    identification::AssetIdentification,
    specifications::BessSpecs,
};
use golion_contract::market::choice::MarketChoice;
use golion_contract::market::commitments::{AncillaryCommitment, WholesaleCommitment};
use golion_contract::optimization::OptimizationInput;
use golion_domain::countries::Countries;
use golion_domain::market::market_type::{
    AncillaryMarketType, EnergyAncillaryMarketType, WholesaleMarketType,
};
use jiff::{SignedDuration, Timestamp, Unit};
use std::collections::HashMap;
/// Temporary simple test of sending assetdata as a payload
/// on optimize endpoint.
#[tokio::test]
async fn test_optimize_bess() -> Result<()> {
    let hc = httpc_test::new_client("http://127.0.0.1:3000")?;

    // 3 days of 15-minute slots = 288 periods
    let n = 3 * 24 * 4;
    let start = Timestamp::now().round((Unit::Minute, 15))?;
    let timestamps: Vec<Timestamp> =
        std::iter::successors(Some(start), |t| Some(*t + SignedDuration::from_mins(15)))
            .take(n)
            .collect();
    let market_choices = vec![
        MarketChoice {
            market: WholesaleMarketType::SpotDayAhead.into(),
            country: Countries::FR,
            product_step_minutes: 15,
            product_increment_kw: 10,
        },
        MarketChoice {
            market: EnergyAncillaryMarketType::AfrrFree.into(),
            country: Countries::BE,
            product_step_minutes: 15,
            product_increment_kw: 1000,
        },
    ];
    let availability = timestamps
        .iter()
        .map(|t| {
            StorageAvailability::builder()
                .start_at(*t)
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
                    .start_at(*t)
                    .upward_power(10.0)
                    .downward_power(10.0)
                    .build()
            })
            .collect(),
    )]);
    let wholesale_commitments = HashMap::from([(
        WholesaleMarketType::IntradayAuction1,
        timestamps
            .iter()
            .map(|t| {
                WholesaleCommitment::builder().start_at(*t).net_position(2.6).build()
            })
            .collect(),
    )]);
    let asset = AssetData::Bess(
        BessData::builder()
            .availability(availability)
            .initial_soc(50.0)
            .specs(BessSpecs::builder().build())
            .identification(AssetIdentification::builder().build())
            .market_choices(market_choices)
            .ancillary_commitments(ancillary_commitments)
            .wholesale_commitments(wholesale_commitments)
            .build(),
    );

    let input = OptimizationInput {
        optimization_start_at: start,
        optimization_end_at: *timestamps.last().unwrap(),
        optimization_step: SignedDuration::from_mins(15),
        assets: vec![asset],
        ancillary_markets: vec![],
        wholesale_markets: vec![],
    };

    hc.do_post("/optimize", serde_json::to_value(input)?).await?.print().await?;

    Ok(())
}
