use thiserror::Error;

#[derive(Error, Debug, Clone, Copy, Hash, Eq, PartialEq)]
#[error("scope in token is not a superset of defined scopes.")]
pub struct ScopeMismatchError;

#[derive(Error, Debug, Clone, Copy, Hash, Eq, PartialEq)]
#[error("state from authorization response does not equal state in request")]
pub struct StatesNotEqual;

#[derive(Error, Debug)]
pub enum TokenFetchError {
    #[error("{0}")]
    Http(#[from] attohttpc::Error),

    #[error("{0}")]
    SecurityViolation(#[from] StatesNotEqual),
}
