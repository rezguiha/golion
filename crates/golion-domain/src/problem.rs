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
pub struct OptimizationProblem {
    grid: RegularTimeGrid,
    assets: HashMap<Uuid, AssetDefinition>,
    brp_perimeters: Vec<BrpDefinition>,
    reserve_perimeters: Vec<ReserveDefinition>,
    revenues: RevenueStore,
}

impl OptimizationProblem {
    pub fn try_new(
        grid: RegularTimeGrid,
        assets: HashMap<Uuid, AssetDefinition>,
        brp_perimeters: Vec<BrpDefinition>,
        reserve_perimeters: Vec<ReserveDefinition>,
        revenues: RevenueStore,
    ) -> crate::Result<Self> {
        Self::check_compositions(&assets, &brp_perimeters, &reserve_perimeters)?;
        Self::check_single_brp_membership(&brp_perimeters)?;
        Self::check_market_revenues(&revenues, &brp_perimeters, &reserve_perimeters)?;
        Ok(Self { grid, assets, brp_perimeters, reserve_perimeters, revenues })
    }
    pub fn grid(&self) -> &RegularTimeGrid {
        &self.grid
    }
    pub fn assets(&self) -> &HashMap<Uuid, AssetDefinition> {
        &self.assets
    }
    pub fn brp_perimeters(&self) -> &[BrpDefinition] {
        &self.brp_perimeters
    }
    pub fn reserve_perimeters(&self) -> &[ReserveDefinition] {
        &self.reserve_perimeters
    }
    pub fn revenues(&self) -> &RevenueStore {
        &self.revenues
    }
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
}
// endregion: Optimization Problem

// region: Tests
#[cfg(test)]
mod tests {
    use super::OptimizationProblem;
    use crate::Error;
    use crate::asset::bess::availability::Availability;
    use crate::asset::bess::efficiency::BessPowerEfficiencies;
    use crate::asset::bess::limits::{BessLimits, SocRange};
    use crate::asset::bess::specification::BessSpecifications;
    use crate::countries::Countries;
    use crate::market::bid::ProductSpecifications;
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
        let availability: Vec<Availability> = grid()
            .iter()
            .map(|dt| Availability {
                start_at: dt,
                max_charge_power: KiloWatt(50.0),
                max_discharge_power: KiloWatt(50.0),
                max_usable_energy: KiloWattHour(100.0),
            })
            .collect();
        let limits = BessLimits {
            soc_range: SocRange {
                min_soc: 0.0.try_into().unwrap(),
                max_soc: 1.0.try_into().unwrap(),
            },
            availability: availability.try_into().unwrap(),
        };
        let efficiencies = BessPowerEfficiencies {
            charge_efficiency: Efficiency::try_from(0.95).unwrap(),
            discharge_efficiency: Efficiency::try_from(0.95).unwrap(),
        };
        AssetDefinition::Bess {
            specifications: BessSpecifications { limits, efficiencies },
            initial_soc: KiloWattHour(20.0),
        }
    }
    fn brp(composition: Vec<Uuid>, markets: Vec<MarketSpecs>) -> BrpDefinition {
        BrpDefinition::new(Uuid::new_v4(), markets, composition, vec![])
    }

    #[test]
    fn accepts_consistent_problem() {
        let asset_id = Uuid::new_v4();
        let assets = HashMap::from([(asset_id, bess())]);
        let problem = OptimizationProblem::try_new(
            grid(),
            assets,
            vec![brp(vec![asset_id], vec![])],
            Vec::new(),
            RevenueStore::default(),
        );
        assert!(problem.is_ok());
    }

    #[test]
    fn rejects_unknown_asset_in_perimeter() {
        let unknown_id = Uuid::new_v4();
        let assets = HashMap::from([(Uuid::new_v4(), bess())]);
        let problem = OptimizationProblem::try_new(
            grid(),
            assets,
            vec![brp(vec![unknown_id], vec![])],
            Vec::new(),
            RevenueStore::default(),
        );
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
        let problem = OptimizationProblem::try_new(
            grid(),
            assets,
            vec![brp(vec![asset_id], vec![]), brp(vec![asset_id], vec![])],
            Vec::new(),
            RevenueStore::default(),
        );
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
        let problem = OptimizationProblem::try_new(
            grid(),
            assets,
            vec![perimeter],
            Vec::new(),
            RevenueStore::default(),
        );
        assert!(matches!(
            problem,
            Err(Error::Market(MarketError::MissingMarketRevenue { .. }))
        ));
    }
}
// endregion: Tests
