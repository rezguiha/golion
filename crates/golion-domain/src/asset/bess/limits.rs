use crate::units::power::{KiloWatt, KiloWattHour};
// region: Battery Limits
/// Bess physical limits coming from availability
/// information and asset management constraints
/// where for example we want to operate in a range of
/// state of charge.
pub struct BessLimits {
    pub max_output_power: KiloWatt,
    pub max_input_power: KiloWatt,
    pub min_soc: KiloWattHour,
    pub max_soc: KiloWattHour,
}
// endregion: Battery Limits
