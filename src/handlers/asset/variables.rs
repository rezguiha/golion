/// Battery variable creation and constraint setting.
use std::collections::HashMap;

use good_lp::Variable;

use super::core::PhysicalKey;

// region: --- Variables
/// Asset type variable definitions.
#[derive(Debug)]
struct BatteryVars {
    /// Discharge output power expressed in kW.
    output_power: Variable,
    /// Charge input power expressed in kW.
    input_power: Variable,
    /// State of charge variable expressed in percentage \[0,1\]
    soc: Variable,
}

#[derive(Debug)]
struct GeneratorVars {
    output_power: Variable,
}
// endregion: --- Variables

// region: --- Variable Stores
/// Variable Stores serve the purpose of having quick
/// single row search by a well defined hash key for physical variables
/// that will be needed to be linked with other components like
/// bidding variables to define constraints linking both.
#[derive(Debug, Default)]
struct BatteryVarStore {
    store: HashMap<PhysicalKey, BatteryVars>,
}
#[derive(Debug, Default)]
struct GeneratorStore {
    store: HashMap<PhysicalKey, GeneratorVars>,
}
// endregion: --- Variable Stores
