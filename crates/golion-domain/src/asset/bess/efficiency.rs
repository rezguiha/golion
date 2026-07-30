use crate::units::efficiency::Efficiency;
#[derive(Debug)]
pub struct BessPowerEfficiencies {
    pub charge_efficiency: Efficiency,
    pub discharge_efficiency: Efficiency,
}
