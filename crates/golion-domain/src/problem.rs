pub mod definition;
pub mod error;

use crate::market::revenue::RevenueStore;
use crate::problem::definition::{AssetDefinition, BrpDefinition, ReserveDefinition};
use crate::problem::error::ProblemError;
use crate::temporal::grid::RegularTimeGrid;
use std::collections::HashMap;
use uuid::Uuid;

// region: Optimization Problem
/// Complete description of an optimization problem. Its invariants
/// are checked at construction, so any existing problem is consistent
/// and can be modelled without further validation.
#[derive(Debug)]
pub struct ProblemData {
    pub grid: RegularTimeGrid,
    pub assets: HashMap<Uuid, AssetDefinition>,
    pub brp_perimeters: Vec<BrpDefinition>,
    pub reserve_perimeters: Vec<ReserveDefinition>,
    pub revenues: RevenueStore,
}
#[derive(Debug)]
pub struct OptimizationProblem(ProblemData);
impl OptimizationProblem {
    /// Make sure every perimeter is only composed of known assets.
    fn check_compositions(
        assets: &HashMap<Uuid, AssetDefinition>,
        brp_perimeters: &[BrpDefinition],
        reserve_perimeters: &[ReserveDefinition],
    ) -> crate::Result<()> {
        let compositions = brp_perimeters
            .iter()
            .map(|perimeter| (*perimeter.id(), perimeter.composition()))
            .chain(
                reserve_perimeters
                    .iter()
                    .map(|perimeter| (*perimeter.id(), perimeter.composition())),
            );
        for (perimeter_id, composition) in compositions {
            let asset_ids: Vec<Uuid> = composition
                .iter()
                .filter(|id| !assets.contains_key(id))
                .copied()
                .collect();
            if !asset_ids.is_empty() {
                return Err(ProblemError::UnknownAssetsInPerimeter {
                    perimeter_id,
                    asset_ids,
                }
                .into());
            }
        }
        Ok(())
    }
    /// Make sure an asset is in at most one balance responsible party
    /// perimeter.
    fn check_single_brp_membership(
        brp_perimeters: &[BrpDefinition],
    ) -> crate::Result<()> {
        let mut counter = HashMap::new();
        for perimeter in brp_perimeters.iter() {
            for id in perimeter.composition().iter() {
                *counter.entry(*id).or_insert(0) += 1;
            }
        }
        let asset_ids: Vec<Uuid> = counter
            .into_iter()
            .filter(|(_, count)| *count > 1)
            .map(|(id, _)| id)
            .collect();
        if asset_ids.is_empty() {
            Ok(())
        } else {
            Err(ProblemError::AssetInMultipleBrps { asset_ids }.into())
        }
    }
    /// Make sure a revenue series is available for every market bid on.
    fn check_market_revenues(
        revenues: &RevenueStore,
        brp_perimeters: &[BrpDefinition],
        reserve_perimeters: &[ReserveDefinition],
    ) -> crate::Result<()> {
        let markets = brp_perimeters
            .iter()
            .flat_map(|perimeter| perimeter.markets().iter())
            .chain(reserve_perimeters.iter().map(|perimeter| perimeter.market()));
        for specs in markets {
            revenues.get(specs.market, specs.country)?;
        }
        Ok(())
    }
    pub fn into_data(self) -> ProblemData {
        self.0
    }
    pub fn data(&self) -> &ProblemData {
        &self.0
    }
}
impl TryFrom<ProblemData> for OptimizationProblem {
    type Error = crate::Error;
    fn try_from(data: ProblemData) -> crate::Result<Self> {
        Self::check_compositions(
            &data.assets,
            &data.brp_perimeters,
            &data.reserve_perimeters,
        )?;
        Self::check_single_brp_membership(&data.brp_perimeters)?;
        Self::check_market_revenues(
            &data.revenues,
            &data.brp_perimeters,
            &data.reserve_perimeters,
        )?;
        Ok(Self(data))
    }
}
// endregion: Optimization Problem

// region: Tests
#[cfg(test)]
mod tests {
    use super::{OptimizationProblem, ProblemData};
    use crate::Error;
    use crate::asset::bess::availability::Availability;
    use crate::asset::bess::efficiency::BessPowerEfficiencies;
    use crate::asset::bess::soc::SocRange;
    use crate::asset::bess::specification::BessSpecifications;
    use crate::asset::bess::throughput::{
        BidirectionalThroughput, Cycling, ThroughputHistory,
    };
    use crate::countries::Countries;
    use crate::market::bid::ProductSpecifications;
    use crate::market::commitment::EnergyCommitment;
    use crate::market::error::MarketError;
    use crate::market::market_type::{MarketType, WholesaleMarketType};
    use crate::market::revenue::RevenueStore;
    use crate::market::specification::MarketSpecs;
    use crate::problem::definition::{AssetDefinition, BrpDefinition};
    use crate::problem::error::ProblemError;
    use crate::temporal::grid::RegularTimeGrid;
    use crate::temporal::step::MinuteStep;
    use crate::units::efficiency::Efficiency;
    use crate::units::power::{KiloWatt, KiloWattHour};
    use jiff::{SignedDuration, Timestamp, ToSpan};
    use std::collections::HashMap;
    use uuid::Uuid;

    fn step() -> MinuteStep {
        MinuteStep::try_from(SignedDuration::from_mins(15)).unwrap()
    }
    fn start() -> Timestamp {
        "2026-01-01T00:00:00Z".parse().unwrap()
    }
    fn grid() -> RegularTimeGrid {
        RegularTimeGrid::try_new(start(), step(), 2).unwrap()
    }
    fn bess() -> AssetDefinition {
        let rated_charge_power = KiloWatt(50.0);
        let rated_discharge_power = KiloWatt(50.0);
        let rated_energy = KiloWattHour(100.0);
        let availability: Vec<Availability> = grid()
            .iter()
            .map(|dt| Availability {
                start_at: dt,
                max_charge_power: rated_charge_power,
                max_discharge_power: rated_discharge_power,
                max_usable_energy: rated_energy,
            })
            .collect();

        let efficiencies = BessPowerEfficiencies {
            charge_efficiency: Efficiency::try_from(0.95).unwrap(),
            discharge_efficiency: Efficiency::try_from(0.95).unwrap(),
        };
        // Zero throughput over the 24h preceding the grid start.
        let history: Vec<BidirectionalThroughput> =
            RegularTimeGrid::try_new(start() - 24.hours(), step(), 96)
                .unwrap()
                .iter()
                .map(|dt| BidirectionalThroughput {
                    start_at: dt,
                    charged_energy: KiloWattHour(0.0),
                    discharged_energy: KiloWattHour(0.0),
                })
                .collect();
        let cycling = Cycling {
            history: ThroughputHistory::Bidirectional(history.try_into().unwrap()),
            max_24h_avg_cycles: 1.0,
            penalty: 0.0,
        };
        AssetDefinition::Bess {
            specifications: BessSpecifications {
                soc_range: SocRange::try_new(
                    0.0.try_into().unwrap(),
                    1.0.try_into().unwrap(),
                )
                .unwrap(),
                availability: availability.try_into().unwrap(),
                cycling,
                efficiencies,
                rated_charge_power,
                rated_discharge_power,
                rated_energy,
            },
            initial_soc: KiloWattHour(20.0),
        }
    }
    fn brp(composition: Vec<Uuid>, markets: Vec<MarketSpecs>) -> BrpDefinition {
        BrpDefinition::try_new(
            Uuid::new_v4(),
            markets,
            composition,
            Vec::<EnergyCommitment>::new().into_iter(),
            &grid(),
        )
        .unwrap()
    }
    /// Problem data over the test grid, without reserve perimeters nor revenues.
    fn data(
        assets: HashMap<Uuid, AssetDefinition>,
        brp_perimeters: Vec<BrpDefinition>,
    ) -> ProblemData {
        ProblemData {
            grid: grid(),
            assets,
            brp_perimeters,
            reserve_perimeters: Vec::new(),
            revenues: RevenueStore::default(),
        }
    }

    #[test]
    fn accepts_consistent_problem() {
        let asset_id = Uuid::new_v4();
        let assets = HashMap::from([(asset_id, bess())]);
        let problem = OptimizationProblem::try_from(data(
            assets,
            vec![brp(vec![asset_id], vec![])],
        ));
        assert!(problem.is_ok());
    }

    #[test]
    fn rejects_unknown_asset_in_perimeter() {
        let unknown_id = Uuid::new_v4();
        let assets = HashMap::from([(Uuid::new_v4(), bess())]);
        let problem = OptimizationProblem::try_from(data(
            assets,
            vec![brp(vec![unknown_id], vec![])],
        ));
        assert!(matches!(
            problem,
            Err(Error::Problem(ProblemError::UnknownAssetsInPerimeter { asset_ids, .. }))
                if asset_ids == vec![unknown_id]
        ));
    }

    #[test]
    fn rejects_asset_in_multiple_brps() {
        let asset_id = Uuid::new_v4();
        let assets = HashMap::from([(asset_id, bess())]);
        let problem = OptimizationProblem::try_from(data(
            assets,
            vec![brp(vec![asset_id], vec![]), brp(vec![asset_id], vec![])],
        ));
        assert!(matches!(
            problem,
            Err(Error::Problem(ProblemError::AssetInMultipleBrps { asset_ids }))
                if asset_ids == vec![asset_id]
        ));
    }

    #[test]
    fn rejects_market_without_revenue() {
        let asset_id = Uuid::new_v4();
        let assets = HashMap::from([(asset_id, bess())]);
        let product = ProductSpecifications::try_new(15.minutes(), 10).unwrap();
        let specs = MarketSpecs::try_new(
            MarketType::WholeSale(WholesaleMarketType::SpotDayAhead),
            Countries::FR,
            product,
        )
        .unwrap();
        let perimeter = brp(vec![asset_id], vec![specs]);
        let problem = OptimizationProblem::try_from(data(assets, vec![perimeter]));
        assert!(matches!(
            problem,
            Err(Error::Market(MarketError::MissingMarketRevenue { .. }))
        ));
    }
}
// endregion: Tests
