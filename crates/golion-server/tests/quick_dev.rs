use anyhow::Result;
use golion_contract::market::choice::MarketChoice;
use golion_contract::market::commitments::{AncillaryCommitment, WholesaleCommitment};
use golion_contract::market::revenue::{AncillaryRevenue, WholesaleRevenue};
use golion_contract::optimization::input::OptimizationInput;
use golion_contract::optimization::output::BessStateOutput;
use golion_contract::perimeter::{reserve::ReservePerimeter, wholesale::BrpPerimeter};
use golion_contract::{
    asset::{
        availability::StorageAvailability,
        core::{AssetData, BessData},
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
use golion_domain::temporal::step::Step;
use jiff::RoundMode::{Expand, Trunc};
use jiff::TimestampRound;
use jiff::{SignedDuration, ToSpan, Unit, civil::date};
use uuid::Uuid;

const NB_ASSETS: u8 = 10;
const NB_BRP: u8 = 2;
const NB_BSP: u8 = 3;

fn create_bess_asset(asset_id: Uuid, grid: &RegularTimeGrid) -> AssetData {
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
    let past_grid = RegularTimeGrid::try_new_start_end(
        grid.start() - 24.hours(),
        grid.step(),
        grid.start(),
    )
    .unwrap();
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
    AssetData::Bess(
        BessData::builder()
            .availability(availability)
            .throughput(throughput)
            .initial_soc(50_000.0)
            .specs(
                BessSpecs::builder()
                    .throughput_penalty(0.0)
                    .max_rolling_cycles(1.0)
                    .build(),
            )
            .asset_id(asset_id)
            .current_state(
                BessStateOutput::builder()
                    .dispatch(1.0)
                    .ancillary_downward(1.0)
                    .ancillary_upward(1.0)
                    .soc(50_000.0)
                    .start_at(grid.start() - grid.step().span())
                    .build(),
            )
            .build(),
    )
}

fn create_wholesale_market_revenue(
    market_type: WholesaleMarketType,
    grid: &RegularTimeGrid,
    ref_sell_price: f64,
    ref_buy_price: f64,
) -> MarketSeries<WholesaleMarketType, WholesaleRevenue> {
    MarketSeries {
        market: market_type,
        values: grid
            .iter()
            .map(|t| WholesaleRevenue::SimplifiedWholesaleRevenue {
                start_at: t,
                sell_price: ref_sell_price,
                buy_price: ref_buy_price,
            })
            .collect(),
    }
}
fn create_wholesale_commitment(
    market: WholesaleMarketType,
    grid: &RegularTimeGrid,
    net_position: f64,
) -> MarketSeries<WholesaleMarketType, WholesaleCommitment> {
    MarketSeries {
        market,
        values: grid
            .iter()
            .map(|t| {
                WholesaleCommitment::builder()
                    .start_at(t)
                    .net_position(net_position)
                    .build()
            })
            .collect(),
    }
}
/// Temporary simple test of sending assetdata as a payload
/// on optimize endpoint.
#[tokio::test]
async fn test_optimize_bess() -> Result<()> {
    let hc = httpc_test::new_client("http://127.0.0.1:3000")?;
    let start_zoned = date(2026, 10, 11).at(7, 28, 0, 0).in_tz("CET")?;

    let start = start_zoned
        .timestamp()
        .round(TimestampRound::new().smallest(Unit::Minute).increment(15).mode(Expand))?;
    let end =
        start_zoned.tomorrow().and_then(|dt| dt.end_of_day())?.timestamp().round(
            TimestampRound::new().smallest(Unit::Minute).increment(15).mode(Trunc),
        )?;
    let step = Step::try_from(15.minutes()).unwrap();
    let grid = RegularTimeGrid::try_new_start_end(start, step, end).unwrap();

    // Fixed ids, in disjoint ranges per kind, so the model reaches HiGHS in the
    // same order on every run.
    let asset_ids: Vec<Uuid> =
        (1..=NB_ASSETS).map(|i| Uuid::from_u128(u128::from(i))).collect();
    let brp_ids: Vec<Uuid> =
        (1..=NB_BRP).map(|i| Uuid::from_u128(1_000 + u128::from(i))).collect();
    let bsp_ids: Vec<Uuid> =
        (1..=NB_BSP).map(|i| Uuid::from_u128(2_000 + u128::from(i))).collect();
    let asset_brp: Vec<(Uuid, Uuid)> =
        asset_ids.iter().copied().zip(brp_ids.iter().copied().cycle()).collect();
    let asset_bsp: Vec<(Uuid, Uuid)> =
        asset_ids.iter().copied().zip(bsp_ids.iter().copied().cycle()).collect();

    let brp_perimeters = brp_ids
        .iter()
        .map(|brp_id| {
            let wholesale_market_choices = vec![MarketChoice {
                market: WholesaleMarketType::IntradayContinuous,
                product_step_minutes: 15,
                product_increment_kw: 100,
            }];
            let wholesale_commitments = vec![
                create_wholesale_commitment(
                    WholesaleMarketType::IntradayAuction1,
                    &grid,
                    1000.0,
                ),
                create_wholesale_commitment(
                    WholesaleMarketType::SpotDayAhead,
                    &grid,
                    -1500.0,
                ),
            ];

            let composition = asset_brp
                .iter()
                .filter(|(_, owner)| owner == brp_id)
                .map(|&(asset_id, _)| asset_id)
                .collect();

            BrpPerimeter {
                id: *brp_id,
                composition,
                markets: wholesale_market_choices,
                commitments: wholesale_commitments,
            }
        })
        .collect::<Vec<_>>();

    let wholesale_revenues = brp_perimeters
        .iter()
        .flat_map(|brp| &brp.markets)
        .map(|choice| create_wholesale_market_revenue(choice.market, &grid, 75.0, 80.0))
        .collect::<Vec<_>>();

    let assets: Vec<AssetData> =
        asset_ids.iter().map(|asset_id| create_bess_asset(*asset_id, &grid)).collect();

    let reserve_perimeters = bsp_ids
        .iter()
        .map(|bsp_id| {
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
            let composition = asset_bsp
                .iter()
                .filter(|(_, owner)| owner == bsp_id)
                .map(|&(asset_id, _)| asset_id)
                .collect();

            ReservePerimeter::builder()
                .id(*bsp_id)
                .market(MarketChoice {
                    market: AncillaryMarketType::Energy(
                        EnergyAncillaryMarketType::AfrrFree,
                    ),
                    product_increment_kw: 1000,
                    product_step_minutes: 15,
                })
                .composition(composition)
                .commitments(reserve_commitments)
                .activation_window(30.minutes().try_into().unwrap())
                .upward_expected_activation_ratio(0.333)
                .downward_expected_activation_ratio(0.333)
                .build()
        })
        .collect::<Vec<_>>();
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
    let input = OptimizationInput {
        optimization_start_at: start,
        optimization_end_at: end,
        optimization_step: SignedDuration::from_mins(15),
        country: Countries::FR,
        assets,
        brp_perimeters,
        reserve_perimeters,
        ancillary_revenues,
        wholesale_revenues,
    };

    hc.do_post("/optimize", serde_json::to_value(input)?).await?.print().await?;

    Ok(())
}
