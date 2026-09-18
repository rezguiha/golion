use anyhow::Result;
use golion_contract::market::choice::MarketChoice;
use golion_contract::market::commitments::{AncillaryCommitment, WholesaleCommitment};
use golion_contract::market::revenue::{AncillaryRevenue, WholesaleRevenue};
use golion_contract::optimization::OptimizationInput;
use golion_contract::perimeter::{
    reserve::ReservePerimeter, wholesale::WholesalePerimeter,
};
use golion_contract::{
    asset::{
        availability::StorageAvailability,
        core::{AssetData, BessData},
        identification::AssetIdentification,
        specifications::BessSpecs,
    },
    market::series::MarketSeries,
};
use golion_domain::countries::Countries;
use golion_domain::market::market_type::{
    AncillaryMarketType, EnergyAncillaryMarketType, WholesaleMarketType,
};
use jiff::{SignedDuration, Timestamp, Unit};
use uuid::Uuid;
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
    let wholesale_market_choices = vec![MarketChoice {
        market: WholesaleMarketType::SpotDayAhead,
        product_step_minutes: 15,
        product_increment_kw: 10,
    }];
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
    let reserve_commitments: Vec<AncillaryCommitment> = timestamps
        .iter()
        .map(|t| {
            AncillaryCommitment::builder()
                .start_at(*t)
                .upward_power(10.0)
                .downward_power(10.0)
                .build()
        })
        .collect();
    let wholesale_commitments = vec![MarketSeries {
        market: WholesaleMarketType::IntradayAuction1,
        values: timestamps
            .iter()
            .map(|t| {
                WholesaleCommitment::builder().start_at(*t).net_position(2.6).build()
            })
            .collect(),
    }];
    let ancillary_revenues = vec![MarketSeries {
        market: AncillaryMarketType::Energy(EnergyAncillaryMarketType::AfrrFree),
        values: timestamps
            .iter()
            .map(|t| AncillaryRevenue::SimplifiedAncillaryRevenue {
                start_at: *t,
                sell_revenue: 15.0,
                buy_revenue: 15.0,
            })
            .collect(),
    }];
    let wholesale_revenues = vec![MarketSeries {
        market: WholesaleMarketType::SpotDayAhead,
        values: timestamps
            .iter()
            .map(|t| WholesaleRevenue::SimplifiedWholesaleRevenue {
                start_at: *t,
                sell_price: 80.0,
                buy_price: 75.0,
            })
            .collect(),
    }];
    let identification = AssetIdentification::builder().build();
    let asset_id = identification.asset_id;
    let asset = AssetData::Bess(
        BessData::builder()
            .availability(availability)
            .initial_soc(50.0)
            .specs(BessSpecs::builder().build())
            .identification(identification)
            .build(),
    );

    let wholesale_perimeters = vec![WholesalePerimeter {
        id: Uuid::new_v4(),
        composition: vec![asset_id],
        markets: wholesale_market_choices,
        commitments: wholesale_commitments,
    }];
    let reserve_perimeters = vec![
        ReservePerimeter::builder()
            .id(Uuid::new_v4())
            .market(AncillaryMarketType::Energy(EnergyAncillaryMarketType::AfrrFree))
            .composition(vec![asset_id])
            .commitments(reserve_commitments)
            .build(),
    ];

    let input = OptimizationInput {
        optimization_start_at: start,
        optimization_end_at: *timestamps.last().unwrap(),
        optimization_step: SignedDuration::from_mins(15),
        country: Countries::FR,
        assets: vec![asset],
        wholesale_perimeters,
        reserve_perimeters,
        ancillary_revenues,
        wholesale_revenues,
    };

    hc.do_post("/optimize", serde_json::to_value(input)?).await?.print().await?;

    Ok(())
}
