#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    UserError(String),
    InternalError(String),
}

impl Error {
    pub fn is_user(&self) -> bool {
        match self {
            Error::UserError(_) => true,
            _ => false,
        }
    }

    pub fn is_internal(&self) -> bool {
        match self {
            Error::InternalError(_) => true,
            _ => false,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Error::UserError(s) => s.clone(),
            Error::InternalError(s) => s.clone(),
        }
    }

    pub fn prepend(&self, prefix: &str) -> Error {
        match self {
            Error::UserError(s) => Error::UserError(format!("{}: {}", prefix, s)),
            Error::InternalError(s) => Error::InternalError(format!("{}: {}", prefix, s)),
        }
    }
}
