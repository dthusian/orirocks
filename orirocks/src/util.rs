use std::fmt::{Display, Formatter, Write};
use std::io;
use std::ops::{Deref, DerefMut};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ORError {
  #[error("in `{0}`: syntax error: `{1}`")]
  YamlError(YamlLocation, serde_yaml::Error),

  #[error("error occurred while performing i/o: `{0}`")]
  IoError(io::Error),

  #[error("in `{0}`: duplicate `{1}` `{2}`")]
  DuplicateSymbol(YamlLocation, String, String),

  #[error("in `{0}`: invalid character in identifier")]
  InvalidCharacter(YamlLocation)
}

pub type ORResult<T> = std::result::Result<T, ORError>;

#[derive(Default, Clone, Debug, Eq, PartialEq)]
pub struct YamlLocation {
  pub file: String,
  pub document_id: usize,
  pub path: Vec<String>
}

impl YamlLocation {
  pub fn new(file: String, document_id: usize, path: Vec<String>) -> Self {
    YamlLocation {
      file,
      document_id,
      path
    }
  }

  pub fn push(&mut self, path: String) {
    self.path.push(path);
  }

  pub fn pop(&mut self) {
    self.path.pop();
  }
}

impl Display for YamlLocation {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    f.write_str(&format!("{}: document #{}: {}", self.file, self.document_id, self.path.join("/")))
  }
}

#[derive(Default, Clone, Debug, Eq, PartialEq)]
pub struct Located<T> {
  location: YamlLocation,
  obj: T
}

impl<T> Located<T> {
  pub fn new(location: YamlLocation, obj: T) -> Self {
    Located {
      location,
      obj
    }
  }

  pub fn location(s: &Located<T>) -> &YamlLocation {
    &s.location
  }
}

impl<T> Deref for Located<T> {
  type Target = T;

  fn deref(&self) -> &Self::Target {
    &self.obj
  }
}

impl<T> DerefMut for Located<T> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.obj
  }
}

/// Validates that the identifier only contains allowed characters.
/// These are a-zA-Z0-9_
pub fn validate_identifier(s: &str, traceback: &YamlLocation) -> ORResult<()> {
  if !s.chars().all(|v| {
    v.is_ascii_alphanumeric() || v == '_'
  }) {
    Err(ORError::InvalidCharacter(traceback.clone()))
  } else {
    Ok(())
  }
}