use derive_more::From;
#[derive(Debug, From)]
pub enum Error {
    // --- External
    #[from]
    Domain(golion_domain::Error),
}

pub type Result<T> = core::result::Result<T, Error>;
