use anyhow::Result;
use golion_contract::market::choice::MarketChoice;
use golion_contract::market::commitments::{AncillaryCommitment, WholesaleCommitment};
use golion_contract::market::revenue::{AncillaryRevenue, WholesaleRevenue};
use golion_contract::optimization::input::OptimizationInput;
use golion_contract::perimeter::{reserve::ReservePerimeter, wholesale::BrpPerimeter};
use golion_contract::{
    asset::{
        availability::StorageAvailability,
        core::{AssetData, BessData},
        identification::AssetIdentification,
        specifications::BessSpecs,
        throughput::{BidirectionalThroughputContract, ThroughputData},
    },
    market::series::MarketSeries,
};
use golion_domain::countries::Countries;
use golion_domain::market::market_type::{
    AncillaryMarketType, EnergyAncillaryMarketType, WholesaleMarketType,
};
use golion_domain::temporal::grid::RegularTimeGrid;
use golion_domain::temporal::step::MinuteStep;
use jiff::RoundMode::Trunc;
use jiff::TimestampRound;
use jiff::{SignedDuration, ToSpan, Unit, civil::date};
use uuid::Uuid;
/// Temporary simple test of sending assetdata as a payload
/// on optimize endpoint.
#[tokio::test]
async fn test_optimize_bess() -> Result<()> {
    let hc = httpc_test::new_client("http://127.0.0.1:3000")?;
    let start_zoned = date(2026, 10, 11).at(7, 30, 0, 0).in_tz("CET")?;
    let start = start_zoned.timestamp();
    let end =
        start_zoned.tomorrow().and_then(|dt| dt.end_of_day())?.timestamp().round(
            TimestampRound::new().smallest(Unit::Minute).increment(15).mode(Trunc),
        )?;
    let step = MinuteStep::try_new(15).unwrap();
    let grid = RegularTimeGrid::try_new_start_end(start, step, end).unwrap();
    let wholesale_market_choices = vec![MarketChoice {
        market: WholesaleMarketType::IntradayContinuous,
        product_step_minutes: 15,
        product_increment_kw: 100,
    }];
    let availability = grid
        .iter()
        .map(|t| {
            StorageAvailability::builder()
                .start_at(t)
                .max_charge_power(20_000.0)
                .max_discharge_power(20_000.0)
                .max_usable_energy(100_000.0)
                .build()
        })
        .collect();
    let reserve_commitments: Vec<AncillaryCommitment> = grid
        .iter()
        .map(|t| {
            AncillaryCommitment::builder()
                .start_at(t)
                .upward_power(0.0)
                .downward_power(2.0)
                .build()
        })
        .collect();
    let wholesale_commitments = vec![
        MarketSeries {
            market: WholesaleMarketType::IntradayAuction1,
            values: grid
                .iter()
                .map(|t| {
                    WholesaleCommitment::builder()
                        .start_at(t)
                        .net_position(1000.0)
                        .build()
                })
                .collect(),
        },
        MarketSeries {
            market: WholesaleMarketType::SpotDayAhead,
            values: grid
                .iter()
                .map(|t| {
                    WholesaleCommitment::builder()
                        .start_at(t)
                        .net_position(-1500.0)
                        .build()
                })
                .collect(),
        },
    ];

    let ancillary_revenues = vec![MarketSeries {
        market: AncillaryMarketType::Energy(EnergyAncillaryMarketType::AfrrFree),
        values: grid
            .iter()
            .map(|t| AncillaryRevenue::SimplifiedAncillaryRevenue {
                start_at: t,
                sell_revenue: 15.0,
                buy_revenue: 15.0,
            })
            .collect(),
    }];
    let wholesale_revenues = vec![MarketSeries {
        market: WholesaleMarketType::IntradayContinuous,
        values: grid
            .iter()
            .map(|t| WholesaleRevenue::SimplifiedWholesaleRevenue {
                start_at: t,
                sell_price: 80.0,
                buy_price: 75.0,
            })
            .collect(),
    }];
    let past_grid =
        RegularTimeGrid::try_new_start_end(start - 24.hours(), step, start).unwrap();
    let throughput = ThroughputData::Bidirectional(
        past_grid
            .iter()
            .map(|t| {
                BidirectionalThroughputContract::builder()
                    .start_at(t)
                    .charged_energy(0.0)
                    .discharged_energy(0.0)
                    .build()
            })
            .collect(),
    );
    let identification = AssetIdentification::builder().build();
    let asset_id = identification.asset_id;
    let asset = AssetData::Bess(
        BessData::builder()
            .availability(availability)
            .throughput(throughput)
            .initial_soc(50_000.0)
            .specs(
                BessSpecs::builder()
                    .throughput_penalty(0.0)
                    .max_rolling_cycles(2.0)
                    .build(),
            )
            .identification(identification)
            .build(),
    );

    let brp_perimeters = vec![BrpPerimeter {
        id: Uuid::new_v4(),
        composition: vec![asset_id],
        markets: wholesale_market_choices,
        commitments: wholesale_commitments,
    }];
    let reserve_perimeters = vec![
        ReservePerimeter::builder()
            .id(Uuid::new_v4())
            .market(MarketChoice {
                market: AncillaryMarketType::Energy(EnergyAncillaryMarketType::AfrrFree),
                product_increment_kw: 1000,
                product_step_minutes: 15,
            })
            .composition(vec![asset_id])
            .commitments(reserve_commitments)
            .activation_window(30.minutes().try_into().unwrap())
            .upward_expected_activation_ratio(0.333)
            .downward_expected_activation_ratio(0.333)
            .build(),
    ];

    let input = OptimizationInput {
        optimization_start_at: start,
        optimization_end_at: end,
        optimization_step: SignedDuration::from_mins(15),
        country: Countries::FR,
        assets: vec![asset],
        brp_perimeters,
        reserve_perimeters,
        ancillary_revenues,
        wholesale_revenues,
    };

    hc.do_post("/optimize", serde_json::to_value(input)?).await?.print().await?;

    Ok(())
}
