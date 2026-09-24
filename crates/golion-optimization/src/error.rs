use derive_more::From;
#[derive(Debug, From)]
pub enum Error {
    #[from]
    Market(crate::market::core::MarketError),
    #[from]
    Physical(crate::physical::PhysicalError),
    // External
    #[from]
    Domain(golion_domain::Error),
    #[from]
    Solver(good_lp::ResolutionError),
}

pub type Result<T> = core::result::Result<T, Error>;
