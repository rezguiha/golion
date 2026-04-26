/// Battery variable creation and constraint setting.
use std::collections::HashMap;
use super::core::BiddingKey;
use good_lp::Variable;

// region: --- Variables
#[derive(Debug)]
struct BiddingVars{
    sell:Variable,
    buy:Variable
}

// endregion: --- Variables

// region: --- Variable Stores
struct BiddingVarStore {
    store: HashMap<BiddingKey, BiddingVars>,
}
// endregion: --- Variable Stores