use thiserror::Error;

/// Every failure the library reports.
///
/// `Invalid` is a value rejected before it reaches the hardware, or a reply
/// that does not decode. `Device` is the transport failing or the mouse not
/// answering. `Profile` is the bundled vendor definition failing to load.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum DarmosharkError {
  #[error("{0}")]
  Invalid(String),
  #[error("{0}")]
  Device(String),
  #[error("{0}")]
  Profile(String),
}

pub type DarmosharkResult<T> = Result<T, DarmosharkError>;
