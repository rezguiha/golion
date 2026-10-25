use derive_more::From;
#[derive(Debug, From)]
pub enum Error {
    #[from]
    Physical(crate::physical::PhysicalError),
    // External
    #[from]
    Domain(golion_domain::Error),
    #[from]
    Solver(good_lp::ResolutionError),
    #[from]
    MipGap(good_lp::solvers::MipGapError),
}

pub type Result<T> = core::result::Result<T, Error>;
