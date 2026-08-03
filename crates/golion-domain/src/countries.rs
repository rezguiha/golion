// region: Countries

use serde::{Deserialize, Serialize};

#[derive(Debug, Hash, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Countries {
    FR,
    DE,
    BE,
    ES,
    IT,
    PT,
}

// endregion: Countries
