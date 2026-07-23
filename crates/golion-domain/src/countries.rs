use crate::serde_type;
// region: Countries
serde_type! {
    #[derive(Debug, Hash, Clone, PartialEq, Eq)]
    pub enum Countries {
        FR,
        DE,
        BE,
        ES,
        IT,
        PT,
    }
}
// endregion: Countries
