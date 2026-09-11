use thiserror::Error;

/// Every failure the library reports.
///
/// `Invalid` is a value rejected before it reaches the hardware, or a reply
/// that does not decode. `Device` is the transport failing or the mouse not
/// answering. `Asleep` is the receiver reporting no link (status 2), which in
/// practice means the mouse went to sleep. `Profile` is the bundled vendor
/// definition failing to load.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum DarmosharkError {
  #[error("{0}")]
  Invalid(String),
  #[error("{0}")]
  Device(String),
  #[error(
    "the receiver has no link to the mouse, which usually means the mouse fell asleep. \
     Move it or click to wake it, then try again; if it stays down, check that the \
     switch is on 2.4G."
  )]
  Asleep,
  #[error("{0}")]
  Profile(String),
}

pub type DarmosharkResult<T> = Result<T, DarmosharkError>;
