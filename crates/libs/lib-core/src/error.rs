use derive_more::From;
#[derive(Debug, From)]
pub enum Error {
    EntityNotFound { name: &'static str, id: i64 },
    // --- Modules
    Store(crate::model::store::Error),
    Context(crate::ctx::CtxError),
    // --- Externals
    Sqlx(sqlx::Error),
}

impl core::fmt::Display for Error {
    fn fmt(
        &self,
        fmt: &mut core::fmt::Formatter,
    ) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {}
pub type Result<T> = core::result::Result<T, Error>;
