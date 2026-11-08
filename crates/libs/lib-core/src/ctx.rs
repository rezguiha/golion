#[derive(Debug)]
pub enum CtxError {
    CtxNewUserCanNotBeRoot,
}

// Future implementation of user information
// for access control.
#[derive(Clone, Debug)]
pub struct Ctx {
    user_id: i64,
}

impl Ctx {
    pub fn new(user_id: i64) -> crate::Result<Self> {
        if user_id == 0 {
            Err(CtxError::CtxNewUserCanNotBeRoot.into())
        } else {
            Ok(Self { user_id })
        }
    }
    pub fn root_user() -> Self {
        Self { user_id: 0 }
    }
}
