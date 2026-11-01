// region: Countries

use derive_more::Display;
use serde::{Deserialize, Serialize};
#[derive(Debug, Hash, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display)]
pub enum Countries {
    FR,
    DE,
    BE,
    ES,
    IT,
    PT,
}

// endregion: Countries
