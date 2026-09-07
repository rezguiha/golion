use derive_more::From;

#[derive(Debug, From)]
pub enum Error {
    #[from]
    Market(crate::market::core::MarketError),
    // External
    #[from]
    Domain(golion_domain::Error),
}

pub type Result<T> = core::result::Result<T, Error>;
