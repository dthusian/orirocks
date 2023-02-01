use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
  #[error("error occurred while loading library")]
  LibLoadingError(#[from] libloading::Error),

  #[error("invalid version, expected `{0}` but found `{1}`")]
  InvalidVersion(u32, u32)
}